import { useNavigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'

const ETIQUETA_ROL: Record<string, string> = {
  it: 'IT',
  administracion: 'Administración',
}

/**
 * Página de listado de peticiones (DESIGN.md 4.3). El contenido principal
 * (consumo de `GET /api/peticiones`, filtro por estado, botón de
 * generación de datos de prueba) se implementa en T10/T12; esta tarea (T9)
 * solo agrega el encabezado con la sesión activa y la acción de logout,
 * ya que la página está protegida por [`ProtectedRoute`].
 */
function PeticionesPage() {
  const { sesion, cerrarSesion } = useAuth()
  const navigate = useNavigate()

  async function handleLogout() {
    await cerrarSesion()
    navigate('/login', { replace: true })
  }

  return (
    <main className="placeholder">
      <header className="peticiones-header">
        <p>
          Sesión activa: <strong>{sesion?.nombre}</strong>
          {sesion ? ` (${ETIQUETA_ROL[sesion.rol] ?? sesion.rol})` : null}
        </p>
        <button type="button" onClick={handleLogout}>
          Cerrar sesión
        </button>
      </header>
      <h1>Peticiones</h1>
      <p>
        Listado de peticiones con filtro por estado. Se implementa en la
        tarea T10.
      </p>
    </main>
  )
}

export default PeticionesPage
