import type { EstadoPeticion } from '../types'

const ETIQUETAS: Record<EstadoPeticion, string> = {
  pendiente: 'Pendiente',
  aprobada: 'Aprobada',
  denegada: 'Denegada',
}

/**
 * Badge visual del estado de una petición (DESIGN.md 4.6). Puramente
 * presentacional: no consulta la API ni maneja estado propio.
 */
function EstadoBadge({ estado }: { estado: EstadoPeticion }) {
  return <span className={`badge badge-estado-${estado}`}>{ETIQUETAS[estado]}</span>
}

export default EstadoBadge
