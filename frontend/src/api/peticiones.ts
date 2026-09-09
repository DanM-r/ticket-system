import { request } from './client'
import type { EstadoPeticion, ListarPeticionesResponse } from '../types'

/**
 * Llama a `GET /api/peticiones` (DESIGN.md 3.8/4.6), opcionalmente filtrando
 * por `estado`. Sin `estado` (o `undefined`) devuelve el listado completo.
 * Requiere sesión activa: ante `401` el cliente ([`request`]) limpia la
 * sesión local y redirige a `/login` antes de que este error se propague.
 */
export function listarPeticiones(estado?: EstadoPeticion): Promise<ListarPeticionesResponse> {
  return request<ListarPeticionesResponse>('/api/peticiones', { query: { estado } })
}
