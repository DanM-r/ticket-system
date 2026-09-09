use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

use backend::routes;
use backend::state::AppState;

/// Cantidad de peticiones simuladas con las que se siembra el `AppState`
/// al arrancar el servidor (DESIGN.md 3.4/3.5), para que el portal tenga
/// datos desde el primer `cargo run`.
const CANTIDAD_SEED_INICIAL: usize = 15;

/// Lee el puerto en el que debe escuchar el servidor desde la variable de
/// entorno `PORT`, con `8080` como valor por defecto si no está definida o
/// no es un número válido.
fn read_port() -> u16 {
    std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080)
}

/// Lee el origin permitido por CORS desde la variable de entorno
/// `CORS_ALLOWED_ORIGIN`, con `http://localhost:5173` (el frontend en
/// desarrollo) como valor por defecto si no está definida (DESIGN.md 3.10).
fn read_cors_allowed_origin() -> String {
    std::env::var("CORS_ALLOWED_ORIGIN").unwrap_or_else(|_| "http://localhost:5173".to_string())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = read_port();
    let cors_allowed_origin = read_cors_allowed_origin();

    let state = Arc::new(AppState::new());
    let creadas = state.seed_peticiones(CANTIDAD_SEED_INICIAL);
    tracing::info!(
        cantidad = creadas,
        "AppState sembrado con peticiones simuladas"
    );

    tracing::info!(origin = %cors_allowed_origin, "CORS restringido al origin configurado");
    let app = routes::build_router_with_cors(state, &cors_allowed_origin);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Servidor escuchando en http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("no se pudo enlazar el puerto del servidor");
    axum::serve(listener, app)
        .await
        .expect("el servidor finalizó inesperadamente");
}
