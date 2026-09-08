pub mod auth;
pub mod health;
pub mod peticiones;

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
        .route("/api/peticiones", get(peticiones::listar))
        .route("/api/peticiones/:id", get(peticiones::detalle))
        .route("/api/peticiones/:id/aprobar", post(peticiones::aprobar))
        .route("/api/peticiones/:id/denegar", post(peticiones::denegar))
        .with_state(state)
}
