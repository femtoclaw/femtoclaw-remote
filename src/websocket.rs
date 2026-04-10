//! WebSocket Handler.

use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message;

pub struct WebSocket {
    tx: mpsc::Sender<String>,
}

impl WebSocket {
    pub fn new(tx: mpsc::Sender<String>) -> Self {
        Self { tx }
    }

    pub async fn handle(&self, mut stream: impl StreamExt<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin) {
        while let Some(msg) = stream.next().await {
            if let Ok(Message::Text(text)) = msg {
                tracing::debug!("WebSocket received: {}", text);
            }
        }
    }

    pub async fn send(&self, msg: &str) -> anyhow::Result<()> {
        self.tx.send(msg.to_string()).await?;
        Ok(())
    }
}
