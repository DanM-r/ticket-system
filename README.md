# ticket-system

Portal de Autorización de Peticiones (Teams Forms) — herramienta de prueba
que simula la recepción de respuestas de un formulario de Microsoft Teams y
expone un portal web para aprobarlas o denegarlas. Ver `REQUIREMENTS.md` y
`DESIGN.md` para el detalle funcional y de arquitectura.

El sistema tiene dos partes independientes:

- **`backend/`** — API REST en Rust (Axum + Tokio), estado en memoria.
- **`frontend/`** — Portal web en React (Vite + TypeScript).

## Requisitos previos

- [Rust](https://www.rust-lang.org/tools/install) (edición 2021) y `cargo`.
- [Node.js](https://nodejs.org/) 18+ y `npm`.

## Backend

```bash
cd backend
cargo build
cargo run
```

El servidor levanta por defecto en `http://localhost:8080`. Podés verificar
que está corriendo con:

```bash
curl http://localhost:8080/api/health
# {"status":"ok"}
```

### Variables de entorno del backend

| Variable              | Default                   | Descripción |
|-----------------------|----------------------------|-------------|
| `PORT`                 | `8080`                     | Puerto en el que escucha el servidor HTTP. |
| `CORS_ALLOWED_ORIGIN`   | `http://localhost:5173`     | Origin permitido por CORS (frontend en desarrollo). Es un valor exacto, nunca `*`. |
| `RUST_LOG`               | `info`                       | Nivel de logging estructurado (`tracing`). |

No hay archivo `.env` para el backend en esta etapa: las variables se pasan
por entorno, por ejemplo:

```bash
PORT=9090 cargo run
```

Para correr los tests y el linter:

```bash
cargo test
cargo clippy
```

## Frontend

```bash
cd frontend
npm install
npm run dev
```

El frontend levanta por defecto en `http://localhost:5173`.

### Variables de entorno del frontend

Copiá `frontend/.env.example` a `frontend/.env` (o `.env.local`) y ajustá
los valores según necesites:

```bash
cd frontend
cp .env.example .env
```

| Variable              | Default en `.env.example` | Descripción |
|-----------------------|-----------------------------|-------------|
| `VITE_API_BASE_URL`     | `http://localhost:8080`      | URL base del backend contra la que el frontend hace las llamadas HTTP. |

Para compilar en modo producción:

```bash
npm run build
```

## Correr backend y frontend juntos

1. En una terminal: `cd backend && cargo run` (puerto `8080` por defecto).
   Al arrancar, el servidor siembra automáticamente 15 peticiones simuladas
   en memoria — no hace falta ningún paso manual para tener datos de prueba.
2. En otra terminal: `cd frontend && npm install && npm run dev` (puerto
   `5173` por defecto).
3. Abrir `http://localhost:5173` en el navegador. La app redirige
   automáticamente a `/login`.

Si cambiás el puerto del backend (`PORT`) o el origin desde el que corre el
frontend, recordá mantener sincronizadas `CORS_ALLOWED_ORIGIN` (backend) y
`VITE_API_BASE_URL` (frontend) — ver las tablas de variables de entorno más
arriba. Si no coinciden, el navegador bloqueará las llamadas por CORS.

## Flujo de prueba end-to-end

Con el backend y el frontend corriendo (paso anterior), este es el flujo
completo que podés seguir en el navegador, sin necesidad de leer el código:

### 1. Iniciar sesión con cada rol

En `http://localhost:5173/login`, ingresá cualquier **nombre** (texto libre,
solo se usa para mostrarlo y para registrar auditoría — no hay contraseña,
es un login simulado) y elegí un **rol** en el selector:

- **IT**
- **Administración**

Ambos roles tienen exactamente los mismos permisos en este sistema (pueden
listar, ver detalle, generar peticiones de prueba, aprobar y denegar
cualquier petición) — la diferencia es solo de trazabilidad: el rol elegido
queda registrado en la auditoría de cada decisión (ver DESIGN.md 3.7/7.5).
Para probar ambos roles, podés loguearte una vez como IT, cerrar sesión, y
volver a loguearte como Administración (o usar una ventana de incógnito para
tener dos sesiones simultáneas en pestañas distintas).

Tras un login exitoso sos redirigido a `/peticiones`. Si recargás la página
con una sesión ya iniciada, la sesión persiste (se guarda en
`localStorage`) sin pedir login de nuevo.

### 2. Ver el listado de peticiones

En `/peticiones` vas a ver la tabla con las peticiones sembradas al arrancar
el backend (tipo, usuario, severidad, estado, fecha de creación). Podés
filtrar por estado (`Todas` / `Pendiente` / `Aprobada` / `Denegada`) con el
selector — el listado se actualiza sin recargar la página.

### 3. Generar peticiones de prueba adicionales

Arriba del listado hay un control "Generar peticiones de prueba": ingresá
una cantidad entre 1 y 50 y hacé clic en "Generar". El listado se refresca
automáticamente mostrando las peticiones nuevas (siempre se generan en
estado `pendiente`).

### 4. Ver el detalle de una petición

Hacé clic en cualquier fila del listado (el link está en la columna "Tipo")
para ir a `/peticiones/:id`. Ahí vas a ver todos los campos, incluido el
`cuerpo_mensaje` completo.

### 5. Aprobar o denegar

Si la petición está en estado `pendiente`, vas a ver los botones **Aprobar**
y **Denegar**, con un campo de comentario opcional (máx. 500 caracteres).
Al hacer clic en cualquiera de los dos, la vista se actualiza en el momento
con el nuevo estado — no hace falta recargar la página.

### 6. Ver la auditoría de una decisión

Una vez decidida (aprobada o denegada), esa misma vista de detalle deja de
mostrar los botones de acción y en su lugar muestra la sección "Decisión"
con:

- **Decidido por**: el nombre con el que inició sesión quien decidió.
- **Rol del decisor**: `IT` o `Administración`.
- **Fecha de decisión**.
- **Comentario** (si se ingresó alguno).

Estos datos siempre corresponden a la sesión que ejecutó la acción — el
backend nunca los toma de lo que el cliente envíe en el body (ver
`DESIGN.md` sección 3.6 y el checklist de seguridad, sección 8). Podés
confirmar esto volviendo al listado (filtrando por `Aprobada` o `Denegada`)
y reabriendo el detalle en cualquier momento: la auditoría queda visible
mientras el backend siga corriendo (se pierde si el proceso se reinicia,
dado que el estado vive solo en memoria — ver `DESIGN.md` 7.1).

### Casos límite ya cubiertos en la UI

- Intentar aprobar/denegar una petición que otra sesión ya decidió mientras
  la estabas viendo responde `409` y la UI lo muestra con un mensaje claro,
  refrescando el detalle para reflejar el estado real.
- Si el backend no responde, o responde `401` (por ejemplo, porque se
  reinició y perdió las sesiones en memoria), la sesión local se limpia y se
  redirige a `/login` automáticamente.
- Filtros sin resultados muestran un mensaje de "sin resultados" en vez de
  una tabla vacía sin contexto.
