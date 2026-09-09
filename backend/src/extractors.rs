use axum::async_trait;
use axum::extract::rejection::{JsonRejection, QueryRejection};
use axum::extract::{FromRequest, FromRequestParts, Query, Request};
use axum::http::request::Parts;
use axum::Json;
use serde::de::DeserializeOwned;

use crate::errors::AppError;

/// Envoltorio sobre `axum::Json<T>` que traduce cualquier rechazo de
/// deserialización del body (JSON malformado, `Content-Type` incorrecto,
/// body vacío cuando se esperaba uno, etc.) al formato de error estándar de
/// la API (`AppError::DatosInvalidos`, DESIGN.md 3.8/3.9), en vez del
/// rechazo genérico en texto plano que produce Axum por defecto (T7).
///
/// Se usa en lugar de `axum::Json<T>` en cualquier handler que reciba un
/// body JSON obligatorio (ej. `POST /api/auth/login`).
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(json_rejection_to_app_error)?;
        Ok(ValidatedJson(value))
    }
}

fn json_rejection_to_app_error(rejection: JsonRejection) -> AppError {
    AppError::DatosInvalidos(format!(
        "El cuerpo de la petición debe ser JSON válido: {rejection}"
    ))
}

/// Envoltorio sobre `axum::extract::Query<T>` equivalente a
/// [`ValidatedJson`], pero para parámetros de query string (ej. `estado` en
/// `GET /api/peticiones`, `cantidad` en `POST /api/peticiones/generar`).
/// Traduce cualquier rechazo de deserialización al formato de error estándar
/// de la API en vez del rechazo genérico de Axum (T7).
pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<S, T> FromRequestParts<S> for ValidatedQuery<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state)
            .await
            .map_err(query_rejection_to_app_error)?;
        Ok(ValidatedQuery(value))
    }
}

fn query_rejection_to_app_error(rejection: QueryRejection) -> AppError {
    AppError::DatosInvalidos(format!(
        "Los parámetros de consulta no son válidos: {rejection}"
    ))
}
