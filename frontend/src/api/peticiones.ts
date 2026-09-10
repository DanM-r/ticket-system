import { request } from './client'
import type {
  DecidirPeticionRequest,
  EstadoPeticion,
  GenerarPeticionesResponse,
  ListarPeticionesResponse,
  Peticion,
} from '../types'

/**
 * Llama a `GET /api/peticiones` (DESIGN.md 3.8/4.6), opcionalmente filtrando
 * por `estado`. Sin `estado` (o `undefined`) devuelve el listado completo.
 * Requiere sesión activa: ante `401` el cliente ([`request`]) limpia la
 * sesión local y redirige a `/login` antes de que este error se propague.
 */
export function listarPeticiones(estado?: EstadoPeticion): Promise<ListarPeticionesResponse> {
  return request<ListarPeticionesResponse>('/api/peticiones', { query: { estado } })
}

/**
 * Llama a `GET /api/peticiones/:id` (DESIGN.md 3.8/4.6). Ante un `id` que no
 * es un UUID válido, el backend responde `400 datos_invalidos`; si no existe
 * ninguna petición con ese `id`, responde `404 peticion_no_encontrada`. Ambos
 * casos se propagan como [`ApiError`](./client) para que la página de
 * detalle (T11) los muestre.
 */
export function obtenerPeticion(id: string): Promise<Peticion> {
  return request<Peticion>(`/api/peticiones/${id}`)
}

/**
 * Llama a `POST /api/peticiones/:id/aprobar` (DESIGN.md 3.8, T11). El
 * `comentario` es opcional; el backend valida su longitud (máx. 500
 * caracteres, `400 datos_invalidos` si se excede) y rechaza la operación con
 * `409 peticion_ya_decidida` si la petición ya no está `pendiente`.
 */
export function aprobarPeticion(id: string, body: DecidirPeticionRequest = {}): Promise<Peticion> {
  return request<Peticion>(`/api/peticiones/${id}/aprobar`, { method: 'POST', body })
}

/**
 * Llama a `POST /api/peticiones/:id/denegar` (DESIGN.md 3.8, T11). Mismas
 * reglas de validación y errores que [`aprobarPeticion`].
 */
export function denegarPeticion(id: string, body: DecidirPeticionRequest = {}): Promise<Peticion> {
  return request<Peticion>(`/api/peticiones/${id}/denegar`, { method: 'POST', body })
}

/**
 * Llama a `POST /api/peticiones/generar` (DESIGN.md 3.8/4.6, T12). `cantidad`
 * es opcional (el backend usa `5` como default si se omite); el backend
 * valida que esté en el rango `1..=50` y responde `400 datos_invalidos` si
 * no lo está, sin crear ninguna petición. El cliente ([`GenerarPeticionesButton`])
 * valida el mismo rango antes de llamar, para no depender de esa respuesta.
 */
export function generarPeticiones(cantidad?: number): Promise<GenerarPeticionesResponse> {
  return request<GenerarPeticionesResponse>('/api/peticiones/generar', {
    method: 'POST',
    query: { cantidad },
  })
}
