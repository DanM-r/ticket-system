use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use backend::routes::build_router;
use backend::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

// Tests de integración del extractor `AuthSession` (DESIGN.md 3.6, T4),
// ejercitado a través del endpoint de demostración `GET /api/auth/me`.

fn app() -> Router {
    build_router(Arc::new(AppState::new()))
}

async fn body_json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn me_request(token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri("/api/auth/me");
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    builder.body(Body::empty()).unwrap()
}

async fn login_y_obtener_token(app: &Router, nombre: &str, rol: &str) -> String {
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(json!({ "nombre": nombre, "rol": rol }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    let json = body_json(response).await;
    json["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn ruta_protegida_sin_header_authorization_devuelve_401() {
    let response = app().oneshot(me_request(None)).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn ruta_protegida_con_token_inexistente_devuelve_401() {
    let response = app()
        .oneshot(me_request(Some("token-que-no-existe")))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn ruta_protegida_con_esquema_distinto_de_bearer_devuelve_401() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/auth/me")
                .header("authorization", "Basic algo")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn ruta_protegida_con_token_valido_expone_nombre_y_rol_de_la_sesion() {
    let app = app();
    let token = login_y_obtener_token(&app, "Ana Pérez", "it").await;

    let response = app.oneshot(me_request(Some(&token))).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["nombre"], "Ana Pérez");
    assert_eq!(json["rol"], "it");
}

#[tokio::test]
async fn ruta_protegida_con_token_valido_de_administracion_expone_el_rol_correcto() {
    let app = app();
    let token = login_y_obtener_token(&app, "Carlos Ruiz", "administracion").await;

    let response = app.oneshot(me_request(Some(&token))).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["nombre"], "Carlos Ruiz");
    assert_eq!(json["rol"], "administracion");
}

#[tokio::test]
async fn token_invalidado_por_logout_deja_de_dar_acceso() {
    let app = app();
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let logout_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(logout_response.status(), StatusCode::NO_CONTENT);

    let response = app.oneshot(me_request(Some(&token))).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}
