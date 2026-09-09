import { Navigate, Route, Routes } from 'react-router-dom'
import LoginPage from './pages/LoginPage'
import PeticionesPage from './pages/PeticionesPage'

/**
 * Define las rutas de la aplicación (DESIGN.md 4.2/4.3). En esta tarea
 * (T8) las páginas son placeholders navegables; el contenido real
 * (autenticación, listado, protección de rutas) se agrega en T9-T12.
 */
function App() {
  return (
    <Routes>
      <Route path="/" element={<Navigate to="/login" replace />} />
      <Route path="/login" element={<LoginPage />} />
      <Route path="/peticiones" element={<PeticionesPage />} />
    </Routes>
  )
}

export default App
