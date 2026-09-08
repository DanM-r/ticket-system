# TASKS.md — Tablero de tareas

Tablero de coordinación entre `designer`, `coder` y `reviewer`. Cada tarea
corresponde a un pull request independiente. Sigue el diseño en
`DESIGN.md` y los requerimientos en `REQUIREMENTS.md`. No cambies el
alcance de una tarea sin actualizar primero este archivo.

Estados posibles: `pendiente`, `en progreso`, `en revisión`,
`cambios solicitados`, `hecha`.

---

## T1 — Estructura inicial del repositorio (scaffolding backend + frontend)

- **Componente**: infra/repo
- **Descripción**: Crear el scaffold inicial de `backend/` (proyecto Cargo,
  binario Axum mínimo) y `frontend/` (proyecto Vite + React + TypeScript
  mínimo), siguiendo la estructura de carpetas de `DESIGN.md` secciones 3.2
  y 4.2 (pueden crearse solo los archivos base necesarios para compilar;
  las carpetas vacías de módulos futuros se agregan en tareas posteriores
  según se necesiten). Incluye:
  - `backend/Cargo.toml` con las dependencias listadas en DESIGN.md 3.1.
  - `backend/src/main.rs` levantando un servidor Axum con un único endpoint
    `GET /api/health` que devuelve `{"status":"ok"}`.
  - Lectura básica de `PORT` desde variable de entorno (default `8080`).
  - `frontend/` inicializado con Vite (template `react-ts`), compilando sin
    errores, con una pantalla placeholder simple.
  - `.gitignore` apropiado para Rust (`target/`) y Node (`node_modules/`,
    `dist/`) si no existe ya.
  - `frontend/.env.example` con `VITE_API_BASE_URL=http://localhost:8080`.
  - Actualizar el `README.md` raíz con instrucciones de cómo correr backend
    y frontend en local (comandos, puertos, variables de entorno).
- **Criterios de aceptación**:
  - `cd backend && cargo build` compila sin errores.
  - `cargo run` levanta el servidor y `GET /api/health` responde `200` con
    `{"status":"ok"}`.
  - `cd frontend && npm install && npm run build` compila sin errores.
  - README documenta ambos comandos de arranque y las variables de entorno
    relevantes de esta tarea.
- **Estado**: hecha
- **Dependencias**: ninguna

---

## T2 — Modelo de datos y generador de peticiones simuladas (backend)

- **Componente**: backend
- **Descripción**: Implementar los modelos `Peticion`, `Severidad`,
  `EstadoPeticion` (y el placeholder de `Decision`, aunque no se rellene
  hasta T6) según DESIGN.md 3.3, el `AppState` en memoria (DESIGN.md 3.4) y
  el generador de datos simulados (DESIGN.md 3.5) en `domain/peticion_generator.rs`.
  Sembrar el store con un lote inicial de peticiones (ej. 15) al arrancar
  el servidor en `main.rs`.
- **Criterios de aceptación**:
  - Tests unitarios del generador: cada `Peticion` generada tiene todos los
    campos requeridos no vacíos, `severidad` y `estado` con valores válidos
    del enum, `estado` inicial siempre `pendiente`, `fecha_creacion <= now`.
  - `cargo test` pasa; `cargo clippy` sin warnings nuevos.
  - Al arrancar el servidor con `cargo run`, el `AppState` queda sembrado
    con peticiones (verificable indirectamente una vez exista el endpoint
    de listado en T5, o mediante un test que invoque la función de siembra
    directamente).
- **Estado**: hecha
- **Dependencias**: T1

---

## T3 — Autenticación simulada: login y logout (backend)

- **Componente**: backend
- **Descripción**: Implementar `Rol`, `Sesion` (DESIGN.md 3.3/3.6) y el
  store de sesiones en memoria. Endpoints `POST /api/auth/login` y
  `POST /api/auth/logout` según los contratos de DESIGN.md 3.8, incluyendo
  validación de `nombre` (no vacío, máx. 100 caracteres) y `rol` (enum
  cerrado `it` | `administracion`).
- **Criterios de aceptación**:
  - `POST /api/auth/login` con `nombre` y `rol` válidos devuelve `201` con
    `{ token, nombre, rol }`.
  - `POST /api/auth/login` con `nombre` vacío o `rol` desconocido devuelve
    `400` con el formato de error estándar (`datos_invalidos`).
  - `POST /api/auth/logout` con un token existente lo invalida (una
    petición protegida posterior con ese token deja de funcionar, una vez
    exista el extractor de T4 — puede validarse en esta tarea inspeccionando
    directamente el store, o dejarse cubierto por el test de T4).
  - `POST /api/auth/logout` sin token o con token inexistente responde
    `204` (idempotente), sin error.
  - Tests de integración cubriendo los casos anteriores. `cargo clippy` sin
    warnings nuevos.
