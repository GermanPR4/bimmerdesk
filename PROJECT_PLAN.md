# BMW Toolbox — Plan de Proyecto

**Documento vivo.** Fuente de verdad del proyecto. Se actualiza en cada fase: progreso, decisiones, tareas nuevas. Las decisiones importantes nunca se eliminan, solo se marcan como superadas si cambian.

Última actualización: 2026-08-01 — Plan v3: incorpora la investigación técnica de `docs/research/ECU_COMMUNICATION_RESEARCH.md` (Fase 0B insertada, norma "ninguna fase depende exclusivamente del vehículo real", Security Access como V1 parcial). Plan de arquitectura y gobernanza aprobado por el usuario. **Rige junto a `PROJECT_PRINCIPLES.md`** — ante conflicto entre un detalle de este plan y un principio de ese documento, gana el principio. Siguiente paso: inicio de Fase 0.

---

## 1. Visión del producto

BMW Toolbox es una aplicación de escritorio profesional de diagnóstico, monitorización y mantenimiento de vehículos, construida como núcleo (core) + módulos, inicialmente centrada en BMW pero con la marca, el vehículo y el protocolo como conceptos desacoplados desde el diseño.

**V1 es de solo lectura.** No escribe absolutamente nada en ninguna centralita. Lee, visualiza, diagnostica (lectura de errores), guarda históricos y genera informes. Cualquier función de escritura (borrado de DTCs, codificación, reprogramación ECU) queda como módulo futuro, presente en la arquitectura pero no implementado ni accesible.

**Público objetivo:** inicialmente el propio desarrollador/usuario con un BMW F-Series, con diseño pensado para escalar a producto comercial multi-usuario y multi-marca en el futuro.

### 1.1 MVP de V1 — alcance mínimo, sin excepciones

La arquitectura está preparada para mucho (módulos, IA, multimarca, permisos...) pero **eso es capacidad de crecer, no trabajo pendiente de V1**. El MVP que hay que construir primero es exactamente esto y nada más:

1. Detectar el vehículo.
2. Leer VIN.
3. Leer información del vehículo.
4. Leer DTCs.
5. Mostrar Live Data.
6. Guardar historial.
7. Generar informe.

Nada de esta lista se amplía sin aprobación explícita. Cualquier idea nueva que surja durante el desarrollo (IA, plugins dinámicos, marketplace, más marcas, coding, flash, cloud...) se anota en la sección "Post-V1" del roadmap (sección 6) — no se implementa antes de que este MVP funcione de punta a punta.

**Objetivo medible de V1 (criterio de "terminado"):** *BMW Toolbox V1 estará terminada cuando un usuario conecte un BMW F20 mediante ENET, la aplicación detecte automáticamente el vehículo, lea correctamente VIN, módulos principales, DTCs y datos en tiempo real, permita guardar el historial y generar un informe, todo ello sin modificar ningún parámetro del vehículo.*

Este objetivo es el que decide si una fase se da por cerrada — no "se ve bien" ni "en principio funciona".

---

## 2. Decisiones técnicas confirmadas

Estas decisiones ya están aprobadas y no deben revisarse sin justificación fuerte:

| Área | Decisión | Justificación |
|---|---|---|
| Shell de escritorio | **Tauri 2.x** (Rust + WebView nativo) | Binarios ligeros (~10-20MB vs Electron ~150MB+), consumo RAM muy inferior, Rust da control preciso de timing y acceso seguro a bajo nivel (serie/TCP/USB), crítico para protocolos ECU con ventanas de tiempo en milisegundos. |
| Frontend | **React + TypeScript + Vite** | El usuario ya domina React. Ecosistema maduro para dashboards y gráficas en tiempo real. Reutilizable si en el futuro se porta a web. |
| Comunicación UI ↔ Core | **Tauri commands (`invoke`) + eventos (`emit`/`listen`)** | Los comandos cubren peticiones request/response. Los eventos cubren streaming continuo (RPM, sensores en vivo) sin polling. |
| Base de datos | **SQLite embebido** (`rusqlite` o `sqlx`) con migraciones versionadas | Cero configuración, transaccional, un único archivo `.db` portable. |
| Plataforma V1 | **Windows only** | Reduce alcance inicial. Tauri permite añadir macOS/Linux después sin reescritura mayor. |
| Cuentas / sync en la nube | **Ninguna en V1. Todo 100% local** | Sin backend propio, sin coste de infraestructura, sin superficie de ataque adicional. |
| Capa de comunicación ECU | **Implementación propia en Rust, sin librerías de terceros** | No existe ecosistema Rust maduro equivalente a herramientas de diagnóstico existentes. Capa propia en 3 niveles: `Transport` → `Protocol` → `Diagnostic Service`. |
| Prioridad de interfaz física | **ENET (BMW F-Series) primero**, abstracción para ELM327 y K+DCAN después | El usuario tiene un F-Series. ENET (Ethernet/DoIP) es más simple de implementar de forma fiable que K-line. |
| Integración IA | **Patrón provider/plugin, desactivada por defecto** | La app debe funcionar 100% sin IA. V1 solo define la interfaz; sin proveedor real conectado. |
| Empaquetado | **Bundler nativo de Tauri** (`.msi`) | Integrado en el propio framework. |
| **Arquitectura de extensión** | **Módulos registrados en tiempo de compilación (no plugins dinámicos)** | Ver justificación detallada en sección 3.2. Rust no tiene ABI estable — carga dinámica de `.dll` en runtime (`libloading`) es frágil entre versiones de compilador. Se obtiene el mismo desacoplamiento con un `ModuleRegistry` estático, sin ese riesgo. Plugins realmente dinámicos quedan reservados a una futura capa WASM (post-V1). |
| **Comunicación entre módulos** | **Command Bus interno, tipado (no genérico por strings)** | Con cientos de acciones a medio plazo, se necesita desacoplar quién dispara una acción de quién la ejecuta. Se mantiene tipado fuerte (enums/structs Rust) para no perder las garantías del compilador. |
| **Permisos entre módulos** | **Manifest declarativo de capacidades, validado en registro** | Cada módulo declara qué necesita (`needs_ecu`, `needs_sqlite`, `needs_network`, `needs_filesystem`, `needs_ai`, `needs_usb`). El núcleo lo valida al arrancar. No es sandboxing en proceso aislado (eso sería sobreingeniería para V1 solo-dev) — es control declarativo y auditable. |

