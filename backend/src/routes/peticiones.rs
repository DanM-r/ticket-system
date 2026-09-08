use std::sync::Arc;

use axum::body::Bytes;
use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::peticion_store::{self, DecidirPeticionError};
use crate::errors::AppError;
use crate::middleware::AuthSession;
use crate::models::{EstadoPeticion, Peticion};
use crate::state::AppState;

/// Longitud máxima permitida para `comentario` al aprobar/denegar una
/// petición (DESIGN.md 3.8).
const COMENTARIO_MAX_LEN: usize = 500;

/// Query params aceptados por `GET /api/peticiones` (DESIGN.md 3.8).
#[derive(Debug, Deserialize)]
pub struct ListarPeticionesQuery {
    pub estado: Option<String>,
}

/// Respuesta de `GET /api/peticiones` (DESIGN.md 3.8).
#[derive(Debug, Serialize)]
pub struct ListarPeticionesResponse {
    pub peticiones: Vec<Peticion>,
}

/// `GET /api/peticiones` — requiere sesión (DESIGN.md 3.8, T5).
///
/// Devuelve el listado completo de peticiones del store, opcionalmente
/// filtrado por el query param `estado` (`pendiente` | `aprobada` |
/// `denegada`). Un valor de `estado` fuera de ese enum responde
/// `400 datos_invalidos`. La sesión (`_auth`) solo se exige por el
/// extractor `AuthSession` (T4); no se usa su contenido en este handler.
pub async fn listar(
    _auth: AuthSession,
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListarPeticionesQuery>,
) -> Result<Json<ListarPeticionesResponse>, AppError> {
    let estado = query
        .estado
        .map(|valor| {
            valor.parse::<EstadoPeticion>().map_err(|_| {
                AppError::DatosInvalidos(format!(
                    "Estado inválido: '{valor}'. Debe ser 'pendiente', 'aprobada' o 'denegada'."
                ))
            })
        })
        .transpose()?;

    let peticiones = peticion_store::listar_peticiones(&state, estado);

    Ok(Json(ListarPeticionesResponse { peticiones }))
}

/// `GET /api/peticiones/:id` — requiere sesión (DESIGN.md 3.8, T5).
///
/// - `id` no es un UUID válido → `400 datos_invalidos`.
/// - `id` válido pero no existe en el store → `404 peticion_no_encontrada`.
/// - Éxito → `200` con el objeto `Peticion` completo.
pub async fn detalle(
    _auth: AuthSession,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Peticion>, AppError> {
    let id_param = id;
    let id: Uuid = id_param.parse().map_err(|_| {
        AppError::DatosInvalidos(format!(
            "Id inválido: '{id_param}'. Debe ser un UUID."
        ))
    })?;

    let peticion =
        peticion_store::obtener_peticion(&state, id).ok_or(AppError::PeticionNoEncontrada)?;

    Ok(Json(peticion))
}

/// Body (opcional) de `POST /api/peticiones/:id/aprobar` y `.../denegar`
/// (DESIGN.md 3.8). Deliberadamente **no** incluye `decidido_por` ni `rol`:
/// aunque el cliente los envíe en el JSON, `serde` los ignora al no formar
/// parte de este struct, y el decisor real siempre se deriva de la sesión
/// (`AuthSession`, T4) — ver DESIGN.md 3.6.
#[derive(Debug, Deserialize, Default)]
pub struct DecidirPeticionRequest {
    pub comentario: Option<String>,
}

/// Parsea el body (opcional) de una request de aprobar/denegar. Un body
/// vacío se interpreta como "sin comentario"; si se envía un body no vacío,
/// debe ser JSON válido con la forma de [`DecidirPeticionRequest`].
fn parsear_body_decision(body: &Bytes) -> Result<DecidirPeticionRequest, AppError> {
    if body.is_empty() {
        return Ok(DecidirPeticionRequest::default());
    }

    serde_json::from_slice(body).map_err(|_| {
        AppError::DatosInvalidos("El cuerpo de la petición debe ser JSON válido.".to_string())
    })
}

/// Valida que, de estar presente, `comentario` no exceda
/// `COMENTARIO_MAX_LEN` caracteres (DESIGN.md 3.8/3.11).
fn validar_comentario(comentario: Option<String>) -> Result<Option<String>, AppError> {
    match comentario {
        Some(comentario) if comentario.chars().count() > COMENTARIO_MAX_LEN => {
            Err(AppError::DatosInvalidos(format!(
                "El comentario no puede tener más de {COMENTARIO_MAX_LEN} caracteres."
            )))
        }
        otro => Ok(otro),
    }
}

fn parsear_id_peticion(id_param: &str) -> Result<Uuid, AppError> {
    id_param
        .parse()
        .map_err(|_| AppError::DatosInvalidos(format!("Id inválido: '{id_param}'. Debe ser un UUID.")))
}

/// Lógica compartida por `aprobar` y `denegar` (DESIGN.md 3.8, T6):
/// - `id` mal formado → `400 datos_invalidos`.
/// - `comentario` de más de 500 caracteres → `400 datos_invalidos`.
/// - `id` bien formado pero inexistente → `404 peticion_no_encontrada`.
/// - Petición con `estado != pendiente` → `409 peticion_ya_decidida`.
/// - Éxito → `200` con la `Peticion` actualizada; `decision.decidido_por` y
///   `decision.rol_decisor` provienen siempre de `auth` (la sesión resuelta
///   por el extractor `AuthSession`), nunca del body.
async fn decidir(
    auth: AuthSession,
    state: Arc<AppState>,
    id_param: String,
    nuevo_estado: EstadoPeticion,
    body: Bytes,
) -> Result<Json<Peticion>, AppError> {
    let id = parsear_id_peticion(&id_param)?;
    let payload = parsear_body_decision(&body)?;
    let comentario = validar_comentario(payload.comentario)?;

    peticion_store::decidir_peticion(&state, id, nuevo_estado, auth.nombre, auth.rol, comentario)
        .map(Json)
        .map_err(|err| match err {
            DecidirPeticionError::NoEncontrada => AppError::PeticionNoEncontrada,
            DecidirPeticionError::YaDecidida => AppError::PeticionYaDecidida,
        })
}

/// `POST /api/peticiones/:id/aprobar` — requiere sesión (DESIGN.md 3.8, T6).
pub async fn aprobar(
    auth: AuthSession,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<Json<Peticion>, AppError> {
    decidir(auth, state, id, EstadoPeticion::Aprobada, body).await
}

/// `POST /api/peticiones/:id/denegar` — requiere sesión (DESIGN.md 3.8, T6).
pub async fn denegar(
    auth: AuthSession,
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    body: Bytes,
) -> Result<Json<Peticion>, AppError> {
    decidir(auth, state, id, EstadoPeticion::Denegada, body).await
}
