//! HTTP Server.

use axum::{
    routing::{get, post},
    Router,
};

pub struct Server {
    port: u16,
}

impl Server {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(health_check))
            .route("/v1/chat", post(handle_chat))
            .route("/v1/tools/execute", post(handle_tool));

        let addr = format!("127.0.0.1:{}", self.port);
        tracing::info!("Starting server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_check() -> &'static str {
    "FemtoClaw OK"
}

async fn handle_chat() -> &'static str {
    "chat"
}

async fn handle_tool() -> &'static str {
    "tool"
}
