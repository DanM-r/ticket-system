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

1. En una terminal: `cd backend && cargo run` (puerto `8080`).
2. En otra terminal: `cd frontend && npm run dev` (puerto `5173`).
3. Abrir `http://localhost:5173` en el navegador.

> Nota: el flujo completo de login, listado de peticiones y
> aprobar/denegar se documentará con más detalle en el README a medida que
> se implementen las tareas correspondientes (ver `TASKS.md`).
