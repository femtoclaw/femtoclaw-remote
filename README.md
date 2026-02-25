# FemtoClaw Remote

[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)
[![Status](https://img.shields.io/badge/Status-Stable-green.svg)]()

FemtoClaw Remote Protocol — API server, event handlers, and WebSocket support.

## Overview

`femtoclaw-remote` provides the network layer for remote access to FemtoClaw runtime. It exposes the FemtoClaw agent via HTTP and WebSocket APIs.

This enables distributed deployments and integration with web applications.

## Features

- **HTTP API Server**: RESTful endpoints for agent interaction
- **WebSocket Support**: Real-time bidirectional communication
- **Request Handling**: Structured request/response handling
- **CORS Support**: Cross-origin resource sharing
- **Tracing**: Request tracing and logging

## Installation

```toml
[dependencies]
femtoclaw-remote = "1.0"
```

## Usage

```rust
use femtoclaw_remote::Server;

let server = Server::new(8080);
server.run().await?;
```

## API Endpoints

### REST API

```
GET  /              — Health check
POST /v1/chat       — Send chat message
POST /v1/tools/execute — Execute capability
```

### WebSocket

```
WS  /v1/ws         — Real-time bidirectional messaging
```

## Message Format

### Chat Request

```json
{
  "messages": [
    { "role": "user", "content": "Hello" }
  ]
}
```

### Chat Response

```json
{
  "message": { "content": "Hello! How can I help?" },
  "tool_call": null
}
```

### Tool Execution Request

```json
{
  "tool": "filesystem.read",
  "args": { "path": "/etc/hosts" }
}
```

### Tool Execution Response

```json
{
  "result": "127.0.0.1 localhost"
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    External Clients                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │   Web App  │  │   CLI       │  │   SDK       │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
└─────────┼────────────────┼────────────────┼─────────────────┘
          │                │                │
          ▼                ▼                ▼
┌─────────────────────────────────────────────────────────────┐
│              femtoclaw-remote                               │
│  ┌────────────┐  ┌────────────┐  ┌────────────────┐        │
│  │   Axum    │  │  WebSocket │  │    Handler     │        │
│  │  Server   │  │   Handler  │  │                │        │
│  └─────┬─────┘  └─────┬──────┘  └───────┬────────┘        │
└────────┼──────────────┼─────────────────┼──────────────────┘
         │              │                 │
         ▼              ▼                 ▼
┌─────────────────────────────────────────────────────────────┐
│              femtoclaw-core                                 │
│  femtoclaw-protocol → femtoclaw-policy → femtoclaw         │
└─────────────────────────────────────────────────────────────┘
```

## Dependencies

- femtoclaw
- femtoclaw-protocol
- femtoclaw-policy
- femtoclaw-audit
- axum 0.8
- tokio 1.x (with rt-multi-thread, macros, net, sync, time)
- tokio-tungstenite 0.24
- futures-util 0.3
- tower 0.5
- tower-http 0.6 (with cors, trace)
- serde 1.x
- serde_json 1.x
- tracing 0.1

## Related Crates

| Crate | Purpose |
|-------|---------|
| `femtoclaw-sdk` | Client library for remote access |
| `femtoclaw-cli` | Interactive CLI |

## Related Specifications

- [FC-DEPLOY-0001: Deployment Specification](../femtoclaw-spec/FC-DEPLOY-0001-FemtoClaw_Deployment_and_Operational_Environment_Specification.md)
- [FC-ABI-0001: Application Binary Interface](../femtoclaw-spec/FC-ABI-0001-Application_Binary_Interface_ABI_Specification.md)

## License

Copyright 2026 FemtoClaw

Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
