import type { Severidad } from '../types'

const ETIQUETAS: Record<Severidad, string> = {
  baja: 'Baja',
  media: 'Media',
  alta: 'Alta',
  critica: 'Crítica',
}

/**
 * Badge visual de la severidad de una petición (DESIGN.md 4.6). Puramente
 * presentacional: no consulta la API ni maneja estado propio.
 */
function SeveridadBadge({ severidad }: { severidad: Severidad }) {
  return <span className={`badge badge-severidad-${severidad}`}>{ETIQUETAS[severidad]}</span>
}

export default SeveridadBadge