---

## 3. Arquitectura general

### 3.1 Vista de capas

```
┌──────────────────────────────────────────────────────────────┐
│  Frontend (React + TypeScript)                                 │
│  Vistas / Componentes / Estado / Gráficas                      │
└───────────────────────────┬──────────────────▲─────────────────┘
                │ invoke (comandos)   │ listen (eventos)
┌───────────────▼──────────────────────┴───────────────────────┐
│  Tauri Command Layer (Rust)                                    │
│  Expone comandos tipados al frontend, emite eventos             │
└───────────────────────────┬────────────────────────────────────┘
┌───────────────────────────▼────────────────────────────────────┐
│  Command Bus (tipado)                                           │
│  Enruta comandos/queries a los módulos correspondientes          │
└───────────────────────────┬────────────────────────────────────┘
┌───────────────────────────▼────────────────────────────────────┐
│  Module Registry                                                 │
│  Vehicle Info · Live Data · Diagnostics · Maintenance · Reports  │
│  AI · Coding (stub) · Flash (stub)                               │
│  Cada módulo: manifest de permisos + API pública propia          │
└──────────────┬─────────────────────────────────┬────────────────┘
               │                                  │
┌──────────────▼───────────────┐   ┌──────────────▼───────────────┐
│  ECU Communication Stack      │   │  Persistence Layer             │
│  Protocol (UDS/KWP2000/OBD2)  │   │  SQLite + Repositories          │
│  Transport (ENET/K+DCAN/ELM327)│  │  (un repositorio por módulo,    │
│  Manufacturer → Vehicle → ... │   │   sin acceso cruzado entre ellos)│
└────────────────────────────--─┘   └───────────────────────────────┘
```

### 3.2 Arquitectura de módulos (revisión clave del diseño)

Todo lo que antes eran "features" del frontend y "servicios" sueltos del backend pasa a ser un **módulo** con forma uniforme, en ambos lados:

- Cada módulo backend implementa un trait `Module`:
  ```rust
  trait Module {
      fn id(&self) -> &'static str;
      fn manifest(&self) -> ModuleManifest;   // permisos declarados, versión, dependencias de otros módulos
      fn register(&self, bus: &mut CommandBus, registry: &mut ApiRegistry);
      fn on_startup(&self, ctx: &AppContext) -> Result<(), ModuleError>;
  }
  ```
- `ModuleRegistry` los recopila todos al arrancar (registro en tiempo de compilación — se listan explícitamente en `main.rs`, no se descubren en runtime). Añadir un módulo nuevo = crear su carpeta implementando `Module` + añadir una línea a la lista de módulos activos.
- Cada módulo expone una **API pública propia** (funciones/tipos que otros módulos pueden llamar) y mantiene su estado de persistencia **privado**: ningún módulo hace queries directas a la tabla de otro módulo, todo pasa por la API pública o por el Command Bus. Esto es la regla de frontera más importante del sistema — se revisa en cada PR.

**Por qué no plugins dinámicos verdaderos en V1:** cargar código Rust en runtime (`.dll`/`.so` vía `libloading`) exige ABI estable entre el host y el plugin, algo que Rust no garantiza entre versiones de compilador ni siquiera entre distintos flags de compilación. Es una fuente conocida de crashes difíciles de depurar. La alternativa segura para "plugins verdaderos" instalables sin recompilar es un runtime WASM (ej. Wasmtime), donde cada módulo compila a un sandbox con interfaz bien definida (WASI/Component Model). Se deja anotado como evolución post-V1; no se persigue en V1 porque el coste de infraestructura no está justificado con un solo desarrollador y sin marketplace real de terceros todavía.

