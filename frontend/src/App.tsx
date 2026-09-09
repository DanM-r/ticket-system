import { Navigate, Route, Routes } from 'react-router-dom'
import ProtectedRoute from './components/ProtectedRoute'
import { AuthProvider } from './context/AuthContext'
import LoginPage from './pages/LoginPage'
import PeticionesPage from './pages/PeticionesPage'

/**
 * Define las rutas de la aplicación (DESIGN.md 4.2/4.3), envueltas en
 * [`AuthProvider`] para que toda la app comparta la misma sesión. `/login`
 * es pública; `/peticiones` está protegida por [`ProtectedRoute`] (T9), que
 * redirige a `/login` si no hay sesión activa.
 */
function App() {
  return (
    <AuthProvider>
      <Routes>
        <Route path="/" element={<Navigate to="/login" replace />} />
        <Route path="/login" element={<LoginPage />} />
        <Route
          path="/peticiones"
          element={
            <ProtectedRoute>
              <PeticionesPage />
            </ProtectedRoute>
          }
        />
      </Routes>
    </AuthProvider>
  )
}

export default App
