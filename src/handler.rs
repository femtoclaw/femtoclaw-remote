//! Request Handler.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

pub struct Handler;

impl Handler {
    pub fn new() -> Self {
        Self
    }

    pub async fn handle(&self, request: Request) -> Response {
        tracing::debug!("Handling request: {}", request.method);
        
        Response {
            id: uuid::Uuid::new_v4().to_string(),
            result: Some(serde_json::json!({"status": "ok"})),
            error: None,
        }
    }
}

impl Default for Handler {
    fn default() -> Self {
        Self::new()
    }
}