- **Estado**: hecha
- **Dependencias**: T1

---

## T4 — Middleware/extractor de autorización por sesión (backend)

- **Componente**: backend
- **Descripción**: Implementar el extractor `AuthSession` (DESIGN.md 3.6)
  que lee el header `Authorization: Bearer <token>`, busca la sesión en el
  store y, si es válida, la inyecta en el handler (exponiendo `nombre` y
  `rol`); si no es válida o falta, responde `401 no_autenticado` con el
  formato de error estándar. Aplicarlo como ejemplo mínimo a un endpoint de
  prueba (o adelantar su uso en una ruta protegida simple) para demostrar
  que funciona; su aplicación completa a todos los endpoints de peticiones
  ocurre en T5-T7 según corresponda.
- **Criterios de aceptación**:
  - Request a una ruta protegida sin header `Authorization` → `401` con
    `error.code = "no_autenticado"`.
  - Request con un token que no existe en el store → `401` igual que arriba.
  - Request con un token válido → el handler recibe `nombre` y `rol`
    correctos de la sesión.
  - Tests de integración cubriendo los tres casos. `cargo clippy` sin
    warnings nuevos.
- **Estado**: pendiente
- **Dependencias**: T3

---

## T5 — Endpoints de listado y detalle de peticiones (backend)

- **Componente**: backend
- **Descripción**: Implementar `GET /api/peticiones` (con filtro opcional
  `?estado=`) y `GET /api/peticiones/:id`, protegidos por el extractor de
  T4, según los contratos de DESIGN.md 3.8.
- **Criterios de aceptación**:
  - Sin sesión válida, ambos endpoints devuelven `401`.
  - `GET /api/peticiones` con sesión válida devuelve el array completo de
    peticiones sembradas, con el shape JSON definido en DESIGN.md.
  - `GET /api/peticiones?estado=pendiente` (y `aprobada`, `denegada`)
    filtra correctamente; un valor de `estado` fuera del enum devuelve
    `400 datos_invalidos`.
  - `GET /api/peticiones/:id` con un id existente devuelve `200` y el
    objeto completo; con un id bien formado pero inexistente devuelve
    `404 peticion_no_encontrada`; con un id mal formado (no UUID) devuelve
    `400 datos_invalidos`.
  - Tests de integración cubriendo todos los casos anteriores. `cargo
    clippy` sin warnings nuevos.
- **Estado**: pendiente
- **Dependencias**: T2, T4

---

## T6 — Endpoints de aprobar/denegar petición (backend)

- **Componente**: backend
- **Descripción**: Implementar `POST /api/peticiones/:id/aprobar` y
  `POST /api/peticiones/:id/denegar` según DESIGN.md 3.8, incluyendo el
  campo `decision` embebido (DESIGN.md 3.3). El `decidido_por` y
  `rol_decisor` deben derivarse **siempre** de la sesión resuelta por el
  extractor de T4, nunca de un valor enviado en el body. `comentario` es
  opcional, máx. 500 caracteres.
- **Criterios de aceptación**:
  - Sin sesión válida, ambos endpoints devuelven `401`.
  - Aprobar/denegar una petición en estado `pendiente` con sesión válida
    devuelve `200`, con `estado` actualizado y `decision.decidido_por` /
    `decision.rol_decisor` iguales a los datos de la sesión usada (no a
    ningún valor del body, aunque el test envíe un `decidido_por` o `rol`
    falso en el body — debe ser ignorado).
  - Intentar decidir una petición que ya tiene `estado != pendiente`
    devuelve `409 peticion_ya_decidida` y no modifica la decisión existente.
  - Un `comentario` de más de 500 caracteres devuelve `400 datos_invalidos`.
  - id mal formado → `400`; id inexistente → `404`.
  - Tests de integración cubriendo todos los casos anteriores, incluyendo
    explícitamente el caso de "el body no puede falsificar el decisor".
    `cargo clippy` sin warnings nuevos.
- **Estado**: pendiente
- **Dependencias**: T5

---

## T7 — Generación de peticiones bajo demanda, CORS y errores estandarizados (backend)