### 3.3 Jerarquía de dominio: Manufacturer → Vehicle → Protocol → Modules

Aunque V1 solo usa BMW, el modelo de dominio se diseña con esta jerarquía desde el principio:

```
Manufacturer (BMW, futuro: Audi, Mercedes...)
    └─ VehiclePlatform (F-Series, E-Series...)
         └─ Vehicle (instancia concreta: VIN, ficha completa)
              └─ Protocol activo (UDS para F-Series)
                   └─ Módulos que consumen ese protocolo (Diagnostics, Live Data...)
```

Esto evita acoplar "BMW" al núcleo: el núcleo solo conoce `Manufacturer` y `Protocol` como conceptos abstractos; `Bmw` es una implementación concreta registrada igual que un módulo.

### 3.4 Ficha de vehículo (Vehicle Profile)

Se amplía el concepto de "vehículo detectado" a una ficha completa persistida:

- Identificación: VIN, modelo, motor, caja, año, versión de software.
- Histórico: kilometraje a lo largo del tiempo, sesiones de diagnóstico, sesiones de datos en vivo.
- Mantenimiento: aceite, filtros, pastillas, discos, ITV, neumáticos (módulo Maintenance).
- Extras: fotos, notas libres, modificaciones/preparación, especificación de ruedas, batería (tipo, fecha instalación — relevante en eléctricos/híbridos BMW).
- Cada bloque de la ficha pertenece a su módulo correspondiente (fotos/notas → módulo Vehicle Info; mantenimiento → módulo Maintenance); la ficha es una vista compuesta, no una tabla monolítica.

### 3.5 Flujo de datos — ejemplo: lectura de códigos de error (DTCs)

1. Usuario pulsa "Escanear errores" en la UI (React).
2. React llama `invoke('scan_dtcs', { vehicleId })`.
3. El comando Rust publica un `ScanDtcsCommand` en el Command Bus.
4. El bus lo enruta al módulo `Diagnostics`, que pide al `Protocol` activo (`UdsProtocol`) ejecutar `ReadDTCInformation`.
5. `UdsProtocol` construye la trama UDS y la pasa al `Transport` activo (`EnetTransport`), que la envía por TCP/DoIP y espera respuesta.
6. La respuesta sube: `Transport` → `Protocol` (interpreta a lista de códigos) → módulo `Diagnostics` (enriquece cada código con descripción/causas desde su propia base de datos de fallos).
7. El módulo `Diagnostics` persiste el resultado en su propio repositorio SQLite (histórico).
8. El comando Rust devuelve la lista tipada al frontend.
9. React actualiza estado y renderiza la vista.

Para datos en tiempo real, el patrón cambia en el paso 8: el backend emite eventos (`emit('live_data', payload)`) en loop controlado, y el frontend se suscribe con `listen`.

### 3.6 Gestión de errores y seguridad de operación

- Toda operación de comunicación con el vehículo devuelve `Result<T, DiagnosticError>` explícito (nunca panics silenciosos).
- Errores de conexión se capturan en `Transport` y se propagan como tipos específicos, nunca strings genéricos.
- Ninguna operación de escritura existe en V1 a nivel de código: ningún módulo tiene registrado un servicio UDS de escritura. Es garantía de capa de protocolo, no solo de UI.
- El manifest de permisos hace visible y auditable qué módulo puede tocar qué (ECU, red, filesystem) — si un módulo futuro pidiera `needs_ecu_write`, sería una señal explícita y revisable, no un cambio silencioso.

### 3.7 Telemetría local (no online)

Sistema de observabilidad interno, sin envío externo de datos:
- **Logs estructurados** (crate `tracing`), niveles por módulo.
- **Errores y crash reports** guardados localmente (`%APPDATA%/BMWToolbox/logs/`), consultables desde Configuración.
- **Performance:** tiempos de respuesta de operaciones ECU (útil para detectar adaptadores lentos o problemas de protocolo).
- Nunca se envía nada a un servidor externo en V1 — coherente con la decisión de "todo local".

---

## 4. Estructura de carpetas

