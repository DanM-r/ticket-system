pub mod auth;
pub mod peticion;

pub use auth::{LoginRequest, LoginResponse, Rol, Sesion};
pub use peticion::{Decision, EstadoPeticion, Peticion, Severidad};
