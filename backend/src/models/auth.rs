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
