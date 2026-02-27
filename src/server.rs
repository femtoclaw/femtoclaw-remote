//! HTTP Server.

use axum::{
    extract::State,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

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
        let app = Router::new()
            .route("/", get(health_check))
            .route("/health", get(health_check))
            .route("/v1/chat", post(handle_chat))
            .route("/v1/tools/execute", post(handle_tool))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        tracing::info!("Starting FemtoClaw server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_check() -> &'static str {
    "FemtoClaw OK"
}

async fn handle_chat(
    State(_state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Json<ChatResponse> {
    tracing::debug!("Chat request: {:?}", request);
    
    let input = request.messages.last()
        .map(|m| m.content.clone())
        .unwrap_or_default();

    Json(ChatResponse {
        message: Some(format!("Echo: {}", input)),
        tool_call: None,
    })
}

async fn handle_tool(
    State(_state): State<AppState>,
    Json(request): Json<ToolRequest>,
) -> Json<ToolResponse> {
    tracing::debug!("Tool request: {:?}", request);
    
    Json(ToolResponse {
        result: format!("Executed {} with args: {:?}", request.tool, request.args),
    })
}