- **Componente**: backend
- **Descripción**: Implementar `POST /api/peticiones/generar` (protegido,
  `cantidad` por query param, default 5, rango 1-50) reutilizando el
  generador de T2. Configurar `tower-http` CORS restringido al origin
  definido por la variable de entorno `CORS_ALLOWED_ORIGIN` (default
  `http://localhost:5173`, ver DESIGN.md 3.10). Revisar y unificar el
  manejo de errores (`AppError` + `IntoResponse`, DESIGN.md 3.9) para que
  todos los endpoints existentes (auth, peticiones) respondan errores con
  el mismo formato JSON, ajustando lo que se haya implementado de forma
  ad-hoc en tareas anteriores.
- **Criterios de aceptación**:
  - `POST /api/peticiones/generar` sin sesión → `401`.
  - `POST /api/peticiones/generar` con sesión válida y sin `cantidad` crea
    5 peticiones nuevas (verificable con un `GET /api/peticiones`
    posterior que muestre más elementos) y responde `201` con
    `{ "creadas": 5 }`.
  - `cantidad=0`, `cantidad=51` u otro valor fuera de rango devuelve `400
    datos_invalidos` y no crea peticiones.
  - Todas las respuestas de error de la API (auth, peticiones, generar)
    siguen exactamente el mismo formato `{ "error": { "code", "message" } }`.
  - CORS configurado: una request preflight/simulada desde el origin
    configurado no es bloqueada (verificable inspeccionando headers de
    respuesta `Access-Control-Allow-Origin` en un test o documentando la
    verificación manual en la descripción del PR).
  - `cargo test` y `cargo clippy` sin warnings nuevos.
- **Estado**: pendiente
- **Dependencias**: T6

---

## T8 — Scaffolding frontend: cliente API y tipos compartidos

- **Componente**: frontend
- **Descripción**: Sobre el scaffold de T1, construir la estructura de
  carpetas de DESIGN.md 4.2 (`api/`, `types/`, `context/`, `components/`,
  `pages/`, `styles/`), `react-router-dom` instalado y configurado con
  rutas placeholder (`/login`, `/peticiones`), `types/index.ts` con las
  interfaces TypeScript que reflejan el modelo de datos y los contratos de
  API definitivos del backend (DESIGN.md 3.3/3.8), y `api/client.ts` con el
  wrapper de `fetch` descrito en DESIGN.md 4.5 (base URL desde
  `VITE_API_BASE_URL`, adjunta `Authorization` si hay token, parsea errores
  al formato `ApiError`).
- **Criterios de aceptación**:
  - `npm run build` compila sin errores de tipo.
  - `api/client.ts` expone una función genérica reutilizable y al menos una
    función concreta que llama a `GET /api/health` como prueba de
    conexión con el backend.
  - `types/index.ts` incluye `Peticion`, `Severidad`, `EstadoPeticion`,
    `Rol`, `Decision`, y los tipos de request/response de los endpoints de
    auth.
  - Rutas `/login` y `/peticiones` navegables (aunque el contenido sea
    placeholder en esta tarea).
- **Estado**: pendiente
- **Dependencias**: T7

---

## T9 — Login simulado y manejo de sesión (frontend)

- **Componente**: frontend
- **Descripción**: Implementar `AuthContext` (DESIGN.md 4.4), `LoginPage`
  con formulario de `nombre` + selector de `rol` (IT / Administración) que
  llama a `POST /api/auth/login`, persiste `{ token, nombre, rol }` en
  `localStorage`, y `ProtectedRoute` que redirige a `/login` si no hay
  sesión. Incluir botón/acción de logout que llame a `POST
  /api/auth/logout` y limpie la sesión local.
- **Criterios de aceptación**:
  - Un usuario puede loguearse eligiendo IT o Administración; tras login
    exitoso es redirigido a `/peticiones`.
  - Si el login falla (backend responde `400`), se muestra un mensaje de
    error legible en el formulario sin romper la app.
  - Recargar la página con una sesión ya iniciada mantiene la sesión
    (leída de `localStorage`) sin pedir login de nuevo.
  - Intentar acceder a `/peticiones` sin sesión redirige a `/login`.
  - Logout limpia `localStorage` y redirige a `/login`.
- **Estado**: pendiente
- **Dependencias**: T8, T3

---

## T10 — Listado de peticiones con filtro por estado (frontend)

- **Componente**: frontend
- **Descripción**: Implementar `PeticionesPage` + `PeticionesTable`
  (DESIGN.md 4.6) consumiendo `GET /api/peticiones`, con filtro por
  `estado` (`pendiente`/`aprobada`/`denegada`/todas), badges de estado y
  severidad (`EstadoBadge`, `SeveridadBadge`), y manejo explícito de
  estados de carga, vacío y error (DESIGN.md 4.7). Si una llamada responde
  `401`, la sesión local se limpia y se redirige a `/login` (DESIGN.md 4.4).