```
BimmerDesk/
├── PROJECT_PLAN.md
├── README.md
├── DESIGN_SYSTEM.md               # Sistema de diseño detallado (se redacta en Fase 0)
├── src/                            # Frontend React + TypeScript
│   ├── main.tsx
│   ├── App.tsx
│   ├── components/                 # Primitivos UI compartidos (no específicos de un módulo)
│   │   ├── ui/
│   │   ├── charts/
│   │   └── layout/
│   ├── modules/                    # Un directorio por módulo (espejo de src-tauri/src/modules)
│   │   ├── dashboard/
│   │   ├── vehicle-info/
│   │   ├── live-data/
│   │   ├── diagnostics/
│   │   ├── maintenance/
│   │   ├── reports/
│   │   ├── settings/
│   │   ├── ai/
│   │   ├── coding/                 # Placeholder "Próximamente"
│   │   └── ecu-flash/              # Placeholder, sin lógica
│   ├── hooks/
│   ├── stores/                     # Estado global (Zustand)
│   ├── types/                      # Tipos compartidos (espejo de los DTOs de Rust)
│   ├── i18n/
│   └── styles/                     # Tokens de diseño (implementación de DESIGN_SYSTEM.md)
│
├── src-tauri/                      # Backend Rust
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── migrations/                 # Migraciones SQLite numeradas (001_*, 002_*...)
│   └── src/
│       ├── main.rs                 # Registro explícito de módulos activos
│       ├── module.rs               # Trait Module, ModuleManifest, ModuleRegistry
│       ├── command_bus/            # Command Bus tipado
│       ├── domain/                 # Manufacturer, VehiclePlatform, Vehicle, Protocol (abstracciones)
│       │   └── bmw/                # Implementación concreta BMW del dominio
│       ├── modules/                # Un directorio por módulo, cada uno implementa `Module`
│       │   ├── vehicle_info/
│       │   ├── live_data/
│       │   ├── diagnostics/
│       │   ├── maintenance/
│       │   ├── reports/
│       │   ├── ai/
│       │   ├── coding/             # Stub
│       │   └── ecu_flash/          # Stub
│       ├── transport/              # Trait Transport + implementaciones
│       │   ├── mod.rs
│       │   ├── enet.rs
│       │   ├── kdcan.rs            # Futuro
│       │   ├── elm327.rs           # Futuro
│       │   └── mock.rs             # Testing sin hardware
│       ├── protocol/               # Trait Protocol + implementaciones
│       │   ├── mod.rs
│       │   ├── uds.rs
│       │   ├── kwp2000.rs          # Futuro
│       │   └── obd2.rs             # Futuro
│       ├── db/                     # Conexión SQLite + repositorios (uno por módulo)
│       ├── telemetry/              # Logging, crash reports, performance
│       └── events/                 # Emisión de eventos en tiempo real
│
├── tests/
│   ├── unit/                       # Tests unitarios (lógica de dominio, parsing UDS...)
│   ├── integration/                # Tests de integración (módulo + DB + MockTransport)
│   └── ui/                         # Tests de componentes/flows React
│
├── .github/
│   └── workflows/                  # CI/CD (lint, test, build)
│
└── docs/
    ├── architecture/                # Diagramas y explicación de la arquitectura viva
    ├── protocols/                   # Notas técnicas de UDS/ENET/DoIP a medida que se documentan
    ├── decisions/                   # ADRs (Architecture Decision Records)
    │   ├── 0001-tauri-como-shell.md
    │   ├── 0002-arquitectura-de-modulos-en-compilacion.md
    │   └── ...
    ├── api/                         # Documentación de la API pública de cada módulo
    ├── design/                      # Mockups, decisiones de UI (referenciado desde DESIGN_SYSTEM.md)
    └── specs/                       # Specs detalladas por fase
```

**Regla de dependencia:** `commands/` (Tauri layer) depende del `command_bus`, que depende de `modules/`, que dependen de `domain/`, `transport/`, `protocol/` y de su propio repositorio en `db/`. Ningún módulo importa el repositorio de otro módulo directamente. `domain/`, `transport/` y `protocol/` no conocen Tauri — permite testear la lógica sin levantar la aplicación completa.

---

## 5. Módulos (arquitectura de módulos internos)

| Módulo | V1 | Descripción | Permisos declarados |
|---|---|---|---|
| **Dashboard** | Sí | Resumen general: estado del vehículo, alertas, accesos rápidos. Solo lee de las APIs públicas de otros módulos. | `needs_sqlite` (lectura agregada) |
| **Vehicle Info** | Sí | VIN, modelo, motor, caja, año, km, versión de software, ficha completa (fotos, notas, extras). | `needs_ecu`, `needs_sqlite`, `needs_filesystem` (fotos) |
| **Live Data** | Sí | RPM, velocidad, voltaje, temperaturas, carga motor, consumo, sensores. Gráficas y registro de sesión. | `needs_ecu`, `needs_sqlite` |
| **Diagnostics** | Sí (solo lectura) | Lectura de DTCs, descripciones, causas y soluciones, historial. Borrado explícitamente fuera de V1. | `needs_ecu`, `needs_sqlite` |
| **Maintenance** | Sí | Historial de aceite, filtros, pastillas, discos, ITV, neumáticos. Recordatorios por km/fecha. | `needs_sqlite` |
| **Reports** | Sí | Exportación PDF/CSV, historial, comparativas. Lee de otros módulos vía sus APIs públicas. | `needs_sqlite`, `needs_filesystem` |
| **Settings** | Sí | Idioma, tema, ajustes de conexión, gestión de adaptadores, actualizaciones. | `needs_sqlite` |
| **AI** | Interfaz only | Trait `AiProvider`, registro de proveedores, toggle activar/desactivar. Sin proveedor conectado en V1. | `needs_ai` (declarado, inactivo), `needs_network` (futuro) |
| **Coding** | No (stub) | Pantalla "Próximamente". Módulo registrado con manifest, sin lógica de negocio. | Ninguno activo |
| **ECU Flash** | No (stub) | Pantalla informativa. Sin lógica, sin acceso a servicios de escritura UDS. | Ninguno activo |

