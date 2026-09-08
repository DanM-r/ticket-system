use uuid::Uuid;

use crate::models::{EstadoPeticion, Peticion};
use crate::state::AppState;

/// Devuelve todas las peticiones del store, opcionalmente filtradas por
/// `estado` (DESIGN.md 3.8). Si `estado` es `None`, devuelve el listado
/// completo. Usado por `GET /api/peticiones` (T5).
pub fn listar_peticiones(state: &AppState, estado: Option<EstadoPeticion>) -> Vec<Peticion> {
    let store = state
        .peticiones
        .read()
        .expect("el lock de peticiones no debería estar envenenado");

    store
        .values()
        .filter(|peticion| estado.is_none_or(|filtro| peticion.estado == filtro))
        .cloned()
        .collect()
}

/// Busca una petición por `id`. Devuelve `None` si no existe en el store.
/// Usado por `GET /api/peticiones/:id` (T5) y por los endpoints de
/// aprobar/denegar (T6).
pub fn obtener_peticion(state: &AppState, id: Uuid) -> Option<Peticion> {
    let store = state
        .peticiones
        .read()
        .expect("el lock de peticiones no debería estar envenenado");
    store.get(&id).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::peticion_generator::generar_peticiones;

    fn sembrar(state: &AppState, cantidad: usize) -> Vec<Peticion> {
        let peticiones = generar_peticiones(cantidad);
        let mut store = state.peticiones.write().unwrap();
        for peticion in &peticiones {
            store.insert(peticion.id, peticion.clone());
        }
        peticiones
    }

    #[test]
    fn listar_peticiones_sin_filtro_devuelve_todas() {
        let state = AppState::new();
        sembrar(&state, 5);

        let peticiones = listar_peticiones(&state, None);
        assert_eq!(peticiones.len(), 5);
    }

    #[test]
    fn listar_peticiones_filtra_por_estado() {
        let state = AppState::new();
        sembrar(&state, 5);

        let pendientes = listar_peticiones(&state, Some(EstadoPeticion::Pendiente));
        assert_eq!(pendientes.len(), 5);

        let aprobadas = listar_peticiones(&state, Some(EstadoPeticion::Aprobada));
        assert!(aprobadas.is_empty());
    }

    #[test]
    fn obtener_peticion_existente_la_devuelve() {
        let state = AppState::new();
        let sembradas = sembrar(&state, 3);
        let id = sembradas[0].id;

        let encontrada = obtener_peticion(&state, id);
        assert_eq!(encontrada.map(|p| p.id), Some(id));
    }

    #[test]
    fn obtener_peticion_inexistente_devuelve_none() {
        let state = AppState::new();
        assert!(obtener_peticion(&state, Uuid::new_v4()).is_none());
    }
}
