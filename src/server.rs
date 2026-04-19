//! HTTP Server.

use axum::extract::ws::{self, WebSocket};
use axum::{
    extract::{State, WebSocketUpgrade},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
    routing::{get, post},
    Router,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};

/// Build the Axum router with given state. Used for testing.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/v1/chat", post(handle_chat))
        .route("/v1/tools/execute", post(handle_tool))
        .route("/v1/cluster/sync", post(handle_cluster_sync))
        .route("/v1/ws", get(handle_websocket))
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
            .with_state(state);

        let addr =
            std::env::var("FEMTO_BIND_ADDR").unwrap_or_else(|_| format!("127.0.0.1:{}", self.port));
        tracing::info!("Starting FemtoClaw server on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn handle_cluster_sync(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(messages): Json<Vec<ChatMessage>>,
) -> impl IntoResponse {
    if !state.is_authorized(&headers).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let internal_messages = messages
        .into_iter()
        .map(|m| crate::state::Message {
            role: m.role,
            content: m.content,
        })
        .collect();
    state.sync_from_remote(internal_messages).await;
    Ok("OK")
}

async fn health_check() -> &'static str {
    "FemtoClaw OK"
}

async fn handle_chat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ChatRequest>,
) -> impl IntoResponse {
    if !state.is_authorized(&headers).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ChatResponse {
                message: Some("Unauthorized".to_string()),
                tool_call: None,
            }),
        );
    }

    tracing::debug!("Chat request: {:?}", request);

    let input = request
        .messages
        .last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    match state.get_agent().await {
        Some(agent) => {
            match agent.run(&input).await {
                Ok(reply) => {
                    // Record message in app state (broadcasts to cluster)
                    state
                        .add_message(crate::state::Message {
                            role: "user".to_string(),
                            content: input,
                        })
                        .await;
                    state
                        .add_message(crate::state::Message {
                            role: "assistant".to_string(),
                            content: reply.clone(),
                        })
                        .await;

                    (
                        StatusCode::OK,
                        Json(ChatResponse {
                            message: Some(reply),
                            tool_call: None,
                        }),
                    )
                }
                Err(e) => {
                    tracing::error!("Agent error: {}", e);
                    (
                        StatusCode::OK,
                        Json(ChatResponse {
                            message: Some(format!("Agent error: {}", e)),
                            tool_call: None,
                        }),
                    )
                }
            }
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ChatResponse {
                message: Some("Agent not initialized".to_string()),
                tool_call: None,
            }),
        ),
    }
}

async fn handle_tool(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ToolRequest>,
) -> impl IntoResponse {
    if !state.is_authorized(&headers).await {
        return (
            StatusCode::UNAUTHORIZED,
            Json(ToolResponse {
                result: "Unauthorized".to_string(),
            }),
        );
    }

    tracing::debug!("Tool request: {:?}", request);

    match state.get_agent().await {
        Some(agent) => {
            match agent
                .execute_tool(&request.tool, request.args.clone())
                .await
            {
                Ok(result) => (StatusCode::OK, Json(ToolResponse { result })),
                Err(e) => {
                    tracing::error!("Tool error: {}", e);
                    (
                        StatusCode::BAD_REQUEST,
                        Json(ToolResponse {
                            result: format!("Error: {}", e),
                        }),
                    )
                }
            }
        }
        None => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ToolResponse {
                result: "Agent not initialized".to_string(),
            }),
        ),
    }
}

async fn handle_websocket(
    State(state): State<AppState>,
    headers: HeaderMap,
    ws: WebSocketUpgrade,
) -> Response {
    if !state.is_authorized(&headers).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(|socket: WebSocket| async move {
        let (mut sender, mut receiver) = socket.split();

        while let Some(Ok(msg)) = receiver.next().await {
            if let ws::Message::Text(text) = msg {
                tracing::debug!("WebSocket received: {}", text);
                let response = if let Some(agent) = state.get_agent().await {
                    match agent.run(&text).await {
                        Ok(reply) => {
                            state
                                .add_message(crate::state::Message {
                                    role: "user".to_string(),
                                    content: text.to_string(),
                                })
                                .await;
                            state
                                .add_message(crate::state::Message {
                                    role: "assistant".to_string(),
                                    content: reply.clone(),
                                })
                                .await;
                            reply
                        }
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
    .into_response()
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
        let response = handle_chat(axum::extract::State(state), HeaderMap::new(), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_handle_tool_shell() {
        let state = AppState::new("test-node".to_string());
        state.init_agent().await.unwrap();
        let request = ToolRequest {
            tool: "shell".to_string(),
            args: json!({"bin":"echo","argv":["hello"]}),
        };
        let response = handle_tool(axum::extract::State(state), HeaderMap::new(), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_tool_unknown_denied() {
        let state = AppState::new("test-node".to_string());
        state.init_agent().await.unwrap();
        let request = ToolRequest {
            tool: "unknown".to_string(),
            args: json!({}),
        };
        let response = handle_tool(axum::extract::State(state), HeaderMap::new(), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_handle_tool_requires_auth_when_configured() {
        let state = AppState::new("test-node".to_string());
        state.config.write().await.api_key = Some("secret".to_string());
        state.init_agent().await.unwrap();
        let request = ToolRequest {
            tool: "unknown".to_string(),
            args: json!({}),
        };
        let response = handle_tool(axum::extract::State(state), HeaderMap::new(), Json(request))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_handle_cluster_sync_requires_auth_when_configured() {
        let state = AppState::new("test-node".to_string());
        state.config.write().await.api_key = Some("secret".to_string());
        let response =
            handle_cluster_sync(axum::extract::State(state), HeaderMap::new(), Json(vec![]))
                .await
                .into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    }

    // WebSocket test would require a real WebSocket connection; deferred.
}