---

## 6. Roadmap por fases

No se inicia una fase sin cerrar correctamente la anterior. Cada fase se cierra actualizando este documento (resumen de lo hecho, pendientes, siguiente paso).

**Norma de todas las fases 1-4 (comunicación con el vehículo):** ninguna fase depende exclusivamente del vehículo real. Cada una tiene un hito intermedio obligatorio — "funciona contra `MockTransport`/`MockEcu`" — antes del hito final "funciona contra el vehículo real". Esto permite seguir avanzando aunque no haya acceso físico al coche en un momento dado. Detalle de la estrategia de mock en `docs/research/ECU_COMMUNICATION_RESEARCH.md` sección 6.

### Fase 0 — Fundamentos, gobernanza y setup
- **Objetivo:** monorepo funcional, arquitectura de módulos en el sitio (vacía de lógica de negocio), tooling de calidad activo desde el primer commit.
- **Tareas:**
  - Setup Tauri + React + TS + Vite.
  - Setup SQLite + sistema de migraciones numeradas.
  - `Module` trait, `ModuleRegistry`, `CommandBus` tipado — esqueleto funcional con un módulo de ejemplo.
  - `Transport`/`Protocol` traits vacíos + `MockTransport`.
  - Dominio `Manufacturer`/`VehiclePlatform`/`Vehicle`/`Protocol` (abstracciones + implementación `Bmw`).
  - `DESIGN_SYSTEM.md` (colores, iconografía, sombras, tipografía, espaciados, radios, animaciones) + tokens implementados en `src/styles/`.
  - Estructura de carpetas completa (incluyendo `tests/`, `docs/`, `.github/workflows/`).
  - Primeros ADRs: 0001 (Tauri), 0002 (arquitectura de módulos en compilación), 0003 (SQLite), 0004 (Command Bus tipado), 0005 (permisos declarativos).
  - Convenciones Git (branching, Conventional Commits, SemVer) documentadas en `README.md` o `docs/decisions/`.
  - CI/CD inicial en GitHub Actions: lint (`clippy` + `eslint`), tests, build.
  - Infra de tests: unit, integration (con `MockTransport`), UI — al menos un test de cada tipo como plantilla.
  - i18n base.
- **Prioridad:** Alta. **Dificultad:** Media-alta (más ancha que antes, pero evita deuda estructural). **Dependencias:** ninguna.

### Fase 0B — Investigación técnica y simulador (Mock ECU)
- **Objetivo:** eliminar la mayor fuente de incertidumbre del proyecto (la capa de comunicación con el vehículo) antes de comprometerse a una implementación real, y desbloquear desarrollo de Fases 2-10 sin depender de tener el coche siempre disponible.
- **Contexto:** ver `docs/research/ECU_COMMUNICATION_RESEARCH.md` — documento de referencia técnica de esta fase, con la tabla de servicios UDS, matriz de compatibilidad y estrategia de mock ya elaboradas.
- **Tareas:**
  - Captura de tráfico real (Wireshark) con el F-Series del usuario para confirmar el framing de ENET (sección 2.6 de la investigación) — punto/puerto exacto, si sigue DoIP estándar o un framing propio de BMW.
  - Implementación de `MockEcu`: máquina de estados que responde como una ECU real (control de sesión, DIDs configurables, DTCs simulados, respuestas negativas, latencia simulada, escenarios de error).
  - Implementación de `MockTransport` completo (no solo el esqueleto de Fase 0) conectado a `MockEcu`.
  - Implementación de `UdsProtocol` (servicios `0x10`, `0x19`, `0x22`, `0x3E`, `0x27` condicional) probada íntegramente contra el mock.
  - Validación de al menos un DID real (VIN) contra el vehículo, usando ya `EnetTransport` en su versión mínima, para cerrar el ciclo mock → real.
  - Documentar en `docs/protocols/` cada hallazgo confirmado (framing, DIDs, timings reales de P2Server/P2*Server).
- **Prioridad:** Alta — bloquea Fase 1 con garantías. **Dificultad:** Alta (incertidumbre técnica real, no solo volumen de trabajo). **Dependencias:** Fase 0.
- **Criterio de cierre:** `UdsProtocol` + `MockEcu`/`MockTransport` funcionando de punta a punta, framing de ENET confirmado por captura propia (no solo por fuentes de terceros), y un VIN real leído del vehículo.

### Fase 1 — Transporte ENET (F-Series)
- **Objetivo:** comunicación real con el vehículo a nivel de bytes, ya con el framing confirmado en Fase 0B.
- **Tareas:** implementar `EnetTransport` completo (conexión, reconexión, gestión de errores de red más allá de la prueba mínima de Fase 0B); tests de integración contra hardware real + contra `MockTransport`.
- **Prioridad:** Alta. **Dificultad:** Media (el riesgo grande ya se resolvió en 0B). **Dependencias:** Fase 0B.

