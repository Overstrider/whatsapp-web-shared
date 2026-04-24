//! whatsapp-web-shared-backend — library crate facade.
//!
//! Re-exports the pieces that integration tests + `main.rs` need to assemble
//! the HTTP/WS stack. Binary entry point is `src/main.rs`.

#![forbid(unsafe_code)]

pub mod config;
pub mod error;
pub mod state;

pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod routes;

pub use error::{AppError, AuthReason};
pub use state::AppState;

/// Build the fully-wired HTTP + WS router from a constructed [`AppState`].
///
/// Kept thin so integration tests can mount the router onto an in-memory
/// sqlite + stub WA client without touching `main.rs`.
pub fn build_app(state: AppState) -> axum::Router {
    routes::build_router(state)
}
