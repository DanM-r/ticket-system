use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use backend::routes::build_router;
use backend::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

// Tests de integración de `GET /api/peticiones` y `GET /api/peticiones/:id`
// (DESIGN.md 3.8, T5), ambos protegidos por el extractor `AuthSession` (T4).

async fn body_json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn get_request(uri: &str, token: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method("GET").uri(uri);
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

/// Construye un estado con `cantidad` peticiones sembradas y el router
/// correspondiente, devolviendo ambos para poder inspeccionar el store
/// directamente en los tests (ej. tomar un id real para el detalle).
fn app_con_peticiones(cantidad: usize) -> (Arc<AppState>, Router) {
    let state = Arc::new(AppState::new());
    state.seed_peticiones(cantidad);
    let app = build_router(state.clone());
    (state, app)
}

#[tokio::test]
async fn listar_sin_sesion_devuelve_401() {
    let (_state, app) = app_con_peticiones(5);

    let response = app
        .oneshot(get_request("/api/peticiones", None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn detalle_sin_sesion_devuelve_401() {
    let (state, app) = app_con_peticiones(1);
    let id = state
        .peticiones
        .read()
        .unwrap()
        .keys()
        .next()
        .copied()
        .unwrap();

    let response = app
        .oneshot(get_request(&format!("/api/peticiones/{id}"), None))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn listar_con_sesion_valida_devuelve_el_array_completo() {
    let (_state, app) = app_con_peticiones(15);
    let token = login_y_obtener_token(&app, "Ana Pérez", "it").await;

    let response = app
        .oneshot(get_request("/api/peticiones", Some(&token)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    let peticiones = json["peticiones"].as_array().unwrap();
    assert_eq!(peticiones.len(), 15);

    let primera = &peticiones[0];
    assert!(primera["id"].is_string());
    assert!(primera["tipo_peticion"].is_string());
    assert!(primera["usuario"].is_string());
    assert!(primera["email"].is_string());
    assert!(primera["fecha_creacion"].is_string());
    assert!(primera["cuerpo_mensaje"].is_string());
    assert!(primera["severidad"].is_string());
    assert_eq!(primera["estado"], "pendiente");
    assert!(primera["decision"].is_null());
}

#[tokio::test]
async fn listar_con_filtro_pendiente_devuelve_todas_las_sembradas() {
    let (_state, app) = app_con_peticiones(10);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(get_request(
            "/api/peticiones?estado=pendiente",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["peticiones"].as_array().unwrap().len(), 10);
}

#[tokio::test]
async fn listar_con_filtro_aprobada_no_devuelve_pendientes() {
    let (_state, app) = app_con_peticiones(10);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(get_request(
            "/api/peticiones?estado=aprobada",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert!(json["peticiones"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn listar_con_filtro_denegada_no_devuelve_pendientes() {
    let (_state, app) = app_con_peticiones(10);
    let token = login_y_obtener_token(&app, "Carlos", "administracion").await;

    let response = app
        .oneshot(get_request(
            "/api/peticiones?estado=denegada",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert!(json["peticiones"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn listar_con_estado_invalido_devuelve_400() {
    let (_state, app) = app_con_peticiones(5);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(get_request("/api/peticiones?estado=cancelada", Some(&token)))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}

#[tokio::test]
async fn detalle_con_id_existente_devuelve_200_y_el_objeto_completo() {
    let (state, app) = app_con_peticiones(3);
    let token = login_y_obtener_token(&app, "Ana", "it").await;
    let id = state
        .peticiones
        .read()
        .unwrap()
        .keys()
        .next()
        .copied()
        .unwrap();

    let response = app
        .oneshot(get_request(
            &format!("/api/peticiones/{id}"),
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["id"], id.to_string());
    assert_eq!(json["estado"], "pendiente");
    assert!(json["decision"].is_null());
}

#[tokio::test]
async fn detalle_con_id_inexistente_devuelve_404() {
    let (_state, app) = app_con_peticiones(3);
    let token = login_y_obtener_token(&app, "Ana", "it").await;
    let id_inexistente = uuid::Uuid::new_v4();

    let response = app
        .oneshot(get_request(
            &format!("/api/peticiones/{id_inexistente}"),
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "peticion_no_encontrada");
}

#[tokio::test]
async fn detalle_con_id_mal_formado_devuelve_400() {
    let (_state, app) = app_con_peticiones(3);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(get_request(
            "/api/peticiones/no-es-un-uuid",
            Some(&token),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}
