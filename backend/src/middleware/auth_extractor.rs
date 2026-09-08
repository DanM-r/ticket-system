use std::sync::Arc;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;

use crate::domain::auth_store;
use crate::errors::AppError;
use crate::models::Rol;
use crate::state::AppState;

/// Extractor de Axum que resuelve la sesión asociada al header
/// `Authorization: Bearer <token>` de la request (DESIGN.md 3.6).
///
/// Cualquier handler que declare `AuthSession` como parámetro solo se
/// ejecuta si la extracción tiene éxito: si el header falta, no tiene el
/// esquema `Bearer`, o el token no corresponde a ninguna sesión vigente en
/// el store, la request se rechaza automáticamente con
/// `401 no_autenticado` (vía [`AppError::NoAutenticado`]) antes de llegar
/// al cuerpo del handler.
///
/// El `nombre` y `rol` expuestos aquí son la única fuente confiable para
/// rellenar campos de auditoría (`decidido_por` / `rol_decisor`, T6): nunca
/// deben tomarse valores equivalentes enviados en el body de la request.
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub nombre: String,
    pub rol: Rol,
}

/// Extrae el token del header `Authorization: Bearer <token>`, si está
/// presente y bien formado. Devuelve `None` en cualquier otro caso (header
/// ausente, esquema distinto de `Bearer`, valor no-UTF8, etc.).
fn extraer_bearer_token(parts: &Parts) -> Option<&str> {
    parts
        .headers
        .get(AUTHORIZATION)
        .and_then(|valor| valor.to_str().ok())
        .and_then(|valor| valor.strip_prefix("Bearer "))
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for AuthSession {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let token = extraer_bearer_token(parts).ok_or(AppError::NoAutenticado)?;

        let sesion = auth_store::buscar_sesion(state, token).ok_or(AppError::NoAutenticado)?;

        Ok(AuthSession {
            nombre: sesion.nombre,
            rol: sesion.rol,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::{HeaderValue, Method, Uri};

    fn parts_con_header(valor: Option<&str>) -> Parts {
        let mut request = axum::http::Request::builder()
            .method(Method::GET)
            .uri(Uri::from_static("/"))
            .body(())
            .unwrap();

        if let Some(valor) = valor {
            request
                .headers_mut()
                .insert(AUTHORIZATION, HeaderValue::from_str(valor).unwrap());
        }

        let (parts, _) = request.into_parts();
        parts
    }

    #[test]
    fn extraer_bearer_token_sin_header_devuelve_none() {
        let parts = parts_con_header(None);
        assert!(extraer_bearer_token(&parts).is_none());
    }

    #[test]
    fn extraer_bearer_token_con_esquema_distinto_devuelve_none() {
        let parts = parts_con_header(Some("Basic abc123"));
        assert!(extraer_bearer_token(&parts).is_none());
    }

    #[test]
    fn extraer_bearer_token_valido_devuelve_el_token() {
        let parts = parts_con_header(Some("Bearer mi-token"));
        assert_eq!(extraer_bearer_token(&parts), Some("mi-token"));
    }
}