### Fase 2 — Protocolo UDS, módulo Vehicle Info
- **Objetivo:** interpretar bytes como diagnóstico real; primer módulo completo de punta a punta.
- **Tareas:** subset UDS necesario (control de sesión, `ReadDataByIdentifier`, `ReadDTCInformation`); parsing VIN/modelo/motor/caja/versión software; módulo `vehicle_info` completo (backend + frontend) incluida ficha ampliada (notas, fotos, extras).
- **Prioridad:** Alta. **Dificultad:** Alta. **Dependencias:** Fase 1.

### Fase 3 — Módulo Live Data
- **Objetivo:** streaming en vivo con gráficas.
- **Tareas:** lectura periódica de PIDs/valores en vivo; eventos Tauri; gráficas en tiempo real; logging de sesión en el repositorio propio del módulo.
- **Prioridad:** Alta. **Dificultad:** Media-alta. **Dependencias:** Fase 2.

### Fase 4 — Módulo Diagnostics
- **Objetivo:** lectura y explicación de errores.
- **Tareas:** lectura de DTCs vía UDS; base de datos local de descripciones/causas/soluciones (ver riesgo sección 8); historial persistente.
- **Prioridad:** Alta. **Dificultad:** Media. **Dependencias:** Fase 2.

### Fase 5 — Módulo Dashboard
- **Objetivo:** vista resumen que agrega datos de otros módulos vía sus APIs públicas.
- **Tareas:** agregación de estado del vehículo, alertas, accesos rápidos.
- **Prioridad:** Media. **Dificultad:** Baja-media. **Dependencias:** Fases 2, 3, 4.

### Fase 6 — Módulo Maintenance
- **Objetivo:** módulo independiente de hardware, puede adelantarse en paralelo si conviene.
- **Tareas:** CRUD histórico de mantenimiento; recordatorios por km/fecha.
- **Prioridad:** Media. **Dificultad:** Baja. **Dependencias:** solo Fase 0 (DB + arquitectura de módulos).

### Fase 7 — Módulo Reports
- **Objetivo:** exportación y comparativas.
- **Tareas:** generación PDF/CSV; comparativas temporales; consumo de APIs públicas de Diagnostics y Maintenance.
- **Prioridad:** Media. **Dificultad:** Media. **Dependencias:** Fases 4, 6.

### Fase 8 — Módulo Settings
- **Objetivo:** ajustes de usuario y de conexión.
- **Tareas:** idioma, tema, gestión de adaptadores, actualizaciones, panel de telemetría local (logs/errores/performance). Puede desarrollarse en paralelo con otras fases.
- **Prioridad:** Media. **Dificultad:** Baja.

### Fase 9 — Módulos stub (Coding, ECU Flash)
- **Objetivo:** dejar reservado sin implementar, ya con forma de módulo real.
- **Tareas:** ambos módulos registrados en `ModuleRegistry` con manifest de permisos vacío; pantalla "Próximamente" / informativa.
- **Prioridad:** Baja. **Dificultad:** Baja.

### Fase 10 — Módulo AI (sin proveedor)
- **Objetivo:** dejar preparado el sistema de proveedores de IA.
- **Tareas:** trait `AiProvider`; registro de proveedores; toggle activar/desactivar en Settings. Puede ir en paralelo.
- **Prioridad:** Baja-media. **Dificultad:** Baja-media.

### Fase 11 — Empaquetado y distribución
- **Objetivo:** instalador entregable.
- **Tareas:** bundler Tauri (`.msi`), icono, firma de código (ver decisión pendiente), auto-update si se decide activar, pipeline de release en CI/CD.
- **Prioridad:** Alta (al final). **Dificultad:** Media. **Dependencias:** todas las fases funcionales anteriores estables.

### Post-V1 (fuera de alcance de este plan, reservado en arquitectura)
- Runtime WASM para plugins de terceros verdaderamente dinámicos (marketplace real).
- `KDcanTransport` (E-Series), `Elm327Transport` (genérico), `Kwp2000Protocol`, `Obd2Protocol` multimarca.
- Nuevos `Manufacturer` (Audi, Mercedes...) como validación real de la jerarquía de dominio.
- Borrado de DTCs (con confirmación explícita reforzada).
- Codificación real.
- Reprogramación ECU real.
- Proveedor IA real conectado.
- Cuentas de usuario / sync en la nube.
- Soporte macOS/Linux.

---

## 7. Aspecto visual

Referencias: BMW, Tesla, VS Code, Discord, Steam, Adobe, JetBrains. Tema oscuro por defecto, limpio, sin sobrecarga visual, animaciones suaves (transiciones de estado, no decorativas).

