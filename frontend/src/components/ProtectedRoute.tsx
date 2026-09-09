import type { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import { useAuth } from '../context/AuthContext'

/**
 * Envuelve una página que requiere sesión activa (DESIGN.md 4.3/4.4). Si no
 * hay sesión (ni en el estado de React ni en `localStorage`), redirige a
 * `/login` en vez de renderizar el contenido protegido.
 */
function ProtectedRoute({ children }: { children: ReactNode }) {
  const { sesion } = useAuth()

  if (!sesion) {
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}

export default ProtectedRoute
