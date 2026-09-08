use chrono::{Duration, Utc};
use rand::seq::SliceRandom;
use rand::Rng;
use uuid::Uuid;

use crate::models::{EstadoPeticion, Peticion, Severidad};

/// Tipos de petición plausibles usados para generar datos simulados.
const TIPOS_PETICION: &[&str] = &[
    "Acceso a VPN",
    "Restablecimiento de contraseña",
    "Alta de usuario",
    "Compra de equipo",
    "Acceso a carpeta compartida",
    "Instalación de software",
    "Cambio de permisos",
    "Solicitud de vacaciones",
];

const NOMBRES: &[&str] = &[
    "Ana", "Carlos", "Beatriz", "Diego", "Elena", "Fernando", "Gabriela", "Hugo", "Isabel",
    "Javier",
];

const APELLIDOS: &[&str] = &[
    "Perez", "Ruiz", "Gomez", "Martinez", "Lopez", "Sanchez", "Torres", "Ramirez", "Flores",
    "Castro",
];

const SEVERIDADES: &[Severidad] = &[
    Severidad::Baja,
    Severidad::Media,
    Severidad::Alta,
    Severidad::Critica,
];

/// Genera el cuerpo del mensaje de la petición interpolando el nombre del
/// solicitante en una plantilla específica por `tipo_peticion`.
fn generar_cuerpo_mensaje(tipo_peticion: &str, usuario: &str) -> String {
    match tipo_peticion {
        "Acceso a VPN" => format!(
            "Solicito acceso VPN para trabajo remoto. Firma, {usuario}."
        ),
        "Restablecimiento de contraseña" => format!(
            "No puedo ingresar a mi cuenta corporativa, necesito restablecer mi contraseña. Gracias, {usuario}."
        ),
        "Alta de usuario" => format!(
            "Solicito el alta de un nuevo usuario en el sistema a nombre de {usuario} para su incorporación."
        ),
        "Compra de equipo" => format!(
            "Requiero la compra de un equipo nuevo para continuar con mis labores. Solicitante: {usuario}."
        ),
        "Acceso a carpeta compartida" => format!(
            "Necesito acceso a la carpeta compartida del equipo para colaborar en proyectos activos. {usuario}."
        ),
        "Instalación de software" => format!(
            "Solicito la instalación de una herramienta de software necesaria para mi puesto. Pide {usuario}."
        ),
        "Cambio de permisos" => format!(
            "Requiero un cambio de permisos sobre un recurso interno para poder realizar mi trabajo. {usuario}."
        ),
        "Solicitud de vacaciones" => format!(
            "Solicito autorización para tomar días de vacaciones en las próximas semanas. Atte, {usuario}."
        ),
        otro => format!("Solicito atención sobre: {otro}. Firma, {usuario}."),
    }
}

/// Genera `cantidad` peticiones simuladas combinando listas fijas embebidas
/// en el código (sin red ni archivos externos), según DESIGN.md 3.5.
///
/// Todas las peticiones generadas nacen en estado [`EstadoPeticion::Pendiente`]
/// y sin decisión, con `fecha_creacion` dentro de los últimos 15 días
/// (siempre `<= now`).
pub fn generar_peticiones(cantidad: usize) -> Vec<Peticion> {
    let mut rng = rand::thread_rng();
    let ahora = Utc::now();

    (0..cantidad)
        .map(|_| {
            let tipo_peticion = TIPOS_PETICION
                .choose(&mut rng)
                .expect("TIPOS_PETICION no debe estar vacío")
                .to_string();
            let nombre = NOMBRES
                .choose(&mut rng)
                .expect("NOMBRES no debe estar vacío");
            let apellido = APELLIDOS
                .choose(&mut rng)
                .expect("APELLIDOS no debe estar vacío");
            let usuario = format!("{nombre} {apellido}");
            let email = format!(
                "{}.{}@empresa.test",
                nombre.to_lowercase(),
                apellido.to_lowercase()
            );
            let cuerpo_mensaje = generar_cuerpo_mensaje(&tipo_peticion, &usuario);
            let severidad = *SEVERIDADES
                .choose(&mut rng)
                .expect("SEVERIDADES no debe estar vacío");

            let dias_atras = rng.gen_range(0..15);
            let segundos_atras = rng.gen_range(0..86_400);
            let fecha_creacion =
                ahora - Duration::days(dias_atras) - Duration::seconds(segundos_atras);

            Peticion {
                id: Uuid::new_v4(),
                tipo_peticion,
                usuario,
                email,
                fecha_creacion,
                cuerpo_mensaje,
                severidad,
                estado: EstadoPeticion::Pendiente,
                decision: None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn genera_la_cantidad_solicitada() {
        let peticiones = generar_peticiones(15);
        assert_eq!(peticiones.len(), 15);
    }

    #[test]
    fn genera_cero_peticiones_cuando_se_pide_cero() {
        let peticiones = generar_peticiones(0);
        assert!(peticiones.is_empty());
    }

    #[test]
    fn cada_peticion_tiene_campos_requeridos_no_vacios_y_validos() {
        let ahora = Utc::now();

        for peticion in generar_peticiones(50) {
            assert!(!peticion.tipo_peticion.trim().is_empty());
            assert!(TIPOS_PETICION.contains(&peticion.tipo_peticion.as_str()));

            assert!(!peticion.usuario.trim().is_empty());

            assert!(!peticion.email.trim().is_empty());
            assert!(peticion.email.contains('@'));
            assert!(peticion.email.ends_with("@empresa.test"));

            assert!(!peticion.cuerpo_mensaje.trim().is_empty());

            assert!(matches!(
                peticion.severidad,
                Severidad::Baja | Severidad::Media | Severidad::Alta | Severidad::Critica
            ));

            assert_eq!(peticion.estado, EstadoPeticion::Pendiente);
            assert!(peticion.decision.is_none());

            assert!(peticion.fecha_creacion <= ahora);
        }
    }

    #[test]
    fn genera_ids_unicos() {
        let peticiones = generar_peticiones(100);
        let ids: HashSet<_> = peticiones.iter().map(|p| p.id).collect();
        assert_eq!(ids.len(), peticiones.len());
    }
}