El detalle completo (paleta de colores, iconografía, sombras, tipografía, espaciados, radios, animaciones) se documenta en **`DESIGN_SYSTEM.md`**, redactado como entregable de la Fase 0, junto con su implementación como tokens en `src/styles/` y componentes primitivos en `src/components/ui/` antes de construir ningún módulo visual.

---

## 8. Estrategia de testing

Desde Fase 0, no como añadido posterior:

| Tipo | Alcance | Herramienta |
|---|---|---|
| **Unitarios** | Lógica de dominio pura: parsing UDS, cálculo de recordatorios de mantenimiento, mapeo DTC → descripción. Sin I/O. | `cargo test` (Rust), `vitest` (TS) |
| **Integración** | Módulo completo contra `MockTransport` + SQLite real (archivo temporal). Verifica el flujo Command Bus → módulo → repositorio. | `cargo test` con fixtures |
| **Protocolo** | Contra hardware real (F-Series del usuario) en fases 1-4, ejecutados manualmente/documentados (no en CI, requieren hardware físico). | Manual + checklist en `docs/protocols/` |
| **UI** | Componentes y flujos críticos del frontend (formularios, tablas de DTC, gráficas). | `vitest` + `@testing-library/react` |

CI ejecuta unitarios + integración + UI en cada push. Los tests de protocolo contra hardware real quedan fuera de CI por naturaleza (no hay vehículo en un runner), pero se documentan como checklist manual obligatorio antes de cerrar Fases 1-4.

---

## 9. Base de datos

- **Migraciones numeradas y secuenciales** en `src-tauri/migrations/` (`001_init.sql`, `002_add_maintenance.sql`, ...), nunca editadas tras aplicarse — solo migraciones nuevas.
- **Un esquema por módulo:** cada módulo es dueño de sus tablas (ej. `diagnostics_history`, `maintenance_records`) y de su propio `Repository`. Tablas compartidas (ej. `vehicles`) son propiedad de `Vehicle Info` y se consultan por los demás módulos exclusivamente a través de su API pública, nunca con SQL directo cruzado.
- **Documento de modelo entidad-relación** en `docs/architecture/database-er.md`, actualizado en cada migración nueva — se crea junto con la migración `001_init` en Fase 0.

---

## 10. Gobernanza técnica

**Principios rectores:** `PROJECT_PRINCIPLES.md` complementa esta sección — define reglas no negociables (ninguna capa se salta a otra, prohibido condicional por identidad de marca/módulo en lógica de negocio, ADR obligatorio en toda decisión relevante, prueba del "dentro de cinco años") y la regla de reconsideración: si durante el desarrollo una decisión ya tomada deja de parecer la mejor opción, se detiene esa parte del trabajo, se explica el problema, se proponen alternativas y se espera aprobación antes de continuar — nunca se sigue implementando algo que se considera inferior solo por fidelidad al plan escrito.

### 10.1 Architecture Decision Records (ADR)
Toda decisión arquitectónica relevante (elección de librería, patrón estructural, cambio de rumbo) se documenta en `docs/decisions/NNNN-titulo.md`: contexto, decisión, alternativas descartadas y motivo. Nunca se borran, solo se marcan como `Superseded by 00NN` si cambian. Arrancan en Fase 0 con las decisiones ya tomadas en este documento (Tauri, arquitectura de módulos, SQLite, Command Bus, permisos declarativos).

### 10.2 Git
- **Ramas:** `main` siempre desplegable; `feature/<modulo>-<descripcion>` para trabajo en curso; sin commits directos a `main` una vez exista CI.
- **Commits:** Conventional Commits (`feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`) — permite generar changelog automático más adelante.
- **Versionado:** SemVer (`MAJOR.MINOR.PATCH`) desde la primera release interna (`0.1.0`), aunque V1 completo no llegue a `1.0.0` hasta cerrar Fase 11.
- **Releases:** tag Git por versión + GitHub Release con el `.msi` adjunto (manual en V1, automatizable en CI/CD más adelante).

### 10.3 CI/CD
GitHub Actions desde Fase 0, aunque el desarrollo sea individual — detecta regresiones sin depender de disciplina manual:
- **Lint:** `cargo clippy --all-targets -- -D warnings`, `eslint`.
- **Test:** unitarios + integración + UI (sección 8).
- **Build:** compila el bundle Tauri en cada push a `main` (no publica release, solo valida que compila).
- Pipeline de release (build + firma + publicación) se activa en Fase 11.

---

## 11. Riesgos identificados

