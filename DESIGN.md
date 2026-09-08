# DESIGN.md — Portal de Autorización de Peticiones (Teams Forms)

Diseño técnico del sistema descrito en `REQUIREMENTS.md`. Este documento es
el contrato de diseño para el equipo (`designer` → `coder` → `reviewer`).
Cualquier ambigüedad de los requerimientos se resuelve aquí con una decisión
explícita y su justificación (ver sección 7).

## 1. Resumen y alcance

Herramienta pequeña de prueba con dos partes:

- **Backend en Rust**: genera peticiones simuladas (en memoria), expone una
  API REST para listarlas, verlas en detalle, aprobarlas o denegarlas, y
  simula autenticación por rol (IT / Administración).
- **Frontend en React**: portal web que consume esa API — login simulado,
  listado con filtros, detalle de cada petición y acciones de
  aprobar/denegar con registro visible de quién decidió y cuándo.

No hay integración real con Microsoft Teams, ni autenticación corporativa
real, ni notificaciones salientes (explícitamente fuera de alcance). El
diseño prioriza simplicidad y ejecución local sin dependencias externas
(sin base de datos externa, sin servicios de terceros).

## 2. Arquitectura general

```
┌─────────────────────┐        HTTP/JSON (REST)        ┌─────────────────────┐
│   frontend/ (React)  │  ───────────────────────────▶  │   backend/ (Rust)    │
│   Vite + TS          │  ◀───────────────────────────  │   Axum + Tokio       │
│   puerto 5173 (dev)  │      Authorization: Bearer      │   puerto 8080         │
└─────────────────────┘                                  │  Estado en memoria:   │
                                                            │  - peticiones        │
                                                            │  - sesiones          │
                                                            └─────────────────────┘
```

- Comunicación exclusivamente vía HTTP/JSON, sin SSR ni BFF intermedio.
- Sin base de datos externa: el estado vive en memoria del proceso backend
  (`Arc<RwLock<...>>`), sembrado con datos simulados al arrancar. Justificado
  en la sección 7.1.
- Sin cookies de sesión: el frontend guarda un token opaco en
  `localStorage` y lo envía como `Authorization: Bearer <token>` en cada
  request. Justificado en la sección 7.2.
- CORS restringido por configuración al origin del frontend (por defecto
  `http://localhost:5173`).

## 3. Backend (Rust)

### 3.1 Stack y dependencias

- `axum` — framework HTTP (async, sobre `tokio`), buen soporte de
  extractors para middleware de autenticación y manejo de errores tipado.
- `tokio` — runtime async.
- `serde` / `serde_json` — (de)serialización de modelos y payloads.
- `uuid` (v4) — identificadores de peticiones y tokens de sesión.
- `chrono` — fechas/horas (`DateTime<Utc>`), serializadas como ISO 8601.
- `rand` — generación de datos simulados.
- `tower-http` (`cors`, `trace`) — CORS y logging de requests.
- `thiserror` — tipos de error ergonómicos para `AppError`.
- `tracing` / `tracing-subscriber` — logging estructurado básico (útil para
  depurar localmente; no es un requerimiento pero es de bajo costo y buena
  práctica mínima).
- Dev-deps para tests de integración: `tokio::test`, `axum::body`,
  `tower::ServiceExt` (`oneshot`), o `reqwest` si se prefieren tests contra
  el servidor levantado. Se deja a criterio del `coder` cuál usar siempre
  que los criterios de aceptación de cada tarea queden cubiertos.

No se usa una base de datos (ni siquiera SQLite) — ver 7.1.

### 3.2 Estructura de carpetas — `backend/`

