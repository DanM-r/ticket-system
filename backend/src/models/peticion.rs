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
