use chrono::Utc;
use uuid::Uuid;

use crate::models::{Decision, EstadoPeticion, Peticion, Rol};
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

/// Error de dominio al intentar decidir (aprobar/denegar) una petición
/// (T6). Se mapea a un `AppError` concreto en `routes::peticiones`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecidirPeticionError {
    /// El `id` es un UUID válido pero no existe en el store.
    NoEncontrada,
    /// La petición ya tiene `estado != pendiente`: no puede volver a
    /// decidirse.
    YaDecidida,
}

/// Aprueba o deniega (según `nuevo_estado`) la petición `id`, siempre que su
/// estado actual sea `pendiente` (DESIGN.md 3.8, T6).
///
/// `decidido_por` y `rol_decisor` deben provenir siempre de la `Sesion`
/// resuelta por el extractor `AuthSession` (T4) — nunca de un valor enviado
/// por el cliente en el body — para que un cliente no pueda falsificar
/// quién tomó la decisión (DESIGN.md 3.6).
///
/// Si la petición no existe, devuelve `DecidirPeticionError::NoEncontrada`.
/// Si ya fue decidida anteriormente, devuelve
/// `DecidirPeticionError::YaDecidida` y no modifica la decisión existente.
pub fn decidir_peticion(
    state: &AppState,
    id: Uuid,
    nuevo_estado: EstadoPeticion,
    decidido_por: String,
    rol_decisor: Rol,
    comentario: Option<String>,
) -> Result<Peticion, DecidirPeticionError> {
    let mut store = state
        .peticiones
        .write()
        .expect("el lock de peticiones no debería estar envenenado");

    let peticion = store
        .get_mut(&id)
        .ok_or(DecidirPeticionError::NoEncontrada)?;

    if peticion.estado != EstadoPeticion::Pendiente {
        return Err(DecidirPeticionError::YaDecidida);
    }

    peticion.estado = nuevo_estado;
    peticion.decision = Some(Decision {
        decidido_por,
        rol_decisor,
        fecha_decision: Utc::now(),
        comentario,
    });

    Ok(peticion.clone())
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

    #[test]
    fn decidir_peticion_pendiente_actualiza_estado_y_decision() {
        let state = AppState::new();
        let sembradas = sembrar(&state, 1);
        let id = sembradas[0].id;

        let resultado = decidir_peticion(
            &state,
            id,
            EstadoPeticion::Aprobada,
            "Ana Pérez".to_string(),
            Rol::It,
            Some("ok".to_string()),
        )
        .expect("debería poder decidirse una petición pendiente");

        assert_eq!(resultado.estado, EstadoPeticion::Aprobada);
        let decision = resultado.decision.expect("debería tener decision");
        assert_eq!(decision.decidido_por, "Ana Pérez");
        assert_eq!(decision.rol_decisor, Rol::It);
        assert_eq!(decision.comentario, Some("ok".to_string()));

        let en_store = obtener_peticion(&state, id).unwrap();
        assert_eq!(en_store.estado, EstadoPeticion::Aprobada);
    }

    #[test]
    fn decidir_peticion_ya_decidida_devuelve_error_y_no_modifica_la_decision() {
        let state = AppState::new();
        let sembradas = sembrar(&state, 1);
        let id = sembradas[0].id;

        decidir_peticion(
            &state,
            id,
            EstadoPeticion::Aprobada,
            "Ana Pérez".to_string(),
            Rol::It,
            None,
        )
        .unwrap();

        let resultado = decidir_peticion(
            &state,
            id,
            EstadoPeticion::Denegada,
            "Carlos".to_string(),
            Rol::Administracion,
            None,
        );

        assert_eq!(resultado, Err(DecidirPeticionError::YaDecidida));

        let en_store = obtener_peticion(&state, id).unwrap();
        assert_eq!(en_store.estado, EstadoPeticion::Aprobada);
        assert_eq!(en_store.decision.unwrap().decidido_por, "Ana Pérez");
    }

    #[test]
    fn decidir_peticion_inexistente_devuelve_error() {
        let state = AppState::new();

        let resultado = decidir_peticion(
            &state,
            Uuid::new_v4(),
            EstadoPeticion::Aprobada,
            "Ana Pérez".to_string(),
            Rol::It,
            None,
        );

        assert_eq!(resultado, Err(DecidirPeticionError::NoEncontrada));
    }
}
