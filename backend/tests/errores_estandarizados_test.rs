use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use axum::Router;
use backend::routes::build_router;
use backend::state::AppState;
use serde_json::{json, Value};
use tower::ServiceExt;

// Tests de integración que verifican que TODAS las respuestas de error de la
// API siguen el formato estándar `{ "error": { "code", "message" } }`
// (DESIGN.md 3.8/3.9), incluyendo los rechazos genéricos que produciría Axum
// por defecto (JSON malformado, query params inválidos, rutas inexistentes)
// si no se hubieran interceptado explícitamente (T7).

fn app() -> Router {
    build_router(Arc::new(AppState::new()))
}

async fn body_json(response: Response) -> Value {
    let bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    serde_json::from_slice(&bytes).unwrap()
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

fn assert_error_estandar(json: &Value, code_esperado: &str) {
    assert!(json["error"].is_object(), "debe existir error: {json}");
    assert_eq!(json["error"]["code"], code_esperado);
    assert!(json["error"]["message"].is_string());
}

#[tokio::test]
async fn login_con_json_malformado_devuelve_400_con_formato_estandar() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("{ esto no es json valido"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_error_estandar(&json, "datos_invalidos");
}

#[tokio::test]
async fn login_sin_content_type_json_devuelve_error_con_formato_estandar() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .body(Body::from(json!({ "nombre": "Ana", "rol": "it" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert!(response.status().is_client_error());
    let json = body_json(response).await;
    assert_error_estandar(&json, "datos_invalidos");
}

#[tokio::test]
async fn listar_con_query_malformado_devuelve_400_con_formato_estandar() {
    let app = app();
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    // `estado` es un `Option<String>` en el struct de query, así que
    // cualquier valor de texto sería válido para axum a nivel de tipos; el
    // 400 real de "estado inválido" lo produce la validación del dominio
    // (ya cubierta en peticiones_test.rs). Este test verifica en cambio que
    // repetir un query param no soportado por serde_urlencoded/serde_qs
    // (múltiples valores para la misma clave) también responde con el
    // formato estándar en vez del rechazo genérico de Axum.
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/peticiones?estado=pendiente&estado=aprobada")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_error_estandar(&json, "datos_invalidos");
}

#[tokio::test]
async fn generar_con_cantidad_no_numerica_devuelve_400_con_formato_estandar() {
    let app = app();
    let token = login_y_obtener_token(&app, "Ana", "it").await;

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/peticiones/generar?cantidad=abc")
                .header("authorization", format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let json = body_json(response).await;
    assert_error_estandar(&json, "datos_invalidos");
}

#[tokio::test]
async fn ruta_inexistente_devuelve_404_con_formato_estandar() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/no-existe")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let json = body_json(response).await;
    assert_error_estandar(&json, "ruta_no_encontrada");
}

#[tokio::test]
async fn metodo_no_soportado_sobre_ruta_existente_devuelve_405_con_formato_estandar() {
    // Caso reportado en revisión de T7 (PR #8): `DELETE /api/peticiones` es
    // un método no soportado sobre un path que sí existe. Axum resuelve
    // este caso dentro del `MethodRouter` de la ruta, antes de llegar al
    // `fallback` del router (que solo se activa si ningún path coincide),
    // así que sin la capa dedicada devolvería el 405 vacío por defecto de
    // Axum en vez del formato estándar de error.
    let response = app()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/peticiones")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    let json = body_json(response).await;
    assert_error_estandar(&json, "metodo_no_permitido");
}

#[tokio::test]
async fn no_autenticado_sigue_el_formato_estandar_como_referencia() {
    let response = app()
        .oneshot(
            Request::builder()
                .method("GET")
                .uri("/api/peticiones")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let json = body_json(response).await;
    assert_error_estandar(&json, "no_autenticado");
}
