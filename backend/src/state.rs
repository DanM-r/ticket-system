use std::collections::HashMap;
use std::sync::RwLock;

use uuid::Uuid;

use crate::domain::peticion_generator::generar_peticiones;
use crate::models::Peticion;

/// Estado compartido del backend, mantenido en memoria del proceso (sin
/// base de datos externa — ver DESIGN.md 3.4 / 7.1). Se expone envuelto en
/// `Arc` a los handlers de Axum.
///
/// El store de sesiones (`sesiones: RwLock<HashMap<String, Sesion>>`) se
/// añade en T3, una vez exista el modelo `Sesion`.
#[derive(Debug, Default)]
pub struct AppState {
    pub peticiones: RwLock<HashMap<Uuid, Peticion>>,
}

impl AppState {
    /// Crea un `AppState` vacío, sin peticiones sembradas.
    pub fn new() -> Self {
        Self {
            peticiones: RwLock::new(HashMap::new()),
        }
    }

    /// Genera `cantidad` peticiones simuladas (DESIGN.md 3.5) y las agrega
    /// al store. Se usa tanto para la siembra inicial al arrancar el
    /// servidor como, en T7, para el endpoint bajo demanda
    /// `POST /api/peticiones/generar`.
    ///
    /// Devuelve la cantidad de peticiones efectivamente insertadas.
    pub fn seed_peticiones(&self, cantidad: usize) -> usize {
        let nuevas = generar_peticiones(cantidad);
        let creadas = nuevas.len();

        let mut store = self
            .peticiones
            .write()
            .expect("el lock de peticiones no debería estar envenenado");
        for peticion in nuevas {
            store.insert(peticion.id, peticion);
        }

        creadas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn appstate_nuevo_empieza_vacio() {
        let state = AppState::new();
        let store = state.peticiones.read().unwrap();
        assert!(store.is_empty());
    }

    #[test]
    fn seed_peticiones_siembra_la_cantidad_pedida() {
        let state = AppState::new();
        let creadas = state.seed_peticiones(15);

        assert_eq!(creadas, 15);
        let store = state.peticiones.read().unwrap();
        assert_eq!(store.len(), 15);
    }

    #[test]
    fn seed_peticiones_es_acumulativo() {
        let state = AppState::new();
        state.seed_peticiones(5);
        state.seed_peticiones(3);

        let store = state.peticiones.read().unwrap();
        assert_eq!(store.len(), 8);
    }
}
