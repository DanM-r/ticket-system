use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::auth::Rol;

/// Nivel de severidad asignado a una petición.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severidad {
    Baja,
    Media,
    Alta,
    Critica,
}

/// Estado del ciclo de vida de una petición. Nace siempre en `Pendiente`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EstadoPeticion {
    Pendiente,
    Aprobada,
    Denegada,
}

impl std::str::FromStr for EstadoPeticion {
    type Err = ();

    /// Parsea el valor textual del query param `estado` recibido en
    /// `GET /api/peticiones` (DESIGN.md 3.8). Se hace un parseo manual (en
    /// vez de dejar que `serde`/`axum` rechacen el valor de forma genérica)
    /// para poder devolver `400 datos_invalidos` con el formato de error
    /// estándar del backend (DESIGN.md 3.11), igual que [`super::auth::Rol`].
    fn from_str(valor: &str) -> Result<Self, Self::Err> {
        match valor {
            "pendiente" => Ok(EstadoPeticion::Pendiente),
            "aprobada" => Ok(EstadoPeticion::Aprobada),
            "denegada" => Ok(EstadoPeticion::Denegada),
            _ => Err(()),
        }
    }
}

/// Registro de auditoría mínima embebido en la petición: quién decidió, con
/// qué rol y cuándo. Solo está presente (`Some`) cuando `estado != pendiente`.
///
/// Estos campos se derivan siempre de la sesión server-side que resuelve el
/// extractor de autenticación (T4), nunca de un valor enviado por el
/// cliente en el body — ver DESIGN.md 3.6. El poblado real de este struct
/// ocurre en T6; en esta tarea solo se define su forma.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Decision {
    pub decidido_por: String,
    pub rol_decisor: Rol,
    pub fecha_decision: DateTime<Utc>,
    pub comentario: Option<String>,
}

/// Petición simulada de autorización (equivalente a un formulario de Teams
/// Forms) sobre la que actúa todo el sistema.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Peticion {
    pub id: Uuid,
    pub tipo_peticion: String,
    pub usuario: String,
    pub email: String,
    pub fecha_creacion: DateTime<Utc>,
    pub cuerpo_mensaje: String,
    pub severidad: Severidad,
    pub estado: EstadoPeticion,
    pub decision: Option<Decision>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estado_peticion_parsea_valores_validos() {
        assert_eq!(
            "pendiente".parse::<EstadoPeticion>().unwrap(),
            EstadoPeticion::Pendiente
        );
        assert_eq!(
            "aprobada".parse::<EstadoPeticion>().unwrap(),
            EstadoPeticion::Aprobada
        );
        assert_eq!(
            "denegada".parse::<EstadoPeticion>().unwrap(),
            EstadoPeticion::Denegada
        );
    }

    #[test]
    fn estado_peticion_rechaza_valores_desconocidos() {
        assert!("cancelada".parse::<EstadoPeticion>().is_err());
        assert!("".parse::<EstadoPeticion>().is_err());
        assert!("Pendiente".parse::<EstadoPeticion>().is_err());
    }
}
