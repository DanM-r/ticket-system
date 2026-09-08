use std::sync::Arc;

use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

use crate::domain::auth_store;
use crate::errors::AppError;
use crate::models::{LoginRequest, LoginResponse, Rol};
use crate::state::AppState;

/// Longitud máxima permitida para el campo `nombre` del login (DESIGN.md 3.8).
const NOMBRE_MAX_LEN: usize = 100;

/// Extrae el token del header `Authorization: Bearer <token>`, si está
/// presente y bien formado. Devuelve `None` en cualquier otro caso (header
/// ausente, esquema distinto de `Bearer`, valor no-UTF8, etc.).
fn extraer_bearer_token(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)
        .and_then(|valor| valor.to_str().ok())
        .and_then(|valor| valor.strip_prefix("Bearer "))
}

/// `POST /api/auth/login` — público (DESIGN.md 3.6/3.8).
///
/// Valida `nombre` (no vacío, máx. 100 caracteres) y `rol` (enum cerrado
/// `it` | `administracion`), crea una `Sesion` en el store y devuelve un
/// token opaco.
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AppError> {
    let nombre = payload.nombre.trim();

    if nombre.is_empty() {
        return Err(AppError::DatosInvalidos(
            "El nombre no puede estar vacío.".to_string(),
        ));
    }
    if nombre.chars().count() > NOMBRE_MAX_LEN {
        return Err(AppError::DatosInvalidos(format!(
            "El nombre no puede tener más de {NOMBRE_MAX_LEN} caracteres."
        )));
    }

    let rol: Rol = payload.rol.parse().map_err(|_| {
        AppError::DatosInvalidos(format!(
            "Rol inválido: '{}'. Debe ser 'it' o 'administracion'.",
            payload.rol
        ))
    })?;

    let sesion = auth_store::crear_sesion(&state, nombre.to_string(), rol);

    let respuesta = LoginResponse {
        token: sesion.token,
        nombre: sesion.nombre,
        rol: sesion.rol,
    };

    Ok((StatusCode::CREATED, Json(respuesta)))
}

/// `POST /api/auth/logout` — elimina la sesión asociada al token del header
/// `Authorization`, si existe (DESIGN.md 3.8). Siempre responde `204`, sea
/// que el token exista, no exista, o no se envíe ningún header — la
/// operación es idempotente.
pub async fn logout(State(state): State<Arc<AppState>>, headers: HeaderMap) -> StatusCode {
    if let Some(token) = extraer_bearer_token(&headers) {
        auth_store::eliminar_sesion(&state, token);
    }

    StatusCode::NO_CONTENT
}
