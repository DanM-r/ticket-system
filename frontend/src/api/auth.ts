import { request } from './client'
import type { LoginRequest, LoginResponse } from '../types'

/**
 * Llama a `POST /api/auth/login` (DESIGN.md 3.8). El backend valida
 * `nombre` (no vacío, máx. 100 caracteres) y `rol` (`it` | `administracion`)
 * y devuelve un token opaco; ante datos inválidos lanza [`ApiError`] con
 * `code === 'datos_invalidos'`, propagado tal cual por [`request`].
 */
export function login(payload: LoginRequest): Promise<LoginResponse> {
  return request<LoginResponse>('/api/auth/login', { method: 'POST', body: payload })
}

/**
 * Llama a `POST /api/auth/logout` (DESIGN.md 3.8). El endpoint es
 * idempotente: responde `204` tanto si el token existe como si no, así que
 * esta llamada nunca debería lanzar salvo un problema real de red.
 */
export function logout(): Promise<void> {
  return request<void>('/api/auth/logout', { method: 'POST' })
}
