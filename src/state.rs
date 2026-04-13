//! Application State.

use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::VecDeque;
use serde::{Deserialize, Serialize};

use femtoclaw::Agent;
use femtoclaw::config::Config as CoreConfig;
use crate::cluster::ClusterManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Clone)]
pub struct AppState {
    pub self_id: String,
    pub messages: Arc<RwLock<VecDeque<Message>>>,
    pub config: Arc<RwLock<Config>>,
    pub agent: Arc<RwLock<Option<Arc<Agent>>>>,
    pub cluster: Arc<RwLock<Option<ClusterManager>>>,
}

impl AppState {
    pub fn new(self_id: String) -> Self {
        Self {
            self_id,
            messages: Arc::new(RwLock::new(VecDeque::new())),
            config: Arc::new(RwLock::new(Config::default())),
            agent: Arc::new(RwLock::new(None)),
            cluster: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn init_agent(&self) -> anyhow::Result<()> {
        let mut core_config = CoreConfig::default();
        core_config.max_iterations = 10;
        let agent = Agent::new(core_config)?;
        let mut agent_lock = self.agent.write().await;
        *agent_lock = Some(Arc::new(agent));
        Ok(())
    }

    pub async fn init_cluster(&self) {
        let mut cluster_lock = self.cluster.write().await;
        *cluster_lock = Some(ClusterManager::new(self.self_id.clone(), self.clone()));
    }

    pub async fn get_agent(&self) -> Option<Arc<Agent>> {
        self.agent.read().await.clone()
    }

    pub async fn add_message(&self, message: Message) {
        let mut messages = self.messages.write().await;
        let config = self.config.read().await;
        
        if messages.len() >= config.max_history {
            messages.pop_front();
        }
        messages.push_back(message.clone());
        
        // Broadcast sync to cluster
        let cluster_lock = self.cluster.read().await;
        if let Some(cluster) = &*cluster_lock {
            let _ = cluster.broadcast_sync().await;
        }
    }

    pub async fn get_messages(&self) -> Vec<Message> {
        let messages = self.messages.read().await;
        messages.iter().cloned().collect()
    }

    pub async fn sync_from_remote(&self, remote_messages: Vec<Message>) {
        let mut messages = self.messages.write().await;
        messages.clear();
        for msg in remote_messages {
            messages.push_back(msg);
        }
        
        // Update local agent if it exists
        let agent_lock = self.agent.read().await;
        if let Some(agent) = &*agent_lock {
            let core_messages: Vec<femtoclaw::Message> = messages.iter().map(|m| {
                match m.role.as_str() {
                    "user" => femtoclaw::Message::user(&m.content),
                    "assistant" => femtoclaw::Message::assistant(&m.content),
                    "system" => femtoclaw::Message::system(&m.content),
                    "tool" => femtoclaw::Message::tool(&m.content),
                    _ => femtoclaw::Message::assistant(&m.content),
                }
            }).collect();
            agent.sync_memory(&core_messages).await;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new("local-node".to_string())
    }
}
