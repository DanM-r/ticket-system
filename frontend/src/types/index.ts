/**
 * Tipos compartidos del frontend, reflejando exactamente el modelo de datos
 * y los contratos de API del backend (Rust) — ver:
 * - backend/src/models/peticion.rs (Peticion, Severidad, EstadoPeticion, Decision)
 * - backend/src/models/auth.rs (Rol, LoginRequest, LoginResponse)
 * - backend/src/routes/peticiones.rs (payloads de listar/decidir/generar)
 * - backend/src/routes/auth.rs (MeResponse)
 * - DESIGN.md secciones 3.3 y 3.8.
 */

/** Nivel de severidad de una petición (backend: `Severidad`, snake_case). */
export type Severidad = 'baja' | 'media' | 'alta' | 'critica'

/** Estado del ciclo de vida de una petición (backend: `EstadoPeticion`). */
export type EstadoPeticion = 'pendiente' | 'aprobada' | 'denegada'

/** Rol de la persona autenticada (backend: `Rol`). */
export type Rol = 'it' | 'administracion'

/**
 * Registro de auditoría mínima embebido en la petición (backend:
 * `Decision`). Solo presente (no `null`) cuando `estado !== 'pendiente'`.
 */
export interface Decision {
  decidido_por: string
  rol_decisor: Rol
  fecha_decision: string
  comentario?: string | null
}

/** Petición simulada de autorización (backend: `Peticion`). */
export interface Peticion {
  id: string
  tipo_peticion: string
  usuario: string
  email: string
  fecha_creacion: string
  cuerpo_mensaje: string
  severidad: Severidad
  estado: EstadoPeticion
  decision: Decision | null
}

/** Body de `POST /api/auth/login` (backend: `LoginRequest`). */
export interface LoginRequest {
  nombre: string
  rol: Rol
}

/** Respuesta de `POST /api/auth/login` (backend: `LoginResponse`). */
export interface LoginResponse {
  token: string
  nombre: string
  rol: Rol
}

/** Respuesta de `GET /api/auth/me` (backend: `MeResponse`). */
export interface MeResponse {
  nombre: string
  rol: Rol
}

/** Respuesta de `GET /api/peticiones` (backend: `ListarPeticionesResponse`). */
export interface ListarPeticionesResponse {
  peticiones: Peticion[]
}

/**
 * Body (opcional) de `POST /api/peticiones/:id/aprobar` y `.../denegar`
 * (backend: `DecidirPeticionRequest`). Deliberadamente no incluye
 * `decidido_por` ni `rol`: el decisor real siempre se deriva de la sesión
 * server-side, nunca del body.
 */
export interface DecidirPeticionRequest {
  comentario?: string
}

/** Respuesta de `POST /api/peticiones/generar` (backend: `GenerarPeticionesResponse`). */
export interface GenerarPeticionesResponse {
  creadas: number
}

/** Respuesta de `GET /api/health`. */
export interface HealthResponse {
  status: string
}

/**
 * Detalle de un error de la API, tal como lo envuelve el backend en toda
 * respuesta 4xx/5xx: `{ "error": { "code", "message" } }` (backend:
 * `errors.rs`, DESIGN.md 3.8/3.9).
 */
export interface ApiErrorDetalle {
  code: string
  message: string
}

/** Envolvente de error tal como la envía el backend. */
export interface ApiErrorEnvelope {
  error: ApiErrorDetalle
}
