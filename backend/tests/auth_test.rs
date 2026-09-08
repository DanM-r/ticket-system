use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use backend::routes::build_router;
use backend::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

/// Construye un router de pruebas con un `AppState` limpio (sin peticiones
/// ni sesiones sembradas).
fn app() -> Router {
    build_router(Arc::new(AppState::new()))
}

async fn body_json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn login_request(nombre: &str, rol: &str) -> Request<Body> {
    Request::builder()
        .method("POST")
        .uri("/api/auth/login")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "nombre": nombre, "rol": rol }).to_string()))
        .unwrap()
}

#[tokio::test]
async fn login_con_datos_validos_devuelve_201_con_token() {
    let response = app().oneshot(login_request("Ana Pérez", "it")).await.unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let json = body_json(response).await;
    assert!(!json["token"].as_str().unwrap().is_empty());
    assert_eq!(json["nombre"], "Ana Pérez");
    assert_eq!(json["rol"], "it");
}

#[tokio::test]
async fn login_con_rol_administracion_tambien_funciona() {
    let response = app()
        .oneshot(login_request("Carlos Ruiz", "administracion"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = body_json(response).await;
    assert_eq!(json["rol"], "administracion");
}

#[tokio::test]
async fn login_con_nombre_vacio_devuelve_400_datos_invalidos() {
    let response = app().oneshot(login_request("", "it")).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}

#[tokio::test]
async fn login_con_nombre_solo_espacios_devuelve_400() {
    let response = app().oneshot(login_request("   ", "it")).await.unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn login_con_nombre_muy_largo_devuelve_400() {
    let nombre_largo = "a".repeat(101);
    let response = app()
        .oneshot(login_request(&nombre_largo, "it"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}

#[tokio::test]
async fn login_con_rol_desconocido_devuelve_400_datos_invalidos() {
    let response = app()
        .oneshot(login_request("Ana", "gerente"))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}

#[tokio::test]
async fn logout_con_token_existente_elimina_la_sesion() {
    let state = Arc::new(AppState::new());
    let app = build_router(state.clone());

    let login_response = app
        .clone()
        .oneshot(login_request("Ana", "it"))
        .await
        .unwrap();
    let login_json = body_json(login_response).await;
    let token = login_json["token"].as_str().unwrap().to_string();

    assert!(state.sesiones.read().unwrap().contains_key(&token));

    let logout_response = app
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
    assert!(!state.sesiones.read().unwrap().contains_key(&token));
}

#[tokio::test]
async fn logout_sin_header_authorization_devuelve_204() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}

#[tokio::test]
async fn logout_con_token_inexistente_devuelve_204_idempotente() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/logout")
                .header("authorization", "Bearer token-que-no-existe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);
}
