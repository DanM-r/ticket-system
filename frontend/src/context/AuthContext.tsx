import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { login as apiLogin, logout as apiLogout } from '../api/auth'
import {
  guardarSesionAlmacenada,
  leerSesionAlmacenada,
  limpiarSesionAlmacenada,
  type SesionAlmacenada,
} from '../api/client'
import type { Rol } from '../types'

/**
 * Contexto de autenticación (DESIGN.md 4.4): mantiene `{ token, nombre, rol }`
 * en estado de React, inicializado desde `localStorage` al cargar la app, y
 * expone `iniciarSesion()` / `cerrarSesion()` para que las páginas no
 * manipulen `localStorage` ni el cliente API directamente.
 */
interface AuthContextValue {
  /** Sesión activa, o `null` si no hay ninguna (usuario no autenticado). */
  sesion: SesionAlmacenada | null
  /**
   * Llama a `POST /api/auth/login`, persiste la sesión resultante en
   * `localStorage` y actualiza el estado de React. Lanza [`ApiError`] si el
   * backend rechaza el login (ej. `400 datos_invalidos`); el llamador
   * decide cómo mostrar ese error.
   */
  iniciarSesion: (nombre: string, rol: Rol) => Promise<void>
  /**
   * Llama a `POST /api/auth/logout` y, sin importar el resultado, limpia la
   * sesión local (`localStorage` + estado de React) — el logout nunca debe
   * dejar a la persona atascada con una sesión que ya no puede usar.
   */
  cerrarSesion: () => Promise<void>
}

const AuthContext = createContext<AuthContextValue | undefined>(undefined)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [sesion, setSesion] = useState<SesionAlmacenada | null>(() => leerSesionAlmacenada())

  const iniciarSesion = useCallback(async (nombre: string, rol: Rol) => {
    const respuesta = await apiLogin({ nombre, rol })
    const nuevaSesion: SesionAlmacenada = {
      token: respuesta.token,
      nombre: respuesta.nombre,
      rol: respuesta.rol,
    }
    guardarSesionAlmacenada(nuevaSesion)
    setSesion(nuevaSesion)
  }, [])

  const cerrarSesion = useCallback(async () => {
    try {
      await apiLogout()
    } catch {
      // El logout es idempotente en el backend; si la llamada de red falla
      // igualmente limpiamos la sesión local para no dejar la UI atascada.
    } finally {
      limpiarSesionAlmacenada()
      setSesion(null)
    }
  }, [])

  const value = useMemo<AuthContextValue>(
    () => ({ sesion, iniciarSesion, cerrarSesion }),
    [sesion, iniciarSesion, cerrarSesion],
  )

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

/** Hook de acceso al [`AuthContext`]. Lanza si se usa fuera de un `<AuthProvider>`. */
export function useAuth(): AuthContextValue {
  const context = useContext(AuthContext)
  if (!context) {
    throw new Error('useAuth debe usarse dentro de un <AuthProvider>.')
  }
  return context
}
