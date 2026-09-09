use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use backend::routes::build_router_with_cors;
use backend::state::AppState;
use tower::ServiceExt;

// Tests de integración de la configuración de CORS (DESIGN.md 3.10, T7):
// el origin configurado vía `CORS_ALLOWED_ORIGIN` debe poder consumir la API
// sin ser bloqueado, y un origin distinto no debe recibir el header
// `Access-Control-Allow-Origin` que lo habilitaría en el navegador.

const ORIGIN_PERMITIDO: &str = "http://localhost:5173";

fn app() -> axum::Router {
    build_router_with_cors(Arc::new(AppState::new()), ORIGIN_PERMITIDO)
}

#[tokio::test]
async fn preflight_desde_el_origin_permitido_no_es_bloqueado() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/peticiones")
                .header("origin", ORIGIN_PERMITIDO)
                .header("access-control-request-method", "GET")
                .header("access-control-request-headers", "authorization")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .expect("la respuesta preflight debe incluir Access-Control-Allow-Origin")
        .to_str()
        .unwrap();
    assert_eq!(allow_origin, ORIGIN_PERMITIDO);
}

#[tokio::test]
async fn respuesta_simple_desde_el_origin_permitido_incluye_el_header_cors() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/health")
                .header("origin", ORIGIN_PERMITIDO)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .expect("la respuesta debe incluir Access-Control-Allow-Origin")
        .to_str()
        .unwrap();
    assert_eq!(allow_origin, ORIGIN_PERMITIDO);
}

#[tokio::test]
async fn respuesta_a_un_origin_distinto_nunca_refleja_ese_origin() {
    // El origin permitido es un valor exacto y fijo (`CORS_ALLOWED_ORIGIN`),
    // no `*` ni un reflejo del header `Origin` de la request (DESIGN.md
    // 3.10, checklist de seguridad sección 8). El bloqueo real de CORS lo
    // aplica el navegador comparando su propio origin contra este header;
    // este test verifica que el servidor sigue devolviendo el origin
    // permitido configurado (nunca `http://sitio-no-autorizado.test`, ni
    // `*`), que es la señal que hace que el navegador rechace la respuesta
    // para un origin no autorizado.
    let response = app()
        .oneshot(
            Request::builder()
                .method("OPTIONS")
                .uri("/api/peticiones")
                .header("origin", "http://sitio-no-autorizado.test")
                .header("access-control-request-method", "GET")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let allow_origin = response
        .headers()
        .get("access-control-allow-origin")
        .map(|v| v.to_str().unwrap().to_string());

    assert_ne!(allow_origin.as_deref(), Some("*"));
    assert_ne!(allow_origin.as_deref(), Some("http://sitio-no-autorizado.test"));
    if let Some(allow_origin) = allow_origin {
        assert_eq!(allow_origin, ORIGIN_PERMITIDO);
    }
}
