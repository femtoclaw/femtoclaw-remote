//! HTTP Server.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};

use crate::state::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest {
    pub messages: Vec<crate::types::Message>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatResponse {
    pub message: Option<String>,
    pub tool_call: Option<crate::types::ToolCall>,
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
    State(state): State<AppState>,
    Json(request): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, StatusCode> {
    tracing::debug!("Chat request: {:?}", request);
    
    let response = ChatResponse {
        message: Some("Echo: ".to_string()),
        tool_call: None,
    };

    Ok(Json(response))
}

async fn handle_tool(
    State(_state): State<AppState>,
    Json(request): Json<ToolRequest>,
) -> Result<Json<ToolResponse>, StatusCode> {
    tracing::debug!("Tool request: {:?}", request);
    
    let response = ToolResponse {
        result: format!("Executed {} with args: {:?}", request.tool, request.args),
    };

    Ok(Json(response))
}
