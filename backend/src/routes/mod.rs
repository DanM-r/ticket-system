pub mod auth;
pub mod health;
pub mod peticiones;

use std::sync::Arc;

use axum::extract::Request;
use axum::http::{HeaderValue, Method, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::Router;
use tower_http::cors::CorsLayer;

use crate::errors::AppError;
use crate::state::AppState;

/// Construye el router principal de la API montando todas las rutas
/// disponibles (DESIGN.md 3.2/3.8), sin CORS. Usado directamente por los
/// tests de integración (que no necesitan simular un origin de navegador) y
/// como base de [`build_router_with_cors`], que sí lo usa `main.rs`.
///
/// Cualquier ruta no montada cae en [`fallback`], que responde
/// `404 ruta_no_encontrada` con el mismo formato de error estándar que el
/// resto de la API, en vez del 404 vacío que produce Axum por defecto (T7).
///
/// Además, [`reescribir_405`] intercepta cualquier `405 Method Not Allowed`
/// (ej. `DELETE /api/peticiones`, un path existente con un método no
/// soportado) para que también siga el formato de error estándar: Axum
/// resuelve ese caso dentro del `MethodRouter` de la ruta, antes de que la
/// request pueda llegar a [`fallback`] (que solo se activa si ningún path
/// coincide), así que hace falta una capa aparte que reescriba la respuesta
/// ya generada (T7).
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/logout", post(auth::logout))
        .route("/api/auth/me", get(auth::me))
        .route("/api/peticiones", get(peticiones::listar))
        .route("/api/peticiones/generar", post(peticiones::generar))
        .route("/api/peticiones/:id", get(peticiones::detalle))
        .route("/api/peticiones/:id/aprobar", post(peticiones::aprobar))
        .route("/api/peticiones/:id/denegar", post(peticiones::denegar))
        .fallback(fallback)
        .layer(middleware::from_fn(reescribir_405))
        .with_state(state)
}

/// Handler de `fallback` para cualquier combinación de método/path que no
/// coincida con ninguna ruta montada (T7). Ver [`build_router`].
async fn fallback() -> AppError {
    AppError::RutaNoEncontrada
}

/// Middleware que reescribe cualquier respuesta `405 Method Not Allowed`
/// generada por el `MethodRouter` de una ruta existente (ej. `DELETE
/// /api/peticiones`) al formato de error estándar de la API, en vez del 405
/// vacío que produce Axum por defecto (T7). Ver [`build_router`].
async fn reescribir_405(request: Request, next: Next) -> Response {
    let response = next.run(request).await;

    if response.status() == StatusCode::METHOD_NOT_ALLOWED {
        return AppError::MetodoNoPermitido.into_response();
    }

    response
}

/// Construye la capa de CORS restringida al `origin` configurado vía
/// `CORS_ALLOWED_ORIGIN` (DESIGN.md 3.10). No se usa `Any`: el origin
/// permitido es siempre uno concreto, nunca un comodín, para no exponer la
/// API a cualquier sitio (DESIGN.md sección 8, checklist de seguridad).
///
/// Se permite el header `Authorization` (requerido por el esquema de
/// sesión por token opaco, DESIGN.md 3.6) y `Content-Type` (requerido para
/// enviar JSON en el body).
fn build_cors_layer(allowed_origin: &str) -> CorsLayer {
    let origin: HeaderValue = allowed_origin
        .parse()
        .expect("CORS_ALLOWED_ORIGIN debe ser un origin HTTP válido");

    CorsLayer::new()
        .allow_origin(origin)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
        ])
}

/// Construye el router principal de la API (ver [`build_router`]) con la
/// capa de CORS aplicada, restringida al `origin` recibido (DESIGN.md 3.10,
/// T7). Es la función que usa `main.rs` para levantar el servidor real.
pub fn build_router_with_cors(state: Arc<AppState>, cors_allowed_origin: &str) -> Router {
    build_router(state).layer(build_cors_layer(cors_allowed_origin))
}