- **Criterios de aceptación**:
  - Al entrar autenticado a `/peticiones` se ve el listado real obtenido
    del backend (no datos hardcodeados en el frontend).
  - Cambiar el filtro de estado actualiza el listado sin recargar la
    página completa.
  - Si no hay peticiones para el filtro elegido, se muestra un mensaje de
    "sin resultados" en vez de una tabla vacía sin contexto.
  - Si el backend no responde o responde `401`, se muestra manejo de error
    apropiado (mensaje o redirección a login, según corresponda).
  - `npm run build` compila sin errores de tipo.
- **Estado**: pendiente
- **Dependencias**: T9

---

## T11 — Detalle de petición y acciones aprobar/denegar (frontend)

- **Componente**: frontend
- **Descripción**: Implementar `PeticionDetallePage` + `PeticionDetalle`
  (DESIGN.md 4.6) consumiendo `GET /api/peticiones/:id`, mostrando todos
  los campos incluido `cuerpo_mensaje` completo. Si `estado === pendiente`,
  mostrar botones **Aprobar**/**Denegar** con campo de comentario opcional
  que llaman a los endpoints correspondientes; si ya fue decidida, mostrar
  `decidido_por`, `rol_decisor` y `fecha_decision` en vez de los botones.
- **Criterios de aceptación**:
  - Aprobar o denegar desde la UI actualiza el estado mostrado (sin
    recargar manualmente la página) y refleja la info de auditoría
    devuelta por el backend.
  - Una petición ya decidida no muestra los botones de acción; en su lugar
    muestra quién decidió y cuándo.
  - Si el backend responde `409` (ya decidida, ej. por una decisión
    concurrente desde otra sesión) o `400` (comentario inválido), se
    muestra un mensaje de error legible sin romper la vista.
  - `npm run build` compila sin errores de tipo.
- **Estado**: pendiente
- **Dependencias**: T10

---

## T12 — Generación de peticiones de prueba desde la UI (frontend)

- **Componente**: frontend
- **Descripción**: Implementar `GenerarPeticionesButton` (DESIGN.md 4.6):
  input numérico (límites 1-50, reflejando el backend) + botón que llama a
  `POST /api/peticiones/generar` y refresca el listado de `PeticionesPage`
  tras la respuesta exitosa. Visible solo si hay sesión activa.
- **Criterios de aceptación**:
  - Con sesión activa, el botón es visible en `/peticiones`.
  - Ingresar una cantidad fuera de 1-50 se valida en el cliente (se
    deshabilita el botón o se muestra un mensaje) sin necesidad de esperar
    la respuesta del backend.
  - Al generar exitosamente, el listado se actualiza mostrando las nuevas
    peticiones sin recargar la página completa.
  - `npm run build` compila sin errores de tipo.
- **Estado**: pendiente
- **Dependencias**: T11

---

## T13 — Documentación final y pulido end-to-end

- **Componente**: infra/repo
- **Descripción**: Actualizar `README.md` con instrucciones completas de
  ejecución local end-to-end (levantar backend y frontend juntos, variables
  de entorno de ambos, cómo loguearse con cada rol, cómo generar peticiones
  de prueba, cómo aprobar/denegar). Revisar en frontend y backend que no
  queden estados de carga/vacío/error sin manejar en las vistas principales,
  y confirmar (documentándolo en la descripción del PR) el checklist de
  seguridad de DESIGN.md sección 8: sin secretos hardcodeados, CORS
  restringido, validaciones de entrada activas, autorización por sesión
  aplicada consistentemente en todos los endpoints.
- **Criterios de aceptación**:
  - README permite a un desarrollador nuevo, sin contexto previo, levantar
    el sistema completo localmente y probar el flujo de login → listado →
    aprobar/denegar → ver auditoría, siguiendo solo el README.
  - No quedan pantallas del frontend sin manejo explícito de carga/vacío/
    error para las llamadas a la API usadas en el flujo principal.
  - La descripción del PR incluye una revisión explícita del checklist de
    seguridad de DESIGN.md sección 8, ítem por ítem.
- **Estado**: pendiente
- **Dependencias**: T12

---

## Notas para el equipo

- Cualquier tarea nueva que surja durante el desarrollo (bugs fuera de
  alcance detectados, mejoras futuras) debe agregarse a este archivo como
  una nueva tarea (`T14`, `T15`, ...) en vez de mezclarse en el PR de una
  tarea existente.
- Si una tarea resulta demasiado grande al implementarla, el `coder` puede
  proponer partirla — pero debe reflejar esa división aquí antes de abrir
  los PRs correspondientes, para mantener este tablero como fuente de
  verdad.
