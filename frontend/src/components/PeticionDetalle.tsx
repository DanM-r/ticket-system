import { useCallback, useEffect, useState } from 'react'
import { aprobarPeticion, denegarPeticion, obtenerPeticion } from '../api/peticiones'
import { ApiError } from '../api/client'
import type { Peticion, Rol } from '../types'
import EstadoBadge from './EstadoBadge'
import SeveridadBadge from './SeveridadBadge'

const ETIQUETA_ROL: Record<Rol, string> = {
  it: 'IT',
  administracion: 'Administración',
}

/** Longitud máxima de `comentario` reflejando la validación del backend (DESIGN.md 3.8). */
const COMENTARIO_MAX_LEN = 500

function formatearFecha(fechaIso: string): string {
  const fecha = new Date(fechaIso)
  return Number.isNaN(fecha.getTime()) ? fechaIso : fecha.toLocaleString()
}

interface PeticionDetalleProps {
  id: string
}

/**
 * Detalle completo de una petición (DESIGN.md 4.6, T11). Consume
 * `GET /api/peticiones/:id` por su cuenta (carga inicial y tras cada
 * acción), y maneja explícitamente carga/error (DESIGN.md 4.7). Si
 * `estado === 'pendiente'`, muestra botones "Aprobar"/"Denegar" con un
 * campo de comentario opcional que llaman a
 * `POST /api/peticiones/:id/aprobar` / `.../denegar`; si ya fue decidida,
 * muestra `decidido_por`, `rol_decisor` y `fecha_decision` en vez de los
 * botones.
 *
 * Casos de error de las acciones manejados explícitamente sin romper la
 * vista (DESIGN.md 3.8/5):
 * - `409 peticion_ya_decidida`: otra sesión decidió esta petición mientras
 *   se veía el detalle. Se muestra el mensaje del backend y se refresca el
 *   detalle para reflejar el estado real (oculta los botones).
 * - `400 datos_invalidos` (comentario inválido): se muestra el mensaje del
 *   backend sin perder el comentario ingresado, para que se pueda corregir.
 */
function PeticionDetalle({ id }: PeticionDetalleProps) {
  const [peticion, setPeticion] = useState<Peticion | null>(null)
  const [cargando, setCargando] = useState(true)
  const [errorCarga, setErrorCarga] = useState<string | null>(null)

  const [comentario, setComentario] = useState('')
  const [enviando, setEnviando] = useState(false)
  const [errorAccion, setErrorAccion] = useState<string | null>(null)

  const cargarDetalle = useCallback(async () => {
    setCargando(true)
    setErrorCarga(null)
    try {
      const detalle = await obtenerPeticion(id)
      setPeticion(detalle)
    } catch (err) {
      setPeticion(null)
      setErrorCarga(
        err instanceof ApiError
          ? err.message
          : 'No se pudo conectar con el servidor. Intenta de nuevo.',
      )
    } finally {
      setCargando(false)
    }
  }, [id])

  useEffect(() => {
    void cargarDetalle()
  }, [cargarDetalle])

  async function decidir(accion: 'aprobar' | 'denegar') {
    setEnviando(true)
    setErrorAccion(null)
    try {
      const cuerpo = comentario.trim().length > 0 ? { comentario: comentario.trim() } : {}
      const actualizada =
        accion === 'aprobar' ? await aprobarPeticion(id, cuerpo) : await denegarPeticion(id, cuerpo)
      setPeticion(actualizada)
      setComentario('')
    } catch (err) {
      if (err instanceof ApiError) {
        setErrorAccion(err.message)
        if (err.status === 409) {
          // Otra sesión ya decidió esta petición: refrescamos el detalle
          // para que la vista deje de ofrecer los botones de acción y
          // muestre la info de auditoría real (DESIGN.md 3.8/5).
          void cargarDetalle()
        }
      } else {
        setErrorAccion('No se pudo conectar con el servidor. Intenta de nuevo.')
      }
    } finally {
      setEnviando(false)
    }
  }

  if (cargando) {
    return <p role="status">Cargando petición…</p>
  }

  if (errorCarga || !peticion) {
    return (
      <div className="peticion-detalle-error" role="alert">
        <p>{errorCarga ?? 'No se pudo cargar la petición.'}</p>
        <button type="button" onClick={() => void cargarDetalle()}>
          Reintentar
        </button>
      </div>
    )
  }

  return (
    <article className="peticion-detalle">
      <dl className="peticion-detalle-campos">
        <dt>Tipo de petición</dt>
        <dd>{peticion.tipo_peticion}</dd>

        <dt>Usuario</dt>
        <dd>{peticion.usuario}</dd>

        <dt>Email</dt>
        <dd>{peticion.email}</dd>

        <dt>Fecha de creación</dt>
        <dd>{formatearFecha(peticion.fecha_creacion)}</dd>

        <dt>Severidad</dt>
        <dd>
          <SeveridadBadge severidad={peticion.severidad} />
        </dd>

        <dt>Estado</dt>
        <dd>
          <EstadoBadge estado={peticion.estado} />
        </dd>

        <dt>Mensaje</dt>
        <dd className="peticion-detalle-cuerpo-mensaje">{peticion.cuerpo_mensaje}</dd>
      </dl>

      {peticion.estado === 'pendiente' ? (
        <section className="peticion-detalle-acciones">
          <label htmlFor="comentario-decision">Comentario (opcional)</label>
          <textarea
            id="comentario-decision"
            value={comentario}
            maxLength={COMENTARIO_MAX_LEN}
            onChange={(event) => setComentario(event.target.value)}
            disabled={enviando}
            rows={3}
          />
          <p className="peticion-detalle-contador">
            {comentario.length}/{COMENTARIO_MAX_LEN}
          </p>

          {errorAccion && (
            <p className="peticion-detalle-error-accion" role="alert">
              {errorAccion}
            </p>
          )}

          <div className="peticion-detalle-botones">
            <button type="button" onClick={() => void decidir('aprobar')} disabled={enviando}>
              Aprobar
            </button>
            <button type="button" onClick={() => void decidir('denegar')} disabled={enviando}>
              Denegar
            </button>
          </div>
        </section>
      ) : (
        peticion.decision && (
          <section className="peticion-detalle-auditoria">
            <h2>Decisión</h2>
            <dl className="peticion-detalle-campos">
              <dt>Decidido por</dt>
              <dd>{peticion.decision.decidido_por}</dd>

              <dt>Rol del decisor</dt>
              <dd>{ETIQUETA_ROL[peticion.decision.rol_decisor] ?? peticion.decision.rol_decisor}</dd>

              <dt>Fecha de decisión</dt>
              <dd>{formatearFecha(peticion.decision.fecha_decision)}</dd>

              {peticion.decision.comentario && (
                <>
                  <dt>Comentario</dt>
                  <dd>{peticion.decision.comentario}</dd>
                </>
              )}
            </dl>
          </section>
        )
      )}
    </article>
  )
}

export default PeticionDetalle
