//! HTTP Server.

use axum::{
    extract::{State, WebSocketUpgrade},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tower_http::cors::CorsLayer;
use axum::extract::ws::{self, WebSocket};

/// Build the Axum router with given state. Used for testing.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/v1/chat", post(handle_chat))
        .route("/v1/tools/execute", post(handle_tool))
        .route("/v1/cluster/sync", post(handle_cluster_sync))
        .route("/v1/ws", get(handle_websocket))
        .layer(CorsLayer::permissive())
        .with_state(state)
}

use crate::state::AppState;
use crate::types::ToolCall;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<ChatMessage>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Option<String>,
    pub tool_call: Option<ToolCall>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolRequest {
    pub tool: String,
    pub args: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResponse {
    pub result: String,
}

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&self, state: AppState) -> anyhow::Result<()> {
        // Initialize the agent if not already done.
        if state.get_agent().await.is_none() {
            state.init_agent().await?;
        }
        state.init_cluster().await;

        let app = Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check))
            .route("/v1/chat", post(handle_chat))
            .route("/v1/tools/execute", post(handle_tool))
            .route("/v1/cluster/sync", post(handle_cluster_sync))
            .route("/v1/ws", get(handle_websocket))
            .layer(CorsLayer::permissive())
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        tracing::info!("Starting FemtoClaw server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn handle_cluster_sync(
    State(state): State<AppState>,
    Json(messages): Json<Vec<ChatMessage>>,
) -> &'static str {
    let internal_messages = messages.into_iter().map(|m| crate::state::Message {
        role: m.role,
        content: m.content,
    }).collect();
    state.sync_from_remote(internal_messages).await;
    "OK"
}

async fn health_check() -> &'static str {
    "FemtoClaw OK"
}

async fn handle_chat(
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    tracing::debug!("Chat request: {:?}", request);
    
    let input = request.messages.last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    match state.get_agent().await {
        Some(agent) => {
            match agent.run(&input).await {
                Ok(reply) => {
                    // Record message in app state (broadcasts to cluster)
                    state.add_message(crate::state::Message {
                        role: "user".to_string(),
                        content: input,
                    }).await;
                    state.add_message(crate::state::Message {
                        role: "assistant".to_string(),
                        content: reply.clone(),
                    }).await;

                    Json(ChatResponse {
                        message: Some(reply),
                        tool_call: None,
                    })
                },
                Err(e) => {
                    tracing::error!("Agent error: {}", e);
                    Json(ChatResponse {
                        message: Some(format!("Agent error: {}", e)),
                        tool_call: None,
                    })
                }
            }
        }
        None => Json(ChatResponse {
            message: Some("Agent not initialized".to_string()),
            tool_call: None,
        })
    }
}

async fn handle_tool(
    State(state): State<AppState>,
    Json(request): Json<ToolRequest>,
) -> Json<ToolResponse> {
    tracing::debug!("Tool request: {:?}", request);
    
    match state.get_agent().await {
        Some(agent) => {
            match agent.execute_tool(&request.tool, request.args.clone()).await {
                Ok(result) => Json(ToolResponse { result }),
                Err(e) => {
                    tracing::error!("Tool error: {}", e);
                    Json(ToolResponse { result: format!("Error: {}", e) })
                }
            }
        }
        None => Json(ToolResponse { result: "Agent not initialized".to_string() }),
    }
}

async fn handle_websocket(
    State(state): State<AppState>,
    ws: WebSocketUpgrade,
) -> impl IntoResponse {
    ws.on_upgrade(|socket: WebSocket| async move {
        let (mut sender, mut receiver) = socket.split();

        while let Some(Ok(msg)) = receiver.next().await {
            if let ws::Message::Text(text) = msg {
                tracing::debug!("WebSocket received: {}", text);
                let response = if let Some(agent) = state.get_agent().await {
                    match agent.run(&text).await {
                        Ok(reply) => {
                            state.add_message(crate::state::Message {
                                role: "user".to_string(),
                                content: text.to_string(),
                            }).await;
                            state.add_message(crate::state::Message {
                                role: "assistant".to_string(),
                                content: reply.clone(),
                            }).await;
                            reply
                        },
                        Err(e) => format!("Error: {}", e),
                    }
                } else {
                    "Agent not initialized".to_string()
                };

                if let Err(e) = sender.send(ws::Message::Text(response.into())).await {
                    tracing::error!("WebSocket send error: {}", e);
                    break;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_check() {
        let response = health_check().await;
        assert_eq!(response, "FemtoClaw OK");
    }

    #[tokio::test]
    async fn test_handle_chat_echo() {
        let state = AppState::new("test-node".to_string());
        state.init_agent().await.unwrap();
        let request = ChatRequest {
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: "ping".to_string(),
            }],
        };
        let response = handle_chat(axum::extract::State(state), Json(request)).await;
        assert!(response.message.is_some());
        let msg = response.message.clone().unwrap();
        assert!(msg.contains("ACK: ping"));
    }

    #[tokio::test]
    async fn test_handle_tool_shell() {
        let state = AppState::new("test-node".to_string());
        state.init_agent().await.unwrap();
        let request = ToolRequest {
            tool: "shell".to_string(),
            args: json!({"bin":"echo","argv":["hello"]}),
        };
        let response = handle_tool(axum::extract::State(state), Json(request)).await;
        assert!(response.result.contains("hello"));
    }

    #[tokio::test]
    async fn test_handle_tool_unknown_denied() {
        let state = AppState::new("test-node".to_string());
        state.init_agent().await.unwrap();
        let request = ToolRequest {
            tool: "unknown".to_string(),
            args: json!({}),
        };
        let response = handle_tool(axum::extract::State(state), Json(request)).await;
        assert!(response.result.contains("denied") || response.result.contains("Capability denied"));
    }

    // WebSocket test would require a real WebSocket connection; deferred.
}
