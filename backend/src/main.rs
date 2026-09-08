mod domain;
mod models;
mod state;

use axum::{routing::get, Json, Router};
use serde_json::{json, Value};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use state::AppState;

/// Cantidad de peticiones simuladas con las que se siembra el `AppState`
/// al arrancar el servidor (DESIGN.md 3.4/3.5), para que el portal tenga
/// datos desde el primer `cargo run`.
const CANTIDAD_SEED_INICIAL: usize = 15;

/// `GET /api/health` — endpoint público de verificación de salud del servicio.
async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

fn build_router() -> Router {
    Router::new().route("/api/health", get(health))
}

/// Lee el puerto en el que debe escuchar el servidor desde la variable de
/// entorno `PORT`, con `8080` como valor por defecto si no está definida o
/// no es un número válido.
fn read_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = read_port();

    let state = Arc::new(AppState::new());
    let creadas = state.seed_peticiones(CANTIDAD_SEED_INICIAL);
    tracing::info!(
        cantidad = creadas,
        "AppState sembrado con peticiones simuladas"
    );

    let app = build_router();

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Servidor escuchando en http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("no se pudo enlazar el puerto del servidor");
    axum::serve(listener, app)
        .await
        .expect("el servidor finalizó inesperadamente");
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_devuelve_ok() {
        let app = build_router();

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        assert_eq!(json, json!({ "status": "ok" }));
    }
}
