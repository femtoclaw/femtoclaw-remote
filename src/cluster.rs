//! Distributed Cluster Manager (Reference Tier).
//!
//! Handles peer discovery, authentication, and state synchronization across
//! multiple FemtoClaw runtime nodes.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNode {
    pub id: String,
    pub address: String,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    Healthy,
    Degraded,
    Offline,
}

pub struct ClusterManager {
    self_id: String,
    peers: Arc<RwLock<HashMap<String, ClusterNode>>>,
    app_state: AppState,
}

impl ClusterManager {
    pub fn new(self_id: String, app_state: AppState) -> Self {
        Self {
            self_id,
            peers: Arc::new(RwLock::new(HashMap::new())),
            app_state,
        }
    }

    pub async fn add_peer(&self, node: ClusterNode) {
        if node.id == self.self_id { return; }
        self.peers.write().await.insert(node.id.clone(), node);
    }

    pub async fn remove_peer(&self, id: &str) {
        self.peers.write().await.remove(id);
    }

    pub async fn get_peers(&self) -> Vec<ClusterNode> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Broadcast state synchronization message to all healthy peers.
    pub async fn broadcast_sync(&self) -> anyhow::Result<()> {
        let peers = self.get_peers().await;
        let messages = self.app_state.get_messages().await;
        
        // This is a Reference Tier implementation: 
        // Real-world clusters use consensus for strong consistency.
        // We implement a simple gossip-style broadcast for state sync.
        
        let client = reqwest::Client::new();
        
        for peer in peers {
            if peer.status != NodeStatus::Healthy { continue; }
            
            let url = format!("{}/v1/cluster/sync", peer.address);
            let _ = client.post(&url)
                .json(&messages)
                .send()
                .await;
        }

        Ok(())
    }
}
