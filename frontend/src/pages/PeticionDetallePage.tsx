import { Link, useNavigate, useParams } from 'react-router-dom'
import PeticionDetalle from '../components/PeticionDetalle'
import { useAuth } from '../context/AuthContext'
import { ETIQUETA_ROL } from '../utils/etiquetas'

/**
 * Página de detalle de una petición (DESIGN.md 4.3/4.6, T11), montada en
 * `/peticiones/:id`. Provee el layout compartido con [`PeticionesPage`]
 * (encabezado de sesión, logout, link de vuelta al listado) y delega toda la
 * lógica de datos/acciones a [`PeticionDetalle`], que consume
 * `GET /api/peticiones/:id` y las acciones de aprobar/denegar por su cuenta.
 */
function PeticionDetallePage() {
  const { id } = useParams<{ id: string }>()
  const { sesion, cerrarSesion } = useAuth()
  const navigate = useNavigate()

  async function handleLogout() {
    await cerrarSesion()
    navigate('/login', { replace: true })
  }

  return (
    <main className="peticiones-page">
      <header className="peticiones-header">
        <p>
          Sesión activa: <strong>{sesion?.nombre}</strong>
          {sesion ? ` (${ETIQUETA_ROL[sesion.rol] ?? sesion.rol})` : null}
        </p>
        <button type="button" onClick={handleLogout}>
          Cerrar sesión
        </button>
      </header>

      <p>
        <Link to="/peticiones">&larr; Volver al listado</Link>
      </p>

      <h1>Detalle de petición</h1>

      {id ? (
        <PeticionDetalle id={id} />
      ) : (
        <p className="peticion-detalle-error" role="alert">
          Id de petición inválido.
        </p>
      )}
    </main>
  )
}

export default PeticionDetallePage
