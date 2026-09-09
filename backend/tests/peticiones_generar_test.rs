use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use backend::routes::build_router;
use backend::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

// Tests de integración de `POST /api/peticiones/generar` (DESIGN.md 3.8, T7),
// protegido por el extractor `AuthSession` (T4) y que reutiliza el generador
// de datos simulados de T2.

async fn body_json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn post_request(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("POST").uri(uri);
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

fn app_con_peticiones(cantidad: usize) -> (Arc<AppState>, Router) {
    let state = Arc::new(AppState::new());
    state.seed_peticiones(cantidad);
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn generar_sin_sesion_devuelve_401() {
    let (_state, app) = app_con_peticiones(0);

    let response = app
        .oneshot(post_request("/api/peticiones/generar", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn generar_sin_cantidad_crea_cinco_por_defecto() {
    let (state, app) = app_con_peticiones(0);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request("/api/peticiones/generar", Some(&token)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = body_json(response).await;
    assert_eq!(json["creadas"], 5);
    assert_eq!(state.peticiones.read().unwrap().len(), 5);
}

#[tokio::test]
async fn generar_con_cantidad_explicita_crea_esa_cantidad() {
    let (state, app) = app_con_peticiones(0);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=12",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    let json = body_json(response).await;
    assert_eq!(json["creadas"], 12);
    assert_eq!(state.peticiones.read().unwrap().len(), 12);
}

#[tokio::test]
async fn generar_es_acumulativo_sobre_peticiones_existentes() {
    let (state, app) = app_con_peticiones(3);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=4",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(state.peticiones.read().unwrap().len(), 7);
}

#[tokio::test]
async fn generar_con_cantidad_en_los_limites_1_y_50_funciona() {
    let (state, app) = app_con_peticiones(0);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .clone()
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=1",
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = app
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=50",
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);
    assert_eq!(state.peticiones.read().unwrap().len(), 51);
}

#[tokio::test]
async fn generar_con_cantidad_cero_devuelve_400_y_no_crea_peticiones() {
    let (state, app) = app_con_peticiones(0);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=0",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
    assert!(state.peticiones.read().unwrap().is_empty());
}

#[tokio::test]
async fn generar_con_cantidad_51_devuelve_400_y_no_crea_peticiones() {
    let (state, app) = app_con_peticiones(0);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=51",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
    assert!(state.peticiones.read().unwrap().is_empty());
}

#[tokio::test]
async fn generar_con_cantidad_no_numerica_devuelve_400_datos_invalidos() {
    let (state, app) = app_con_peticiones(0);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request(
            "/api/peticiones/generar?cantidad=no-es-un-numero",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
    assert!(state.peticiones.read().unwrap().is_empty());
}
