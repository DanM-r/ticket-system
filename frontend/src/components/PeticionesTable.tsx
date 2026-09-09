import { Link } from 'react-router-dom'
import type { Peticion } from '../types'
import EstadoBadge from './EstadoBadge'
import SeveridadBadge from './SeveridadBadge'

interface PeticionesTableProps {
  peticiones: Peticion[]
}

function formatearFecha(fechaIso: string): string {
  const fecha = new Date(fechaIso)
  return Number.isNaN(fecha.getTime()) ? fechaIso : fecha.toLocaleString()
}

/**
 * Tabla de peticiones (DESIGN.md 4.6): columnas tipo, usuario, severidad,
 * estado y fecha de creación; cada fila enlaza al detalle
 * (`/peticiones/:id`, implementado en T11). No consulta la API por su
 * cuenta: recibe el listado ya filtrado por [`PeticionesPage`] (T10), que es
 * quien decide el estado de carga/vacío/error.
 */
function PeticionesTable({ peticiones }: PeticionesTableProps) {
  return (
    <table className="peticiones-table">
      <thead>
        <tr>
          <th>Tipo</th>
          <th>Usuario</th>
          <th>Severidad</th>
          <th>Estado</th>
          <th>Fecha de creación</th>
        </tr>
      </thead>
      <tbody>
        {peticiones.map((peticion) => (
          <tr key={peticion.id}>
            <td>
              <Link to={`/peticiones/${peticion.id}`}>{peticion.tipo_peticion}</Link>
            </td>
            <td>{peticion.usuario}</td>
            <td>
              <SeveridadBadge severidad={peticion.severidad} />
            </td>
            <td>
              <EstadoBadge estado={peticion.estado} />
            </td>
            <td>{formatearFecha(peticion.fecha_creacion)}</td>
          </tr>
        ))}
      </tbody>
    </table>
  )
}

export default PeticionesTable
