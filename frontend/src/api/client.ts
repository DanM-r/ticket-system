import type { ApiErrorEnvelope, HealthResponse, Rol } from '../types'

/**
 * Cliente API del frontend (DESIGN.md 4.5): un wrapper único sobre `fetch`
 * que:
 * - antepone `VITE_API_BASE_URL` a la ruta solicitada;
 * - serializa/deserializa JSON;
 * - adjunta `Authorization: Bearer <token>` si hay una sesión guardada;
 * - ante un status >= 400, parsea el cuerpo `{ error: { code, message } }`
 *   del backend y lanza un `ApiError` tipado.
 */

const API_BASE_URL: string =
  (import.meta.env.VITE_API_BASE_URL as string | undefined) ?? 'http://localhost:8080'

/**
 * Clave usada en `localStorage` para persistir la sesión simulada
 * `{ token, nombre, rol }` (DESIGN.md 4.4). Se centraliza aquí para que el
 * `AuthContext` (T9) y este cliente lean/escriban siempre el mismo formato.
 */
export const SESSION_STORAGE_KEY = 'ticket-system:sesion'

/** Forma de la sesión persistida en `localStorage` (DESIGN.md 3.8/4.4). */
export interface SesionAlmacenada {
  token: string
  nombre: string
  rol: Rol
}

/**
 * Lee la sesión actualmente guardada en `localStorage`, si existe y tiene
 * la forma esperada (`token`, `nombre`, `rol` válidos). Devuelve `null` en
 * cualquier otro caso (sin sesión, JSON corrupto, `localStorage` no
 * disponible). Usada por [`AuthContext`](../context/AuthContext.tsx) (T9)
 * para inicializar la sesión de React al cargar la app, y por este cliente
 * para adjuntar el header `Authorization`.
 */
export function leerSesionAlmacenada(): SesionAlmacenada | null {
  try {
    const raw = window.localStorage.getItem(SESSION_STORAGE_KEY)
    if (!raw) {
      return null
    }
    const sesion = JSON.parse(raw) as Partial<SesionAlmacenada>
    if (
      typeof sesion.token === 'string' &&
      sesion.token.length > 0 &&
      typeof sesion.nombre === 'string' &&
      (sesion.rol === 'it' || sesion.rol === 'administracion')
    ) {
      return { token: sesion.token, nombre: sesion.nombre, rol: sesion.rol }
    }
    return null
  } catch {
    return null
  }
}

/** Persiste la sesión activa en `localStorage` (DESIGN.md 4.4). */
export function guardarSesionAlmacenada(sesion: SesionAlmacenada): void {
  try {
    window.localStorage.setItem(SESSION_STORAGE_KEY, JSON.stringify(sesion))
  } catch {
    // `localStorage` no disponible (ej. modo privado estricto): la sesión
    // queda solo en el estado de React para la pestaña actual.
  }
}

/** Elimina la sesión persistida en `localStorage` (ej. al hacer logout). */
export function limpiarSesionAlmacenada(): void {
  try {
    window.localStorage.removeItem(SESSION_STORAGE_KEY)
  } catch {
    // no-op: si `localStorage` no está disponible, no había nada que limpiar.
  }
}

/** Token de la sesión guardada en `localStorage`, o `null` si no hay sesión. */
function leerTokenAlmacenado(): string | null {
  return leerSesionAlmacenada()?.token ?? null
}

/**
 * Ante una respuesta `401` de cualquier llamada autenticada, limpia la
 * sesión local y fuerza una redirección "dura" (`window.location`, no
 * `react-router`) a `/login` (DESIGN.md 4.4). Se resuelve así, en vez de con
 * `useNavigate`, porque este cliente vive fuera del árbol de React y no
 * tiene acceso al router; la recarga completa además asegura que
 * `AuthContext` se reinicialice desde `localStorage` (ya vacío) en vez de
 * quedar con estado inconsistente. Es un caso esperado si el backend se
 * reinicia y pierde las sesiones en memoria (ver DESIGN.md 7.4), o si el
 * token fue invalidado por un logout en otra pestaña.
 */
function manejarNoAutenticado(): void {
  limpiarSesionAlmacenada()
  if (typeof window !== 'undefined' && window.location.pathname !== '/login') {
    window.location.href = '/login'
  }
}

/** Error tipado lanzado por [`request`] ante una respuesta de error de la API. */
export class ApiError extends Error {
  /** Código de error estable (`snake_case`) devuelto por el backend, ej. `peticion_ya_decidida`. */
  readonly code: string
  /** Status HTTP de la respuesta. */
  readonly status: number

  constructor(code: string, message: string, status: number) {
    super(message)
    this.name = 'ApiError'
    this.code = code
    this.status = status
  }
}

/** Métodos HTTP soportados por el cliente. */
export type HttpMethod = 'GET' | 'POST' | 'PUT' | 'DELETE'

export interface RequestOptions {
  method?: HttpMethod
  /** Cuerpo a serializar como JSON. Se omite si es `undefined`. */
  body?: unknown
  /** Query params a anexar a la URL; los valores `undefined` se omiten. */
  query?: Record<string, string | number | boolean | undefined>
}

function construirUrl(path: string, query?: RequestOptions['query']): string {
  const base = API_BASE_URL.endsWith('/') ? API_BASE_URL.slice(0, -1) : API_BASE_URL
  const rutaRelativa = path.startsWith('/') ? path : `/${path}`
  const url = new URL(`${base}${rutaRelativa}`)

  if (query) {
    for (const [clave, valor] of Object.entries(query)) {
      if (valor !== undefined) {
        url.searchParams.set(clave, String(valor))
      }
    }
  }

  return url.toString()
}

/**
 * Intenta parsear el cuerpo de una respuesta de error con el formato
 * estándar `{ error: { code, message } }` (DESIGN.md 3.8). Si el cuerpo no
 * tiene esa forma (ej. una respuesta inesperada del servidor o de un proxy
 * intermedio), se cae a un código/mensaje genérico en vez de romper.
 */
async function parsearError(response: Response): Promise<ApiError> {
  let code = 'error_desconocido'
  let message = `La API respondió con status ${response.status}.`

  try {
    const data = (await response.json()) as Partial<ApiErrorEnvelope>
    if (data.error?.code && data.error?.message) {
      code = data.error.code
      message = data.error.message
    }
  } catch {
    // Cuerpo vacío o no-JSON: se mantienen los valores genéricos.
  }

  return new ApiError(code, message, response.status)
}

/**
 * Wrapper genérico de `fetch` (DESIGN.md 4.5). Lanza [`ApiError`] ante
 * cualquier status >= 400; para respuestas sin contenido (`204`) devuelve
 * `undefined` como `T`.
 */
export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = 'GET', body, query } = options

  const headers: Record<string, string> = {
    Accept: 'application/json',
  }

  const token = leerTokenAlmacenado()
  if (token) {
    headers.Authorization = `Bearer ${token}`
  }

  let requestBody: string | undefined
  if (body !== undefined) {
    headers['Content-Type'] = 'application/json'
    requestBody = JSON.stringify(body)
  }

  const response = await fetch(construirUrl(path, query), {
    method,
    headers,
    body: requestBody,
  })

  if (!response.ok) {
    const error = await parsearError(response)
    if (response.status === 401) {
      manejarNoAutenticado()
    }
    throw error
  }

  if (response.status === 204) {
    return undefined as T
  }

  return (await response.json()) as T
}

/**
 * Función concreta de prueba de conexión con el backend: llama a
 * `GET /api/health` (público, sin autenticación) y devuelve `{ status }`.
 */
export function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>('/api/health')
}
