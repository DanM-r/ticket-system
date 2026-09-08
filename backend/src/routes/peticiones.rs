use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::peticion_store;
use crate::errors::AppError;
use crate::middleware::AuthSession;
use crate::models::{EstadoPeticion, Peticion};
use crate::state::AppState;

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
