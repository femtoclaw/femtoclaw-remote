//! FemtoClaw Remote Protocol Library.
//!
//! Provides API server, event handlers, and WebSocket support for remote
//! FemtoClaw runtime access.

pub mod handler;
pub mod server;
pub mod state;
pub mod types;
pub mod websocket;

pub use handler::Handler;
pub use server::Server;
pub use state::AppState;
