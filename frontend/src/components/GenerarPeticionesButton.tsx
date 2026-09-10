import { useState } from 'react'
import { generarPeticiones } from '../api/peticiones'
import { ApiError } from '../api/client'
import { useAuth } from '../context/AuthContext'

/** Límites de `cantidad` reflejando la validación del backend (DESIGN.md 3.8). */
const CANTIDAD_MIN = 1
const CANTIDAD_MAX = 50
const CANTIDAD_DEFAULT = 5

interface GenerarPeticionesButtonProps {
  /**
   * Se llama tras un `POST /api/peticiones/generar` exitoso, para que el
   * llamador (`PeticionesPage`, T10) refresque el listado sin recargar la
   * página completa (criterio de aceptación de T12).
   */
  onGenerado: () => void
}

/**
 * Utilidad de generación de datos de prueba (DESIGN.md 4.6, T12): input
 * numérico con los mismos límites que el backend (`1..=50`, sección 3.8) +
 * botón que llama a `POST /api/peticiones/generar` y avisa al llamador para
 * que refresque el listado. No renderiza nada sin sesión activa (el botón
 * solo debe estar visible con sesión activa, criterio de aceptación de
 * T12) — en la práctica ya vive detrás de `ProtectedRoute`, pero el
 * componente valida su propia precondición para no depender solo de eso.
 *
 * La validación de rango es puramente de cliente y no espera respuesta del
 * backend: se deshabilita el botón y se muestra un mensaje si `cantidad`
 * está fuera de `1..=50`, en vez de dejar que el backend responda
 * `400 datos_invalidos` (DESIGN.md 3.8/3.11).
 */
function GenerarPeticionesButton({ onGenerado }: GenerarPeticionesButtonProps) {
  const { sesion } = useAuth()

  const [cantidadTexto, setCantidadTexto] = useState(String(CANTIDAD_DEFAULT))
  const [generando, setGenerando] = useState(false)
  const [error, setError] = useState<string | null>(null)

  if (!sesion) {
    return null
  }

  const cantidadNumerica = Number(cantidadTexto)
  const cantidadValida =
    cantidadTexto.trim().length > 0 &&
    Number.isInteger(cantidadNumerica) &&
    cantidadNumerica >= CANTIDAD_MIN &&
    cantidadNumerica <= CANTIDAD_MAX

  async function handleGenerar() {
    if (!cantidadValida) {
      return
    }
    setGenerando(true)
    setError(null)
    try {
      await generarPeticiones(cantidadNumerica)
      onGenerado()
    } catch (err) {
      setError(
        err instanceof ApiError
          ? err.message
          : 'No se pudo conectar con el servidor. Intenta de nuevo.',
      )
    } finally {
      setGenerando(false)
    }
  }

  return (
    <div className="generar-peticiones">
      <label htmlFor="generar-cantidad">Generar peticiones de prueba</label>
      <div className="generar-peticiones-controles">
        <input
          id="generar-cantidad"
          type="number"
          min={CANTIDAD_MIN}
          max={CANTIDAD_MAX}
          value={cantidadTexto}
          onChange={(event) => setCantidadTexto(event.target.value)}
          disabled={generando}
        />
        <button type="button" onClick={() => void handleGenerar()} disabled={generando || !cantidadValida}>
          {generando ? 'Generando…' : 'Generar'}
        </button>
      </div>

      {!cantidadValida && (
        <p className="generar-peticiones-error" role="alert">
          Ingresa un número entero entre {CANTIDAD_MIN} y {CANTIDAD_MAX}.
        </p>
      )}

      {cantidadValida && error && (
        <p className="generar-peticiones-error" role="alert">
          {error}
        </p>
      )}
    </div>
  )
}

export default GenerarPeticionesButton