| Riesgo | Impacto | Mitigación |
|---|---|---|
| Protocolo ENET/UDS de BMW no está documentado oficialmente por BMW | Alto — bloquea Fases 1-2 | Implementación únicamente a partir de documentación pública y análisis de tráfico propio (Wireshark sobre el propio vehículo), nunca reutilizando código o binarios de herramientas propietarias BMW, para evitar problemas de propiedad intelectual. |
| Fuente de la base de datos de descripciones de DTCs | Medio — afecta calidad de Fase 4 | Ver decisión pendiente 12.1. |
| Curva de aprendizaje de Rust (perfil React/Laravel) | Medio — puede ralentizar Fases 0-3 | Cada fase de Rust se acompaña de explicación de conceptos (ownership, traits, async, traits objects para el `ModuleRegistry`) aplicados directamente al código del proyecto. |
| Complejidad añadida por arquitectura de módulos/Command Bus | Medio — más superficie en Fase 0 | Mitigado por ser registro en compilación (no dinámico) y bus tipado — complejidad conceptual, no complejidad de infraestructura en runtime. |
| Desarrollador único (bus factor) | Medio | ADRs + specs de fase + este documento. |
| Firma de código / advertencias SmartScreen | Bajo-medio | Ver decisión pendiente 12.5. |
| Acceso a hardware real para testing | Alto para Fases 1-4 | Usuario ya dispone de F-Series; se asume cable ENET disponible o adquirido antes de Fase 1. `MockTransport` permite avanzar en Fase 0 sin hardware. |

---

## 12. Decisiones pendientes de aprobación

### 12.1 Fuente de la base de datos de códigos de error (DTCs)
- **Opción A (recomendada):** DTCs genéricos OBD2 estándar (públicos) + ampliación manual progresiva con códigos BMW detectados en el propio vehículo durante desarrollo.
- **Opción B:** dataset comunitario existente de códigos BMW, si se localiza uno con licencia clara.
- **Recomendación:** Opción A — sin riesgo legal ni de calidad, aunque más lenta al inicio.

### 12.2 Gestión de estado en frontend
- **Opción A (recomendada):** Zustand (estado global) + TanStack Query (llamadas a comandos Tauri con loading/error automáticos).
- **Opción B:** Context API + hooks propios, sin librería adicional.
- **Recomendación:** Opción A.

### 12.3 Librería de gráficas en tiempo real
- **Opción A:** Recharts — simple, suficiente para frecuencias moderadas.
- **Opción B:** uPlot — mucho más rápida en refresco de alta frecuencia, API más low-level.
- **Recomendación:** Recharts en Fase 3, migrar a uPlot solo si el rendimiento real lo exige.

### 12.4 Generación de informes PDF
- **Opción A:** generación nativa en Rust (`printpdf`/`genpdf`).
- **Opción B (recomendada):** renderizar HTML/CSS en el WebView y exportar vía impresión del sistema — reutiliza el sistema de diseño ya construido en React.
- **Recomendación:** Opción B para V1.

### 12.5 Firma de código del instalador
- **Opción A (recomendada para V1):** sin firmar (uso personal). SmartScreen mostrará advertencia.
- **Opción B:** certificado de firma de código de pago.
- **Recomendación:** Opción A mientras el uso sea personal/interno.

### 12.6 Auto-actualización
- **Opción A (recomendada para V1):** no activar todavía, solo instalación manual.
- **Opción B:** `tauri-plugin-updater` desde V1, requiere infraestructura de releases firmadas.
- **Recomendación:** Opción A, dependencia preparada pero desactivada.

### 12.7 Identificador de aplicación / bundle
- Falta definir: nombre de paquete, identificador reverso (ej. `com.tunombre.bmwtoolbox`), icono.
- **Recomendación:** definirlo al inicio de Fase 0, no bloquea el resto del diseño.

### 12.8 Alojamiento de CI/CD y control de versiones
- Se asume **GitHub** (Actions + Releases) por ser el estándar de facto y no tener coste en repos privados pequeños para uso individual.
- **Recomendación:** confirmar que GitHub es el proveedor deseado antes de crear `.github/workflows/` en Fase 0; si se prefiere GitLab/otro, el diseño de CI se traslada sin cambios conceptuales.

### 12.9 Security Access (`0x27`) — RESUELTA
- Incorporada tras `docs/research/ECU_COMMUNICATION_RESEARCH.md`: se implementa en V1 **solo si algún DID de lectura concreto lo exige**, limitado a lectura, nunca como base para escritura/coding. Ver sección 3 de la investigación.

**Nota sobre 12.1-12.8:** siguen abiertas formalmente, pero no bloquean el inicio de Fase 0 — son decisiones de detalle dentro de fases concretas (frontend state en Fase 0/2, gráficas en Fase 3, PDF en Fase 7, firma/auto-update/bundle en Fase 11, CI en Fase 0). Se resuelven cuando la fase correspondiente las necesite, aplicando la opción recomendada salvo objeción explícita antes de ese punto.

---

## 13. Estado del proyecto

- **Fase actual:** Planificación cerrada. Arquitectura, gobernanza, principios (`PROJECT_PRINCIPLES.md`) e investigación técnica (`docs/research/ECU_COMMUNICATION_RESEARCH.md`) aprobados por el usuario. Luz verde para empezar a construir.
- **Siguiente paso:** iniciar Fase 0 (setup del monorepo, arquitectura de módulos, gobernanza y tooling) — prerrequisito de Fase 0B, que es donde se resuelve el mayor riesgo técnico del proyecto (comunicación real con el vehículo).