```
backend/
  Cargo.toml
  src/
    main.rs               # bootstrap: config, estado, router, servidor
    config.rs              # lectura de variables de entorno (puerto, CORS origin)
    state.rs                # AppState compartido (peticiones + sesiones)
    errors.rs                # AppError + impl IntoResponse (formato estándar)
    models/
      mod.rs
      peticion.rs            # Peticion, Severidad, EstadoPeticion
      auth.rs                  # Rol, Sesion, payloads de login
    domain/
      mod.rs
      peticion_generator.rs    # generación de datos simulados
      peticion_store.rs         # operaciones sobre el store en memoria
      auth_store.rs               # operaciones sobre el store de sesiones
    routes/
      mod.rs                       # construcción del Router y montaje de rutas
      health.rs                     # GET /api/health
      auth.rs                        # POST /api/auth/login, /api/auth/logout
      peticiones.rs                   # listar, detalle, aprobar, denegar, generar
    middleware/
      mod.rs
      auth_extractor.rs                # extractor AuthSession (valida Bearer token)
  tests/
    peticiones_test.rs
    auth_test.rs
```

Convención: `routes/` solo traduce HTTP ↔ dominio (parseo, status codes);
la lógica de negocio (generación, transición de estados, validaciones de
dominio) vive en `domain/`. Esto mantiene los handlers finos y testeables
por separado.

### 3.3 Modelo de datos

**`Peticion`**

| Campo             | Tipo                              | Notas |
|-------------------|------------------------------------|-------|
| `id`              | UUID (v4)                          | generado al crear |
| `tipo_peticion`   | string                              | ej. "Acceso a VPN", "Alta de usuario", "Compra de equipo" |
| `usuario`         | string                              | nombre del solicitante simulado |
| `email`           | string                              | email simulado, formato válido |
| `fecha_creacion`  | datetime (UTC, ISO 8601)           | generada al crear la petición |
| `cuerpo_mensaje`  | string                              | detalle/contenido de la petición |
| `severidad`       | enum: `baja` \| `media` \| `alta` \| `critica` | |
| `estado`          | enum: `pendiente` \| `aprobada` \| `denegada`  | por defecto `pendiente` |
| `decision`        | objeto opcional (null si `pendiente`) | ver abajo |

**`decision`** (presente cuando `estado != pendiente`):

| Campo             | Tipo      | Notas |
|-------------------|-----------|-------|
| `decidido_por`    | string    | nombre de la sesión que decidió (no del body) |
| `rol_decisor`     | enum: `it` \| `administracion` | rol de la sesión que decidió |
| `fecha_decision`  | datetime (UTC) | momento de la decisión |
| `comentario`      | string opcional, máx. 500 caracteres | comentario libre del decisor |

Se decide **no crear una entidad de auditoría separada**: la sección
"decision" embebida en la propia petición cubre el requerimiento de
"auditoría mínima" (quién y cuándo) sin añadir otra colección a
sincronizar. Ver 7.3.

**`Sesion`** (interna, no expuesta tal cual por la API):

| Campo        | Tipo    |
|--------------|---------|
| `token`      | string (UUID v4, opaco) |
| `nombre`     | string  |
| `rol`        | enum: `it` \| `administracion` |
| `creada_en`  | datetime (UTC) |

Las sesiones viven en memoria (`HashMap<token, Sesion>`) y no expiran
automáticamente en esta versión (no hay requerimiento de expiración; se
documenta como decisión en 7.4). `POST /api/auth/logout` elimina la sesión
explícitamente.

### 3.4 Almacenamiento en memoria

`AppState` contiene dos stores protegidos por `RwLock` (lecturas
concurrentes, escritura exclusiva), compartidos vía `Arc` entre todos los
handlers de Axum:

- `peticiones: RwLock<HashMap<Uuid, Peticion>>`
- `sesiones: RwLock<HashMap<String, Sesion>>`

Al arrancar el servidor (`main.rs`), se siembra el store de peticiones con
un lote inicial simulado (ver 3.5) para que el portal tenga datos desde el
primer `cargo run`, sin pasos manuales adicionales.

### 3.5 Generación de datos simulados

`domain::peticion_generator` expone una función que produce N peticiones
aleatorias combinando listas fijas embebidas en el código (no requieren red
ni archivos externos):

