use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

/// Error de dominio/aplicación centralizado (DESIGN.md 3.9). Cada variante
/// se mapea a un status code HTTP y al formato de error estándar de la API
/// (DESIGN.md 3.8): `{ "error": { "code", "message" } }`.
///
/// Solo se incluyen aquí las variantes efectivamente usadas hasta esta
/// tarea, para evitar código muerto; tareas posteriores (T4-T7) añaden las
/// variantes que necesiten (`NoAutenticado`, `PeticionNoEncontrada`,
/// `PeticionYaDecidida`), siguiendo el mismo patrón.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    DatosInvalidos(String),
}

impl AppError {
    fn code(&self) -> &'static str {
        match self {
            AppError::DatosInvalidos(_) => "datos_invalidos",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            AppError::DatosInvalidos(_) => StatusCode::BAD_REQUEST,
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
