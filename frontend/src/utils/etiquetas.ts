import type { Rol } from '../types'

/**
 * Etiqueta legible en español para cada `Rol` (backend: `Rol`, DESIGN.md
 * 3.3). Centralizado aquí porque se usa en varios lugares del portal
 * (cabecera con la sesión activa, detalle de auditoría de una decisión)
 * y se había duplicado en cada uno.
 */
export const ETIQUETA_ROL: Record<Rol, string> = {
  it: 'IT',
  administracion: 'Administración',
}
