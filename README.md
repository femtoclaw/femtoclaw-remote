# 🌐 FemtoClaw Remote & Clustering

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![Tier](https://img.shields.io/badge/Tier-Reference-blue.svg)]()

The **FemtoClaw Remote** library provides the network and coordination layer required for distributed agent clusters. It implements the **Distributed Runtime Model (Spec 41)**, enabling multi-node state synchronization, high availability, and remote API access.

---

## 📡 Distributed State Synchronization

FemtoClaw nodes in a cluster use a high-efficiency synchronization protocol to ensure that all execution history and short-term memory (STM) remain consistent across instances.

- **Gossip-style Sync**: Every execution event (ToolCall, Result, Message) is broadcast to all healthy peers in the cluster.
- **Remote Reconciliation**: Inbound synchronization events update the local agent's memory, allowing a node to resume a task started by a peer.
- **Contextual Consistency**: Ensures that "Brain" inference always has access to the most recent global execution history, regardless of which node executed the previous step.

---

## 🔌 Industrial API Endpoints

The remote server provides a standardized interface for interacting with autonomous agent loops.

### REST API
- `POST /v1/chat`: Send a prompt to the autonomous agent loop. The agent will iterate until a final response is generated or limits are reached.
- `POST /v1/tools/execute`: Direct, low-level execution of a system capability (authorized only).
- `POST /v1/cluster/sync`: Internal endpoint for inbound state updates from cluster peers.

### WebSocket
- `GET /v1/ws`: An interactive, bidirectional tunnel for real-time agent sessions. Ideal for terminal-based integrations and streaming responses.

---

## 🚀 Cluster Usage

```rust
use femtoclaw_remote::{Server, AppState};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize application state with a unique node ID
    let state = AppState::new("industrial-node-01".to_string());
    
    // 2. Initialize the agent and cluster management
    state.init_agent().await?;
    state.init_cluster().await;

    // 3. Launch the API server on the standard port
    let server = Server::new(8080);
    server.run(state).await?;

    Ok(())
}
```

---

## 📄 Related Specifications
- **[FC-DIST-0001: Distributed Runtime Model](../femtoclaw-spec/41-FemtoClaw_Distributed_Runtime_Model_Specification.md)**
- **[FC-DEPLOY-0001: Deployment Specification](../femtoclaw-spec/FC-DEPLOY-0001-FemtoClaw_Deployment_and_Operational_Environment_Specification.md)**

Copyright © 2026 FemtoClaw Project.
