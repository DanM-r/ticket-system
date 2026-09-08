use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use backend::routes::build_router;
use backend::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

// Tests de integración de `POST /api/peticiones/:id/aprobar` y
// `POST /api/peticiones/:id/denegar` (DESIGN.md 3.8, T6), protegidos por el
// extractor `AuthSession` (T4).

async fn body_json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
}

fn post_request(uri: &str, token: Option<&str>, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method("POST").uri(uri);
    if let Some(token) = token {
        builder = builder.header("authorization", format!("Bearer {token}"));
    }
    match body {
        Some(body) => builder
            .header("content-type", "application/json")
            .body(Body::from(body.to_string()))
            .unwrap(),
        None => builder.body(Body::empty()).unwrap(),
    }
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

/// Construye un estado con `cantidad` peticiones sembradas (todas
/// `pendiente`) y el router correspondiente.
fn app_con_peticiones(cantidad: usize) -> (Arc<AppState>, Router) {
    let state = Arc::new(AppState::new());
    state.seed_peticiones(cantidad);
    let app = build_router(state.clone());
    (state, app)
}

fn primer_id(state: &AppState) -> uuid::Uuid {
    state
        .peticiones
        .read()
        .unwrap()
        .keys()
        .next()
        .copied()
        .unwrap()
}

#[tokio::test]
async fn aprobar_sin_sesion_devuelve_401() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);

    let response = app
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/aprobar"),
            None,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn denegar_sin_sesion_devuelve_401() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);

    let response = app
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/denegar"),
            None,
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "no_autenticado");
}

#[tokio::test]
async fn aprobar_pendiente_con_sesion_valida_devuelve_200_y_actualiza_estado_y_decision() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);
    let token = login_y_obtener_token(&app, "Ana Pérez", "it").await;

    let response = app
        .clone()
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/aprobar"),
            Some(&token),
            Some(json!({ "comentario": "Cumple política." })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["estado"], "aprobada");
    assert_eq!(json["decision"]["decidido_por"], "Ana Pérez");
    assert_eq!(json["decision"]["rol_decisor"], "it");
    assert_eq!(json["decision"]["comentario"], "Cumple política.");
    assert!(json["decision"]["fecha_decision"].is_string());
}

#[tokio::test]
async fn denegar_pendiente_con_sesion_valida_devuelve_200_y_actualiza_estado_y_decision() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);
    let token = login_y_obtener_token(&app, "Carlos Ruiz", "administracion").await;

    let response = app
        .clone()
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/denegar"),
            Some(&token),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["estado"], "denegada");
    assert_eq!(json["decision"]["decidido_por"], "Carlos Ruiz");
    assert_eq!(json["decision"]["rol_decisor"], "administracion");
    assert!(json["decision"]["comentario"].is_null());
}

/// Caso explícito requerido por T6: el body no puede falsificar el decisor.
/// Aunque el cliente envíe `decidido_por`/`rol` falsos, deben ignorarse por
/// completo y usarse siempre los datos de la sesión.
#[tokio::test]
async fn aprobar_ignora_decidido_por_y_rol_falsos_enviados_en_el_body() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);
    let token = login_y_obtener_token(&app, "Ana Pérez", "it").await;

    let response = app
        .clone()
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/aprobar"),
            Some(&token),
            Some(json!({
                "comentario": "ok",
                "decidido_por": "Impostor",
                "rol": "administracion",
                "rol_decisor": "administracion"
            })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let json = body_json(response).await;
    assert_eq!(json["decision"]["decidido_por"], "Ana Pérez");
    assert_eq!(json["decision"]["rol_decisor"], "it");
}

#[tokio::test]
async fn aprobar_peticion_ya_decidida_devuelve_409_y_no_modifica_la_decision() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);
    let token = login_y_obtener_token(&app, "Ana Pérez", "it").await;

    let primera = app
        .clone()
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/aprobar"),
            Some(&token),
            None,
        ))
        .await
        .unwrap();
    assert_eq!(primera.status(), StatusCode::OK);

    let token_otro = login_y_obtener_token(&app, "Carlos", "administracion").await;
    let segunda = app
        .clone()
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/denegar"),
            Some(&token_otro),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(segunda.status(), StatusCode::CONFLICT);
    let json = body_json(segunda).await;
    assert_eq!(json["error"]["code"], "peticion_ya_decidida");

    // La decisión original no se modificó.
    let detalle = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(format!("/api/peticiones/{id}"))
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    let detalle_json = body_json(detalle).await;
    assert_eq!(detalle_json["estado"], "aprobada");
    assert_eq!(detalle_json["decision"]["decidido_por"], "Ana Pérez");
}

#[tokio::test]
async fn comentario_de_mas_de_500_caracteres_devuelve_400() {
    let (state, app) = app_con_peticiones(1);
    let id = primer_id(&state);
    let token = login_y_obtener_token(&app, "Ana", "it").await;
    let comentario_largo = "a".repeat(501);

    let response = app
        .oneshot(post_request(
            &format!("/api/peticiones/{id}/aprobar"),
            Some(&token),
            Some(json!({ "comentario": comentario_largo })),
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}

#[tokio::test]
async fn aprobar_con_id_mal_formado_devuelve_400() {
    let (_state, app) = app_con_peticiones(1);
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(post_request(
            "/api/peticiones/no-es-un-uuid/aprobar",
            Some(&token),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "datos_invalidos");
}

#[tokio::test]
async fn denegar_con_id_inexistente_devuelve_404() {
    let (_state, app) = app_con_peticiones(1);
    let token = login_y_obtener_token(&app, "Ana", "it").await;
    let id_inexistente = uuid::Uuid::new_v4();

    let response = app
        .oneshot(post_request(
            &format!("/api/peticiones/{id_inexistente}/denegar"),
            Some(&token),
            None,
        ))
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = body_json(response).await;
    assert_eq!(json["error"]["code"], "peticion_no_encontrada");
}
