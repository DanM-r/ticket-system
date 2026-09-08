use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Rol de la persona autenticada. Se usa tanto para restringir los valores
/// aceptados en el login (T3) como para el campo `rol_decisor` embebido en
/// una [`crate::models::peticion::Decision`] (T2/T6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rol {
    It,
    Administracion,
}

impl std::str::FromStr for Rol {
    type Err = ();

    /// Parsea el valor textual de `rol` recibido en el body de
    /// `POST /api/auth/login`. Se hace un parseo manual (en vez de derivar
    /// `Deserialize` directamente sobre el campo del payload de login) para
    /// poder devolver `400 datos_invalidos` con el formato de error
    /// estándar del backend, en vez del rechazo genérico de `serde`/`axum`
    /// (DESIGN.md 3.8/3.11).
    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "it" => Ok(Rol::It),
            "administracion" => Ok(Rol::Administracion),
            _ => Err(()),
        }
    }
}

/// Sesión activa creada tras un login simulado (DESIGN.md 3.3/3.6). Vive
/// solo en el store en memoria del backend; la API nunca expone este struct
/// completo (ver [`LoginResponse`] para el payload público).
#[derive(Debug, Clone)]
pub struct Sesion {
    pub token: String,
    pub nombre: String,
    pub rol: Rol,
    pub creada_en: DateTime<Utc>,
}

/// Body de la request `POST /api/auth/login`. `rol` se recibe como texto
/// libre y se valida/parsea explícitamente contra [`Rol`] en el handler,
/// para poder producir un mensaje de error propio ante un valor
/// desconocido (ver [`Rol::from_str`]).
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub nombre: String,
    pub rol: String,
}

/// Respuesta pública de `POST /api/auth/login` (DESIGN.md 3.8).
#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub nombre: String,
    pub rol: Rol,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rol_parsea_valores_validos() {
        assert_eq!("it".parse::<Rol>().unwrap(), Rol::It);
        assert_eq!(
            "administracion".parse::<Rol>().unwrap(),
            Rol::Administracion
        );
    }

    #[test]
    fn rol_rechaza_valores_desconocidos() {
        assert!("gerente".parse::<Rol>().is_err());
        assert!("".parse::<Rol>().is_err());
        assert!("IT".parse::<Rol>().is_err());
    }
}
