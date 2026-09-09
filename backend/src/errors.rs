use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Error de dominio/aplicación centralizado (DESIGN.md 3.9). Cada variante
/// se mapea a un status code HTTP y al formato de error estándar de la API
/// (DESIGN.md 3.8): `{ "error": { "code", "message" } }`.
///
/// Solo se incluyen aquí las variantes efectivamente usadas hasta esta
/// tarea, para evitar código muerto; tareas posteriores (T7) añaden las
/// variantes que necesiten, siguiendo el mismo patrón.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    DatosInvalidos(String),

    /// Falta el header `Authorization: Bearer <token>`, está mal formado, o
    /// el token no corresponde a ninguna sesión vigente en el store
    /// (DESIGN.md 3.6). Usado por el extractor `AuthSession` (T4).
    #[error("Se requiere un token de sesión válido.")]
    NoAutenticado,

    /// El `id` de la petición es un UUID válido pero no existe en el store
    /// (DESIGN.md 3.8). Usado por `GET /api/peticiones/:id` (T5) y por los
    /// endpoints de aprobar/denegar (T6).
    #[error("La petición solicitada no existe.")]
    PeticionNoEncontrada,

    /// La petición ya fue aprobada o denegada anteriormente: no puede
    /// volver a decidirse (DESIGN.md 3.8). Usado por los endpoints
    /// `POST /api/peticiones/:id/aprobar` y `.../denegar` (T6).
    #[error("Esta petición ya fue decidida anteriormente.")]
    PeticionYaDecidida,

    /// No existe ninguna ruta montada para el método/path solicitado. Se usa
    /// como `fallback` del router (T7) para que incluso un 404 de "ruta
    /// inexistente" siga el formato de error estándar de la API, en vez del
    /// 404 vacío que produce Axum por defecto.
    #[error("El recurso solicitado no existe.")]
    RutaNoEncontrada,

    /// El path existe pero no soporta el método HTTP usado (ej. `DELETE
    /// /api/peticiones`). A diferencia de [`AppError::RutaNoEncontrada`],
    /// Axum resuelve este caso dentro del `MethodRouter` de la ruta, antes
    /// de llegar al `fallback` del router (que solo se activa si ningún
    /// path coincide) — por eso se intercepta con una capa dedicada que
    /// reescribe cualquier `405` a este formato estándar (T7).
    #[error("El método HTTP usado no está permitido para este recurso.")]
    MetodoNoPermitido,
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::DatosInvalidos(_) => "datos_invalidos",
            AppError::NoAutenticado => "no_autenticado",
            AppError::PeticionNoEncontrada => "peticion_no_encontrada",
            AppError::PeticionYaDecidida => "peticion_ya_decidida",
            AppError::RutaNoEncontrada => "ruta_no_encontrada",
            AppError::MetodoNoPermitido => "metodo_no_permitido",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::DatosInvalidos(_) => StatusCode::BAD_REQUEST,
            AppError::NoAutenticado => StatusCode::UNAUTHORIZED,
            AppError::PeticionNoEncontrada => StatusCode::NOT_FOUND,
            AppError::PeticionYaDecidida => StatusCode::CONFLICT,
            AppError::RutaNoEncontrada => StatusCode::NOT_FOUND,
            AppError::MetodoNoPermitido => StatusCode::METHOD_NOT_ALLOWED,
        }
    }
}

#[derive(Serialize)]
struct ErrorEnvelope {
    error: ErrorDetalle,
}

#[derive(Serialize)]
struct ErrorDetalle {
    code: &'static str,
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();
        let code = self.code();
        let message = self.to_string();

        let body = Json(ErrorEnvelope {
            error: ErrorDetalle { code, message },
        });

        (status, body).into_response()
    }
}