- `tipo_peticion`: lista fija de ~8-10 tipos plausibles (ej. "Acceso a VPN",
  "Restablecimiento de contraseña", "Alta de usuario", "Compra de equipo",
  "Acceso a carpeta compartida", "Instalación de software", "Cambio de
  permisos", "Solicitud de vacaciones").
- `usuario` / `email`: combinación de listas de nombres/apellidos fijos para
  generar pares nombre↔email coherentes (ej. `nombre.apellido@empresa.test`).
- `cuerpo_mensaje`: 1-2 frases de plantilla por tipo de petición, con datos
  interpolados, para que se lea razonable sin ser texto real de Teams.
- `severidad`: aleatoria uniforme entre las 4 opciones (o con pesos simples,
  a discreción del `coder`, sin que esto sea un requisito estricto).
- `fecha_creacion`: aleatoria dentro de una ventana razonable (ej. últimos
  15 días) para que el listado se vea realista, siempre `<= now`.
- `estado`: siempre `pendiente` al generarse (las peticiones nacen sin
  decisión).

Esta función se usa tanto para la siembra inicial al arrancar como para el
endpoint bajo demanda `POST /api/peticiones/generar` (sección 3.8).

### 3.6 Autenticación simulada

No hay autenticación corporativa real (fuera de alcance). Se simula así:

1. El usuario elige un **nombre** (texto libre, para fines de auditoría) y
   un **rol**: `it` o `administracion`.
2. `POST /api/auth/login` crea una `Sesion` en el store y devuelve un
   **token opaco** (UUID v4) — no es un JWT firmado ni contiene el rol
   codificado en el propio token; el token es solo una llave para buscar la
   sesión server-side.
3. El frontend guarda `{ token, nombre, rol }` en `localStorage` y envía el
   token como `Authorization: Bearer <token>` en cada request subsecuente.
4. Todas las rutas bajo `/api/peticiones*` requieren una sesión válida
   (ver 3.7). `/api/auth/login` y `/api/health` son públicas.

**Decisión de seguridad clave**: el backend **nunca** confía en un
`rol`/`nombre` enviado por el cliente en el body de una request protegida
para decidir permisos o para rellenar el campo de auditoría `decidido_por`
/ `rol_decisor`. Esos valores siempre se leen de la `Sesion` asociada al
token vigente, resuelta server-side por el extractor de autenticación. Esto
evita que un cliente falsifique quién aprobó/denegó una petición.

### 3.7 Autorización por rol

Los requerimientos indican que **tanto IT como Administración** pueden
aprobar o denegar peticiones — no hay una regla de "solo IT puede X, solo
Administración puede Y". Por lo tanto:

- La única regla de autorización real es: **debe existir una sesión válida
  (cualquiera de los dos roles)** para listar peticiones, ver detalle,
  aprobar, denegar o generar nuevas peticiones de prueba.
- El rol sí se registra y se muestra (para trazabilidad — "quién" incluye
  su rol), pero no restringe qué acción puede tomar.
- Esta decisión se documenta explícitamente en 7.5 para que quede claro que
  no es una omisión sino una lectura literal del requerimiento 4 de
  `REQUIREMENTS.md`. Si en el futuro se requiere una regla más fina (p. ej.
  "solo IT aprueba peticiones de tipo X"), el extractor de autenticación ya
  expone el rol de la sesión en cada handler, por lo que añadir esa regla
  sería un cambio localizado.

### 3.8 Contratos de API

Formato de error estándar en todas las respuestas 4xx/5xx:

```json
{
  "error": {
    "code": "no_autenticado",
    "message": "Se requiere un token de sesión válido."
  }
}
```

Códigos de error usados (no exhaustivo, el `coder` puede añadir más
siguiendo el mismo patrón `snake_case`): `no_autenticado`,
`peticion_no_encontrada`, `peticion_ya_decidida`, `datos_invalidos`,
`rol_invalido`.

---

**`GET /api/health`** — público.
Respuesta `200`: `{ "status": "ok" }`.

---

**`POST /api/auth/login`** — público.

Request:
```json
{ "nombre": "Ana Pérez", "rol": "it" }
```
Validaciones: `nombre` no vacío, longitud máx. 100; `rol` ∈ {`it`,
`administracion`} (si no, `400 datos_invalidos`).

Respuesta `201`:
```json
{ "token": "b3f1...", "nombre": "Ana Pérez", "rol": "it" }
```

---

**`POST /api/auth/logout`** — requiere sesión.
Elimina la sesión asociada al token del header `Authorization`.
Respuesta `204` sin body. Si el token ya no existe, también `204`
(operación idempotente).

---

**`GET /api/peticiones`** — requiere sesión.
Query params opcionales: `estado` ∈ {`pendiente`, `aprobada`, `denegada`}.

Respuesta `200`:
```json
{
  "peticiones": [
    {
      "id": "b3d2...",
      "tipo_peticion": "Acceso a VPN",
      "usuario": "Carlos Ruiz",
      "email": "carlos.ruiz@empresa.test",
      "fecha_creacion": "2026-09-01T14:30:00Z",
      "cuerpo_mensaje": "Solicito acceso VPN para trabajo remoto.",
      "severidad": "media",
      "estado": "pendiente",
      "decision": null
    }
  ]
}
```
`estado` inválido en el query → `400 datos_invalidos`.

---

**`GET /api/peticiones/:id`** — requiere sesión.
- `id` no es un UUID válido → `400 datos_invalidos`.
- `id` válido pero no existe → `404 peticion_no_encontrada`.
- Éxito → `200` con el objeto `Peticion` completo (mismo shape que arriba).

---

**`POST /api/peticiones/:id/aprobar`** y **`POST /api/peticiones/:id/denegar`**
— requieren sesión.

Request (body opcional):
```json
{ "comentario": "Aprobado, cumple política de acceso remoto." }
```
Validación: `comentario` opcional, máx. 500 caracteres si se envía.

Reglas:
- `id` inválido → `400`; no encontrado → `404`.
- Si `estado != pendiente` → `409 peticion_ya_decidida` (no se puede
  re-decidir una petición ya aprobada/denegada).
- Éxito → `200` con el objeto `Peticion` actualizado, incluyendo
  `decision.decidido_por` / `decision.rol_decisor` tomados de la sesión del
  token (nunca del body), y `decision.fecha_decision = now()`.

---

**`POST /api/peticiones/generar`** — requiere sesión.
Query param opcional `cantidad` (entero, default `5`, rango permitido
`1..=50`; fuera de rango → `400 datos_invalidos`).

Respuesta `201`:
```json
{ "creadas": 5 }
```
(el listado actualizado se obtiene con un `GET /api/peticiones` posterior,
para mantener el endpoint simple).

### 3.9 Manejo de errores

`AppError` (enum con `thiserror`) centraliza los casos: `NoAutenticado`,
`PeticionNoEncontrada`, `PeticionYaDecidida`, `DatosInvalidos(String)`.
Implementa `IntoResponse` mapeando cada variante a su status code y al
formato JSON estándar de la sección 3.8. Los handlers devuelven
`Result<T, AppError>`, evitando `unwrap()`/`panic!` sobre datos de entrada
o de negocio (los `panic!` solo son aceptables ante errores de programación
irrecuperables, no ante input de usuario).

### 3.10 CORS y configuración

Variables de entorno del backend (con valores por defecto razonables para
desarrollo local, documentadas en el README):

- `PORT` (default `8080`)
- `CORS_ALLOWED_ORIGIN` (default `http://localhost:5173`)
- `RUST_LOG` (opcional, para `tracing`, default `info`)

No hay secretos que gestionar en esta versión (no hay JWT firmado, no hay
credenciales de terceros). Si en el futuro se agregan, deben venir por
variable de entorno y nunca hardcodeados ni commiteados.

### 3.11 Validación de entradas

Toda entrada de usuario (login: `nombre`/`rol`; acciones: `comentario`;
query params: `estado`, `cantidad`) se valida en el borde (routes/domain)
antes de tocar el store, devolviendo `400 datos_invalidos` con un mensaje
claro. No se hace interpolación de strings hacia ningún motor de consultas
(no hay base de datos, así que no aplica inyección SQL), pero sí se limitan
longitudes de strings libres (`nombre`, `comentario`) para evitar payloads
abusivos.

## 4. Frontend (React)

### 4.1 Stack

- **Vite + React + TypeScript** — arranque rápido, tipado compartido con
  los contratos de la API (sección 3.8), sin necesidad de un framework full
  como Next.js (no hay SSR ni rutas de servidor que lo justifiquen).
- **react-router-dom** — dos vistas principales (`/login`,
  `/peticiones`) más el detalle; suficientemente simple para no requerir
  gestión de estado global compleja (Redux, etc.).
- **fetch** nativo envuelto en un cliente API propio (`api/client.ts`), sin
  librerías adicionales de data-fetching — el volumen de llamadas no lo
  justifica.
- CSS simple (módulos CSS o un único stylesheet global) — no se requiere un
  design system para esta herramienta de prueba.

### 4.2 Estructura de carpetas — `frontend/`

```
frontend/
  package.json
  vite.config.ts
  .env.example                # VITE_API_BASE_URL=http://localhost:8080
  src/
    main.tsx
    App.tsx                     # define rutas
    types/
      index.ts                    # Peticion, Severidad, EstadoPeticion, Rol, Decision...
    api/
      client.ts                    # fetch wrapper: base URL, headers, parseo de errores
      auth.ts                       # login/logout
      peticiones.ts                  # listar, detalle, aprobar, denegar, generar
    context/
      AuthContext.tsx                 # provee { token, nombre, rol, login, logout }
    components/
      ProtectedRoute.tsx                # redirige a /login si no hay sesión
      EstadoBadge.tsx                    # pendiente/aprobada/denegada
      SeveridadBadge.tsx                  # baja/media/alta/critica
      PeticionesTable.tsx                  # tabla/lista + filtro por estado
      PeticionDetalle.tsx                   # vista de detalle + acciones aprobar/denegar
      GenerarPeticionesButton.tsx             # utilidad de generación de datos de prueba
    pages/
      LoginPage.tsx
      PeticionesPage.tsx
      PeticionDetallePage.tsx
    styles/
      global.css
```

### 4.3 Enrutamiento y páginas

- `/login` — pública. Formulario: nombre + selector de rol (IT /
  Administración). Si ya hay sesión activa, redirige directo a
  `/peticiones`.
- `/peticiones` — protegida. Listado con filtro por estado y botón para
  generar peticiones de prueba.
- `/peticiones/:id` — protegida. Detalle completo + acciones
  aprobar/denegar (con comentario opcional) + info de auditoría si ya fue
  decidida.
- Cualquier ruta protegida sin sesión válida (o si el backend responde
  `401` a una llamada) redirige a `/login`.

### 4.4 Manejo de sesión

`AuthContext` mantiene `{ token, nombre, rol }` en estado de React,
inicializado desde `localStorage` al cargar la app, y expone `login()` /
`logout()`. El cliente API (`api/client.ts`) adjunta automáticamente el
header `Authorization` cuando hay token, y ante una respuesta `401`
limpia la sesión local y fuerza redirección a `/login` (evita que la UI
quede en un estado inconsistente si el backend se reinicia y pierde las
sesiones en memoria — caso esperado dado el almacenamiento en memoria del
backend, ver 7.4).

### 4.5 Cliente API

Wrapper único (`request<T>(path, options)`) que:
- Antepone `VITE_API_BASE_URL` a la ruta.
- Serializa/deserializa JSON.
- Adjunta `Authorization: Bearer <token>` si existe sesión.
- Ante status >= 400, parsea el cuerpo `{ error: { code, message } }` y
  lanza un error tipado (`ApiError`) con `code`, `message` y `status`, para
  que los componentes puedan mostrar mensajes específicos (ej. `409` →
  "Esta petición ya fue decidida").

### 4.6 Componentes principales

- **`PeticionesTable`**: consume `GET /api/peticiones` (con filtro
  `estado` opcional vía query param), muestra columnas: tipo, usuario,
  severidad, estado, fecha de creación; cada fila enlaza al detalle.
- **`PeticionDetalle`**: consume `GET /api/peticiones/:id`; si
  `estado === 'pendiente'`, muestra botones **Aprobar** / **Denegar** con
  un campo de comentario opcional; si ya fue decidida, muestra
  `decidido_por`, `rol_decisor` y `fecha_decision` en vez de los botones.
- **`GenerarPeticionesButton`**: input numérico simple (con límites
  reflejando los del backend, 1–50) + botón que llama a
  `POST /api/peticiones/generar` y refresca el listado.

### 4.7 UX de estados

Cada vista que depende de la red maneja explícitamente: estado de carga
(spinner/texto simple), estado vacío (sin peticiones / sin resultados para
el filtro elegido) y estado de error (mensaje legible tomado de `ApiError`,
con opción de reintentar). No se requiere un manejo más sofisticado
(retries automáticos, cache, etc.) para el alcance de esta prueba.

## 5. Flujo de extremo a extremo (ejemplo)

1. Usuario abre el frontend → `/login` → ingresa nombre "Ana Pérez", rol
   "IT" → `POST /api/auth/login` → token guardado en `localStorage`.
2. Redirige a `/peticiones` → `GET /api/peticiones` con
   `Authorization: Bearer <token>` → tabla poblada con datos simulados
   (sembrados al arrancar el backend, o generados manualmente con el botón
   de prueba).
3. Usuario entra al detalle de una petición `pendiente` → click en
   "Aprobar" con comentario opcional → `POST
   /api/peticiones/:id/aprobar` → backend registra
   `decidido_por: "Ana Pérez"`, `rol_decisor: "it"`,
   `fecha_decision: now()` → UI refleja el nuevo estado y la info de
   auditoría, botones deshabilitados.
4. Un segundo intento de aprobar/denegar la misma petición devuelve `409`
   y la UI muestra "Esta petición ya fue decidida".

## 6. Pruebas

- **Backend**: tests de integración por endpoint (`tests/`) cubriendo el
  camino feliz y los casos de error descritos en 3.8 (401, 400, 404, 409).
  Tests unitarios para el generador de datos (campos no vacíos, enums
  válidos, fechas `<= now`). `cargo clippy` sin warnings antes de abrir PR.
- **Frontend**: al menos smoke tests o pruebas de componentes clave si el
  `coder` decide incluir un runner de tests (no es un requisito duro dado
  el alcance); como mínimo, `npm run build` debe pasar sin errores de tipo
  en cada PR relevante.

## 7. Decisiones técnicas y justificación

### 7.1 Almacenamiento en memoria (sin base de datos)

Los requerimientos no piden persistencia entre reinicios del backend, y es
una "herramienta pequeña de prueba" cuyo dato de entrada además es
simulado/regenerable. Añadir SQLite/Postgres implicaría migraciones, un
ORM/query builder y más superficie para configurar — sobre-ingeniería para
este alcance. Se documenta como limitación conocida: **reiniciar el
backend borra todas las peticiones y sesiones**. Si en el futuro se
necesita persistencia, el store está aislado en `domain/`, por lo que
cambiar la implementación (a SQLite, por ejemplo) no debería tocar
`routes/`.

### 7.2 Sesión por token opaco en `localStorage` (no cookies, no JWT firmado)

No hay backend de autenticación real que emitir/validar JWTs firmados con
una clave secreta agregue valor — y gestionar esa clave (rotación, no
commitearla, etc.) sería complejidad innecesaria para un login simulado.
Un token opaco generado con UUID v4 y validado contra un store server-side
in-memory da la misma garantía práctica (el cliente no puede fabricar un
token válido) con una implementación mucho más simple. Se prefieren
headers `Authorization: Bearer` sobre cookies porque evita tener que
configurar `SameSite`/CSRF para un caso donde no hay sesión de navegador
persistente entre backend y frontend en dominios distintos durante
desarrollo local (puertos distintos).

### 7.3 Auditoría embebida en la petición, no como colección separada

El requerimiento pide "registro mínimo" de quién y cuándo. Embeber
`decision` en la propia `Peticion` cumple esto sin duplicar datos en una
segunda colección que habría que mantener sincronizada. Si más adelante se
necesita un historial de múltiples cambios de estado por petición, se
puede migrar a una lista de eventos sin romper el contrato actual (el
campo `decision` seguiría representando "la decisión vigente").

### 7.4 Sesiones sin expiración automática

No hay requerimiento de expiración de sesión. Añadir TTL, refresco de
token, etc. es complejidad no solicitada. Se documenta explícitamente el
trade-off: una sesión vive hasta logout explícito o hasta que el backend se
reinicia (lo cual, dado 7.1, también borra las sesiones). El frontend ya
maneja el caso "el backend ya no reconoce mi token" (sección 4.4)
degradando con gracia a la pantalla de login en vez de romperse.

### 7.5 Ambos roles (IT y Administración) pueden aprobar y denegar cualquier petición

`REQUIREMENTS.md` dice: *"La autorización puede darla una persona con rol
IT o una persona con rol Administración"* — se lee como "cualquiera de los
dos roles autoriza", no como una partición de qué tipo de petición
corresponde a cada rol (no hay campo ni regla en los requerimientos que
vincule `tipo_peticion` a un rol autorizador específico). Se elige la
interpretación más simple y explícita: autenticación (cualquier rol) es
suficiente para decidir; el rol solo se registra con fines de trazabilidad.
Ver 3.7 para el detalle de cómo extender esto si se requiere una regla más
fina en el futuro.

### 7.6 `rand` con listas fijas embebidas en vez de una librería tipo `fake`

Mantiene el árbol de dependencias pequeño y el resultado determinista de
inspeccionar (las listas son parte del código, no un dataset externo a
mantener). Es suficiente para el propósito de "datos simulados de
demostración".

## 8. Seguridad — checklist

- [x] Autorización por sesión válida en todas las rutas que mutan estado o
  exponen datos de peticiones (sección 3.6/3.7).
- [x] El backend nunca confía en `rol`/`nombre` provistos por el cliente
  para decisiones de autorización o campos de auditoría — siempre se
  derivan de la sesión server-side (sección 3.6).
- [x] Validación de todas las entradas de usuario con límites de longitud y
  enums cerrados (secciones 3.8, 3.11).
- [x] CORS restringido a un origin configurable (no `*`) (sección 3.10).
- [x] Sin secretos hardcodeados; configuración por variables de entorno con
  defaults de desarrollo documentados, nunca credenciales reales (sección
  3.10).
- [x] Manejo de errores sin `panic!`/`unwrap()` sobre input de usuario;
  respuestas de error uniformes que no filtran detalles internos (stack
  traces, rutas de archivo) (sección 3.9).

## 9. Cómo ejecutar localmente (resumen)

Detalle completo vive en el `README.md` del repo (tarea `T1` lo actualiza).
En resumen:

- Backend: `cd backend && cargo run` (sirve en `http://localhost:8080`).
- Frontend: `cd frontend && npm install && npm run dev` (sirve en
  `http://localhost:5173`, configurado vía `VITE_API_BASE_URL` para
  apuntar al backend).

## 10. Fuera de alcance (recordatorio)

Igual que en `REQUIREMENTS.md`: integración real con Teams, SSO/Azure AD,
notificaciones salientes. No se diseña nada para esos casos en esta
versión.
