import { useState, type FormEvent } from 'react'
import { Navigate, useNavigate } from 'react-router-dom'
import { ApiError } from '../api/client'
import { useAuth } from '../context/AuthContext'
import type { Rol } from '../types'

/**
 * Página de login simulado (DESIGN.md 4.3/T9): formulario de `nombre` +
 * selector de `rol` (IT / Administración) que llama a
 * `POST /api/auth/login` a través de [`AuthContext`]. Si ya hay una sesión
 * activa, redirige directo a `/peticiones` en vez de mostrar el formulario.
 */
function LoginPage() {
  const { sesion, iniciarSesion } = useAuth()
  const navigate = useNavigate()

  const [nombre, setNombre] = useState('')
  const [rol, setRol] = useState<Rol>('it')
  const [error, setError] = useState<string | null>(null)
  const [enviando, setEnviando] = useState(false)

  if (sesion) {
    return <Navigate to="/peticiones" replace />
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setError(null)
    setEnviando(true)

    try {
      await iniciarSesion(nombre, rol)
      navigate('/peticiones', { replace: true })
    } catch (err) {
      setError(
        err instanceof ApiError
          ? err.message
          : 'No se pudo conectar con el servidor. Intenta de nuevo.',
      )
    } finally {
      setEnviando(false)
    }
  }

  return (
    <main className="login-page">
      <h1>Iniciar sesión</h1>
      <form className="login-form" onSubmit={handleSubmit}>
        <label htmlFor="nombre">Nombre</label>
        <input
          id="nombre"
          name="nombre"
          type="text"
          value={nombre}
          onChange={(event) => setNombre(event.target.value)}
          maxLength={100}
          required
          autoFocus
          disabled={enviando}
        />

        <label htmlFor="rol">Rol</label>
        <select
          id="rol"
          name="rol"
          value={rol}
          onChange={(event) => setRol(event.target.value as Rol)}
          disabled={enviando}
        >
          <option value="it">IT</option>
          <option value="administracion">Administración</option>
        </select>

        {error && (
          <p className="login-error" role="alert">
            {error}
          </p>
        )}

        <button type="submit" disabled={enviando}>
          {enviando ? 'Ingresando…' : 'Ingresar'}
        </button>
      </form>
    </main>
  )
}

export default LoginPage
