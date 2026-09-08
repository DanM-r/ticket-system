pub mod auth;
pub mod health;

use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use crate::state::AppState;

/// Construye el router principal de la API montando todas las rutas
/// disponibles hasta el momento (DESIGN.md 3.2/3.8).
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .with_state(state)
}
