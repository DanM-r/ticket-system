# Requerimientos — Portal de Autorización de Peticiones (Teams Forms)

## Contexto

Herramienta pequeña de prueba que simula la recepción de respuestas de un
formulario de Microsoft Teams en un canal, y expone un portal web donde
personas autorizadas (de IT y de Administración) pueden **aprobar** o
**denegar** cada petición.

De momento **no hay integración real con Microsoft Teams**: los datos de las
peticiones se generan de forma aleatoria/simulada por el backend.

## Requerimientos funcionales

1. **Generación de datos simulados de peticiones**, con al menos estos campos:
   - `tipo_peticion` (tipo de petición, texto/categoría)
   - `usuario` (nombre del solicitante)
   - `email` (correo del solicitante)
   - `fecha_creacion` (fecha/hora de creación de la petición)
   - `cuerpo_mensaje` (contenido/detalle de la petición)
   - `severidad` (nivel de severidad/prioridad)
2. **Portal web** que lista las peticiones generadas.
3. Cada petición puede ser **aprobada** o **denegada** desde el portal.
4. La autorización puede darla una persona con rol **IT** o una persona con
   rol **Administración** (dos roles autorizadores distintos).
5. Debe quedar registro de quién autorizó/denegó y cuándo (auditoría mínima).

## Requerimientos no funcionales

- **Backend**: Rust.
- **Frontend**: React.
- Alcance de **herramienta pequeña de prueba** — priorizar simplicidad sobre
  robustez de producción (sin sobre-ingeniería), pero manteniendo buenas
  prácticas de seguridad básicas (autorización por rol, validación de
  entradas, no exponer secretos).
- El sistema debe poder ejecutarse y probarse localmente por un desarrollador.

## Fuera de alcance (por ahora)

- Integración real con la API/Webhooks de Microsoft Teams.
- Autenticación corporativa real (SSO/Azure AD) — se puede simular con un
  mecanismo simple de login/roles para efectos de la prueba.
- Notificaciones salientes (email, Teams, etc.) tras aprobar/denegar.

## Rol del agente `designer`

A partir de estos requerimientos, el agente `designer` debe producir:

1. `DESIGN.md` — diseño de arquitectura (backend, frontend, modelo de datos,
   API, manejo de roles/autorización, decisiones técnicas y su justificación).
2. `TASKS.md` — tablero de tareas de desarrollo, como la herramienta de
   gestión de proyecto compartida entre los agentes `coder` y `reviewer`.
