use chrono::Utc;
use uuid::Uuid;

use crate::models::{Rol, Sesion};
use crate::state::AppState;

/// Crea una nueva sesión con un token opaco (UUID v4) para `nombre`/`rol`,
/// la guarda en el store de sesiones y devuelve la sesión creada
/// (DESIGN.md 3.6).
pub fn crear_sesion(state: &AppState, nombre: String, rol: Rol) -> Sesion {
    let sesion = Sesion {
        token: Uuid::new_v4().to_string(),
        nombre,
        rol,
        creada_en: Utc::now(),
    };

    let mut store = state
        .sesiones
        .write()
        .expect("el lock de sesiones no debería estar envenenado");
    store.insert(sesion.token.clone(), sesion.clone());

    sesion
}

/// Elimina la sesión asociada a `token`, si existe. Operación idempotente:
/// no falla si el token no existe en el store (DESIGN.md 3.8, logout).
pub fn eliminar_sesion(state: &AppState, token: &str) {
    let mut store = state
        .sesiones
        .write()
        .expect("el lock de sesiones no debería estar envenenado");
    store.remove(token);
}

/// Busca una sesión vigente por token. Devuelve `None` si el token no
/// existe en el store. Usado por el extractor de autenticación (T4) y por
/// los tests de esta tarea.
pub fn buscar_sesion(state: &AppState, token: &str) -> Option<Sesion> {
    let store = state
        .sesiones
        .read()
        .expect("el lock de sesiones no debería estar envenenado");
    store.get(token).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crear_sesion_la_guarda_en_el_store() {
        let state = AppState::new();
        let sesion = crear_sesion(&state, "Ana Pérez".to_string(), Rol::It);

        assert!(!sesion.token.is_empty());
        assert_eq!(sesion.nombre, "Ana Pérez");
        assert_eq!(sesion.rol, Rol::It);

        let encontrada = buscar_sesion(&state, &sesion.token);
        assert!(encontrada.is_some());
        assert_eq!(encontrada.unwrap().nombre, "Ana Pérez");
    }

    #[test]
    fn eliminar_sesion_la_quita_del_store() {
        let state = AppState::new();
        let sesion = crear_sesion(&state, "Carlos".to_string(), Rol::Administracion);

        eliminar_sesion(&state, &sesion.token);

        assert!(buscar_sesion(&state, &sesion.token).is_none());
    }

    #[test]
    fn eliminar_sesion_inexistente_no_falla() {
        let state = AppState::new();
        eliminar_sesion(&state, "token-que-no-existe");
    }

    #[test]
    fn buscar_sesion_inexistente_devuelve_none() {
        let state = AppState::new();
        assert!(buscar_sesion(&state, "token-que-no-existe").is_none());
    }
}
