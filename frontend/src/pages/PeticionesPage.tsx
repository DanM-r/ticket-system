import { useCallback, useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { listarPeticiones } from '../api/peticiones'
import { ApiError } from '../api/client'
import GenerarPeticionesButton from '../components/GenerarPeticionesButton'
import PeticionesTable from '../components/PeticionesTable'
import { useAuth } from '../context/AuthContext'
import type { EstadoPeticion, Peticion } from '../types'
import { ETIQUETA_ROL } from '../utils/etiquetas'

/** Valores del filtro de estado, agregando `'todas'` al enum del backend. */
type FiltroEstado = EstadoPeticion | 'todas'

const OPCIONES_FILTRO: { valor: FiltroEstado; etiqueta: string }[] = [
  { valor: 'todas', etiqueta: 'Todas' },
  { valor: 'pendiente', etiqueta: 'Pendiente' },
  { valor: 'aprobada', etiqueta: 'Aprobada' },
  { valor: 'denegada', etiqueta: 'Denegada' },
]

/**
 * Página de listado de peticiones (DESIGN.md 4.3/4.6/4.7, T10/T12). Consume
 * `GET /api/peticiones` con filtro opcional por `estado`, maneja
 * explícitamente carga/vacío/error, y delega la tabla propiamente a
 * [`PeticionesTable`]. Ante un `401` de la API, el cliente
 * ([`../api/client`], T10) ya limpia la sesión local y redirige a `/login`
 * (DESIGN.md 4.4); el estado de error de esta página cubre el resto de los
 * casos (red caída, errores 4xx/5xx no relacionados con autenticación).
 * También monta [`GenerarPeticionesButton`] (T12), pasándole
 * `cargarPeticiones` como callback de refresco tras generar exitosamente,
 * para que el listado se actualice sin recargar la página completa.
 */
function PeticionesPage() {
  const { sesion, cerrarSesion } = useAuth()
  const navigate = useNavigate()

  const [filtro, setFiltro] = useState<FiltroEstado>('todas')
  const [peticiones, setPeticiones] = useState<Peticion[] | null>(null)
  const [cargando, setCargando] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const cargarPeticiones = useCallback(async (filtroActual: FiltroEstado) => {
    setCargando(true)
    setError(null)
    try {
      const estado = filtroActual === 'todas' ? undefined : filtroActual
      const respuesta = await listarPeticiones(estado)
      setPeticiones(respuesta.peticiones)
    } catch (err) {
      setPeticiones(null)
      setError(
        err instanceof ApiError
          ? err.message
          : 'No se pudo conectar con el servidor. Intenta de nuevo.',
      )
    } finally {
      setCargando(false)
    }
  }, [])

  useEffect(() => {
    void cargarPeticiones(filtro)
  }, [filtro, cargarPeticiones])

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

      <h1>Peticiones</h1>

      <div className="peticiones-toolbar">
        <label htmlFor="filtro-estado">Filtrar por estado</label>
        <select
          id="filtro-estado"
          value={filtro}
          onChange={(event) => setFiltro(event.target.value as FiltroEstado)}
        >
          {OPCIONES_FILTRO.map((opcion) => (
            <option key={opcion.valor} value={opcion.valor}>
              {opcion.etiqueta}
            </option>
          ))}
        </select>
      </div>

      <GenerarPeticionesButton onGenerado={() => void cargarPeticiones(filtro)} />

      {cargando && <p role="status">Cargando peticiones…</p>}

      {!cargando && error && (
        <div className="peticiones-error" role="alert">
          <p>{error}</p>
          <button type="button" onClick={() => void cargarPeticiones(filtro)}>
            Reintentar
          </button>
        </div>
      )}

      {!cargando && !error && peticiones && peticiones.length === 0 && (
        <p className="peticiones-vacio">
          No hay peticiones {filtro === 'todas' ? '' : `en estado "${filtro}"`} para mostrar.
        </p>
      )}

      {!cargando && !error && peticiones && peticiones.length > 0 && (
        <PeticionesTable peticiones={peticiones} />
      )}
    </main>
  )
}

export default PeticionesPage
