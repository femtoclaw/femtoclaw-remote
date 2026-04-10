//! Application State.

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;

use femtoclaw::Agent;
use femtoclaw::config::Config as CoreConfig;

#[derive(Clone)]
pub struct AppState {
    pub messages: Arc<RwLock<VecDeque<Message>>>,
    pub config: Arc<RwLock<Config>>,
    pub agent: Option<Arc<Agent>>,
}

#[derive(Debug, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub brain: String,
    pub max_history: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            brain: "echo".to_string(),
            max_history: 100,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::new())),
            config: Arc::new(RwLock::new(Config::default())),
            agent: None,
        }
    }

    pub async fn init_agent(&mut self) -> anyhow::Result<()> {
        let core_config = CoreConfig::default();
        let agent = Agent::new(core_config)?;
        self.agent = Some(Arc::new(agent));
        Ok(())
    }

    pub fn get_agent(&self) -> Option<Arc<Agent>> {
        self.agent.clone()
    }

    pub async fn add_message(&self, message: Message) {
        let mut messages = self.messages.write().await;
        let config = self.config.read().await;
        
        if messages.len() >= config.max_history {
            messages.pop_front();
        }
        messages.push_back(message);
    }

    pub async fn get_messages(&self) -> Vec<Message> {
        let messages = self.messages.read().await;
        messages.iter().cloned().collect()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
