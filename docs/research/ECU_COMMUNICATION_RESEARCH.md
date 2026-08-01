# Investigación técnica: comunicación con la ECU (BMW F-Series)

**Estado:** Borrador inicial para revisión. No es aún verificación validada contra hardware real — eso es tarea explícita de la Fase 0B (ver sección 12).

**Propósito de este documento:** ser la referencia técnica de fondo para todo lo que la aplicación necesite implementar en la capa `transport`/`protocol`. Antes de escribir una sola línea de `EnetTransport` o `UdsProtocol`, cualquier duda sobre "¿qué hace este byte?" debe poder resolverse aquí.

**Nivel de confianza de las fuentes:** los protocolos genéricos (OBD-II, UDS, ISO-TP, KWP2000, DoIP) son estándares ISO/SAE públicos y bien documentados — se presentan aquí con confianza alta. Los detalles específicos de la implementación de BMW sobre ENET (framing exacto, puertos, direccionamiento) **no son un estándar público de BMW**: provienen de ingeniería inversa de la comunidad (proyectos open-source que lo han documentado a base de capturar tráfico real de herramientas BMW como INPA/ESYS/ISTA). Se marcan explícitamente como **hipótesis a validar** — no se deben implementar como si fueran hechos confirmados hasta contrastarlos con una captura de tráfico propia (Fase 0B).

---

## 1. Panorama general: de la app a la ECU

```
BMW Toolbox (Rust)
      │
      │ 1. Construye un mensaje de servicio de diagnóstico (ej. "leer VIN")
      ▼
  Capa UDS (Protocol)
      │
      │ 2. Codifica el mensaje como Application Protocol Data Unit (A_PDU) de UDS
      ▼
  Capa de transporte del vehículo
      │
      │ 3a. Si CAN: fragmenta el A_PDU en frames CAN vía ISO-TP
      │ 3b. Si Ethernet (F-Series/ENET): encapsula el A_PDU en un paquete DoIP-like sobre TCP
      ▼
  Interfaz física (adaptador ENET / cable K+DCAN / ELM327)
      │
      │ 4. Envía las tramas por el medio físico correspondiente
      ▼
  Gateway del vehículo (BMW: "ZGW", Zentrale Gateway)
      │
      │ 5. Enruta el mensaje al bus interno correcto (K-CAN, PT-CAN...) y a la ECU destino
      ▼
  ECU objetivo (ej. DME/DDE motor, cuadro, etc.)
      │
      │ 6. Procesa el servicio UDS, genera respuesta (positiva o negativa)
      ▼
  (mismo camino de vuelta)
```

Punto clave para la arquitectura: los pasos 1-2 (qué se pide) son independientes de los pasos 3-4 (cómo viaja). Esto es exactamente la separación `Protocol` / `Transport` ya definida en `PROJECT_PLAN.md` — esta investigación la confirma en vez de contradecirla.

---

## 2. Protocolos: qué es cada uno y para qué sirve aquí

### 2.1 OBD-II (SAE J1979 / ISO 15031)
Protocolo genérico, obligatorio en todos los vehículos (por normativa de emisiones) desde mediados de los 90 (EE.UU.) / principios 2000 (UE). Expone un conjunto **estandarizado y limitado** de datos vía "Modes" (Mode 01 = datos en vivo, Mode 03 = leer DTCs, Mode 09 = info del vehículo). Cualquier lector OBD-II genérico (incluido un ELM327) puede leerlo, sin conocer nada específico de BMW.

**Rol en BMW Toolbox:** protocolo de bajo nivel/fallback, útil sobre todo para el futuro `Elm327Transport` + `Obd2Protocol` (multimarca). No da acceso a la riqueza de datos que BMW expone vía UDS (equipamiento, versiones de software por módulo, DTCs específicos de fabricante).

### 2.2 UDS — Unified Diagnostic Services (ISO 14229)
Protocolo de diagnóstico **rico y genérico en su estructura**, pero cuyo contenido concreto (qué identificadores de datos existen, qué DTCs hay) lo define cada fabricante. Es lo que usan las herramientas profesionales (ISTA, ESYS) para hablar con las ECUs BMW modernas. Estructura de mensaje:

```
[SID] [Sub-función (opcional)] [Datos]
```

`SID` = Service Identifier (ej. `0x22` = ReadDataByIdentifier). Respuesta positiva = `SID + 0x40`; respuesta negativa = `0x7F [SID] [NRC]` (Negative Response Code).

**Rol en BMW Toolbox:** protocolo principal para F-Series. Es el que da acceso a VIN, versiones de software, DTCs específicos BMW, live data rico.

### 2.3 KWP2000 (ISO 14230)
Predecesor de UDS, usado en BMW E-Series (anteriores a F-Series) típicamente sobre K-Line (protocolo serie de un solo hilo, no CAN). Estructuralmente similar a UDS (misma filosofía de servicio + sub-función + datos) pero con juego de servicios distinto y códigos de servicio diferentes.

**Rol en BMW Toolbox:** **no se implementa en V1.** Reservado para un futuro `Kwp2000Protocol` si se decide dar soporte a E-Series. Se menciona aquí solo para dejar constancia de que UDS y KWP2000 no son intercambiables — son primos, no el mismo protocolo con otro nombre.

### 2.4 ISO-TP — ISO-TP / ISO 15765-2
No es un protocolo de diagnóstico en sí — es un **protocolo de transporte** que permite enviar mensajes UDS más largos que los 8 bytes que caben en una trama CAN clásica, fragmentándolos:

- **Single Frame (SF):** mensaje completo cabe en una trama.
- **First Frame (FF) + Consecutive Frames (CF):** mensaje largo, fragmentado.
- **Flow Control (FC):** el receptor le dice al emisor cuántas tramas puede mandar seguidas (Block Size) y con qué separación mínima (STmin) antes de seguir.

**Rol en BMW Toolbox:** **no aplica directamente a ENET.** ISO-TP es el transporte de UDS **sobre CAN** (relevante para un futuro `KDcanTransport`, que sí habla CAN/K-Line). Sobre Ethernet, el transporte equivalente es DoIP (sección 2.5) — DoIP ya resuelve el problema de mensajes largos a su manera (longitud explícita en el header), no necesita fragmentación tipo ISO-TP.

### 2.5 DoIP — Diagnostics over IP (ISO 13400)
Estándar que define cómo encapsular mensajes de diagnóstico (UDS) sobre TCP/IP y UDP, pensado para vehículos con gateway Ethernet. Define:

- **Descubrimiento de vehículo (UDP, puerto 13400):** el tester emite un `Vehicle Identification Request`; el vehículo responde con un `Vehicle Announcement`/`Identification Response` que incluye VIN, dirección lógica del gateway (Logical Address), EID, GID.
- **Sesión de diagnóstico (TCP, puerto 13400):** tras el descubrimiento, el tester abre TCP contra el gateway, envía un `Routing Activation Request` (autoriza la sesión), y a partir de ahí intercambia `Diagnostic Message` que llevan dentro, literalmente, los bytes de un mensaje UDS.
- **Header genérico DoIP:** `[Versión protocolo][Inverso de versión][Tipo de payload][Longitud del payload][Payload]`.

**Rol en BMW Toolbox:** es el modelo conceptual de referencia para `EnetTransport`. **Importante (ver 2.6):** BMW no necesariamente implementa DoIP al pie de la letra del estándar ISO 13400 en todos sus modelos/años — es la hipótesis de partida a confirmar.

### 2.6 ENET — el adaptador físico de BMW, y lo que aún no está confirmado
"ENET" no es un protocolo — es el **nombre del cable/adaptador** que convierte el conector de diagnosis de BMW (bajo el volante en F-Series) a un conector Ethernet (RJ45) estándar. Lo que viaja por ese cable es lo que hay que determinar con precisión.

**Lo que la comunidad de ingeniería inversa (herramientas open-source que interoperan con INPA/ESYS/ISTA) reporta, con distintos grados de certeza según el modelo/año:**
- El PC obtiene una IP del gateway del vehículo (DHCP) al conectar el cable.
- La sesión de diagnóstico se establece contra una IP del gateway (ZGW) en la red del vehículo.
- Existen referencias (no verificadas por este proyecto) tanto a un **puerto TCP específico de BMW (frecuentemente citado: 6801)** con un framing propio, más simple que un header DoIP completo, como a un comportamiento **más alineado con DoIP estándar (puerto 13400)** en modelos más recientes (parece variar según generación de gateway).
- El contenido de los mensajes, una vez fuera del framing de transporte, sí parece ser UDS "de libro" (mismos SIDs, misma estructura de servicio).

**Conclusión de esta sección — tratar como hipótesis, no como hecho:** el `EnetTransport` de la Fase 1 no se debe escribir contra una "especificación BMW" porque no existe una pública y fiable al 100%. Se debe escribir **contra una captura de tráfico real** (Wireshark, con el propio F-Series del usuario y una herramienta de referencia conocida, o en su defecto contra ejemplos documentados por proyectos open-source ya existentes cuyo comportamiento se pueda verificar). Esto es precisamente el objetivo de la Fase 0B (sección 12).

---

## 3. Servicios UDS candidatos para BMW Toolbox V1

Tabla completa de servicios UDS relevantes, con decisión explícita de alcance. Código hexadecimal según ISO 14229-1.

| Servicio | SID | Descripción | Uso en BMW Toolbox | Estado | Motivo |
|---|---|---|---|---|---|
| Diagnostic Session Control | `0x10` | Cambia el tipo de sesión de diagnóstico (default, extendida, programación...) | Necesario para entrar en sesión extendida antes de leer ciertos datos/DTCs | **V1** | Requisito previo de casi todo lo demás. |
| ECU Reset | `0x11` | Solicita reinicio de la ECU | No aplica a lectura | **No implementar** | Fuera de alcance de V1 (no es escritura destructiva, pero altera el estado del vehículo — se excluye por precaución). |
| Clear Diagnostic Information | `0x14` | Borra DTCs almacenados | Borrado de errores | **No implementar (V1). Futuro.** | Explícitamente fuera de V1 según `PROJECT_PLAN.md` — operación que modifica estado del vehículo. |
| Read DTC Information | `0x19` | Lee códigos de fallo almacenados (con sub-funciones: por estado, por severidad, snapshot...) | Núcleo del módulo Diagnostics | **V1** | Requisito directo de V1. |
| Read Data By Identifier | `0x22` | Lee un valor identificado por un DID (Data Identifier) — VIN, versión SW, sensores en vivo, etc. | Núcleo de Vehicle Info y Live Data | **V1** | Requisito directo de V1. |
| Read Memory By Address | `0x23` | Lectura directa de memoria de la ECU | Ninguno previsto | **No implementar** | No hay caso de uso en V1; uso típico es ingeniería/depuración de bajo nivel, no diagnóstico de usuario. |
| Read Scaling Data By Identifier | `0x24` | Metadatos de escalado de un DID (unidades, rango) | Podría enriquecer Live Data (unidades correctas) | **Futuro (opcional)** | No bloquea V1; los valores se pueden mostrar con escalado fijo conocido inicialmente. |
| Security Access | `0x27` | Desbloquea funciones protegidas mediante seed/key | Requisito previo para leer ciertos datos protegidos, y obligatorio para cualquier escritura futura | **V1 parcial** — solo si algún dato de **lectura** necesario lo exige; **No implementar** la parte orientada a escritura/coding. | BMW protege algunos DIDs incluso en lectura. Se implementa el mínimo necesario para lectura, documentando cada caso; no se persigue como puerta de entrada a funciones de escritura. |
| Communication Control | `0x28` | Habilita/deshabilita comunicación de ciertos mensajes | Ninguno previsto | **No implementar** | Sin caso de uso en solo-lectura. |
| Read Data By Periodic Identifier | `0x2A` | Suscripción a datos periódicos (streaming nativo del propio protocolo) | Alternativa a polling manual para Live Data | **Futuro (a evaluar en Fase 0B/3)** | Podría simplificar Live Data si el gateway BMW lo soporta bien; se decide tras pruebas reales — polling con `0x22` repetido es el fallback seguro y más simple para V1. |
| Dynamically Define Data Identifier | `0x2C` | Define un DID compuesto por el propio tester | Ninguno previsto en V1 | **No implementar** | Optimización avanzada, no necesaria para el alcance actual. |
| Write Data By Identifier | `0x2E` | Escribe un valor en la ECU (base de la codificación) | — | **No implementar. Nunca en V1.** | Es literalmente "escribir en la centralita" — prohibido por diseño en V1. |
| Input Output Control By Identifier | `0x2F` | Fuerza entradas/salidas (ej. activar un actuador) | — | **No implementar** | Operación de escritura/control, fuera de alcance V1. |
| Routine Control | `0x31` | Ejecuta rutinas (test de actuadores, procesos de codificación/flash) | — | **No implementar** | Base de Coding/Flash — módulos stub, sin lógica. |
| Request Download / Upload | `0x34` / `0x35` | Prepara transferencia de firmware | — | **No implementar** | Base de ECU Flash — explícitamente prohibido en V1. |
| Transfer Data | `0x36` | Transfiere bloques de datos (firmware) | — | **No implementar** | Idem. |
| Request Transfer Exit | `0x37` | Cierra una transferencia | — | **No implementar** | Idem. |
| Tester Present | `0x3E` | "Sigo aquí" — mantiene viva una sesión no-default | Necesario para mantener sesión extendida durante Live Data/Diagnostics | **V1** | Sin esto, la ECU vuelve sola a sesión default y corta el flujo de datos extendido. |
| Control DTC Setting | `0x85` | Activa/desactiva el registro de nuevos DTCs | Ninguno previsto | **No implementar** | Alteraría el comportamiento de diagnóstico del vehículo — fuera de alcance de solo-lectura. |

**Resumen del subconjunto mínimo V1:** `0x10`, `0x19`, `0x22`, `0x3E`, y `0x27` solo si algún DID de lectura concreto lo exige (a confirmar en Fase 0B con captura real). Todo lo demás queda fuera hasta nueva decisión explícita.

---

## 4. Matriz de compatibilidad

| Función | OBD-II | UDS | ENET (transporte) | BMW F20 (caso real usuario) | V1 |
|---|:---:|:---:|:---:|:---:|:---:|
| Leer VIN | ✅ (Mode 09) | ✅ (`0x22`) | ✅ | ✅ | ✅ |
| Leer DTCs | ✅ (Mode 03, genérico) | ✅ (`0x19`, rico, específico BMW) | ✅ | ✅ | ✅ |
| Live Data básico (RPM, velocidad, temp.) | ✅ (Mode 01, PIDs estándar) | ✅ (`0x22`, DIDs BMW, más completo) | ✅ | ✅ | ✅ |
| Live Data avanzado (sensores específicos BMW) | ❌ | ✅ | ✅ | ✅ | ✅ (según DID disponible) |
| Versión de software por módulo | ❌ | ✅ | ✅ | ✅ | ✅ |
| Equipamiento / opciones instaladas | ❌ | ✅ (vía DIDs específicos, a confirmar cuáles) | ✅ | ⚠️ a confirmar | Futuro/opcional — depende de disponibilidad real del DID |
| Borrado de DTCs | ✅ (Mode 04) | ✅ (`0x14`) | ✅ | ✅ (capacidad del vehículo) | ❌ (excluido por diseño) |
| Coding | ❌ | ✅ (`0x2E`, `0x31`...) | ✅ | ✅ (capacidad del vehículo) | ❌ |
| Flash ECU | ❌ | ✅ (`0x34`-`0x37`) | ✅ | ✅ (capacidad del vehículo) | ❌ |

Lectura de la tabla: las columnas OBD-II/UDS/ENET indican **si el protocolo/transporte es técnicamente capaz** de esa función. La columna "BMW F20" indica si el vehículo del usuario la soporta en la práctica. La columna V1 es la **decisión de producto** — que algo sea técnicamente posible no significa que se implemente.

---

## 5. Orden de implementación recomendado

1. **`MockTransport` + `MockEcu`** (sección 6) — antes que nada, sin excepción.
2. **`UdsProtocol` contra el mock** — construcción/parseo de mensajes `0x10`, `0x22`, `0x19`, `0x3E`, manejo de respuesta positiva/negativa, sin tocar red todavía.
3. **`EnetTransport` — descubrimiento y conexión** contra el vehículo real, validando primero las hipótesis de la sección 2.6 con captura de tráfico.
4. **`EnetTransport` — envío/recepción de mensajes UDS reales**, integrando con el `UdsProtocol` ya probado contra el mock.
5. **DIDs concretos** (VIN, versión SW, live data) — se documentan aquí a medida que se identifican por captura real, no se asumen de antemano.

Esta secuencia es la aplicación directa de la norma pedida: **ninguna fase depende exclusivamente del coche** hasta el paso 3.

---

## 6. Estrategia de simulación (Mock ECU)

```
App (módulos)
   │
   ▼
Protocol (UdsProtocol) ── idéntico código tanto en mock como en real
   │
   ▼
Transport (trait)
   │
   ├── EnetTransport ──── habla con el vehículo real (TCP/DoIP-like)
   │
   └── MockTransport ──── habla con MockEcu (proceso/objeto en memoria)
                              │
                              ▼
                          MockEcu: máquina de estados que:
                          - responde a Diagnostic Session Control como una ECU real
                            (rechaza servicios si la sesión activa no los permite)
                          - tiene un "banco de DIDs" configurable (VIN falso, RPM falso...)
                          - puede simular latencia, timeouts, respuestas negativas (NRC)
                          - permite "escenarios": arranque en frío, DTC activo, sensor
                            fuera de rango, ECU que no responde, etc.
```

**Qué se puede probar 100% sin coche:**
- Toda la lógica de `UdsProtocol` (construcción de tramas, parseo de respuestas, manejo de NRC, reintentos, timeouts lógicos).
- Todos los módulos de dominio (Diagnostics, Live Data, Vehicle Info) de punta a punta, incluyendo su interacción con SQLite.
- Toda la UI (React) contra datos simulados — desarrollo de pantallas no bloqueado por acceso al vehículo.
- Casos de error: ECU no responde, sesión rechazada, DTC con formato inesperado, desconexión a mitad de operación.

**Qué requiere obligatoriamente hardware real:**
- Confirmar el framing exacto de `EnetTransport` (sección 2.6).
- Confirmar qué DIDs concretos responde un F20 real y con qué formato de datos exacto.
- Medir timings reales (P2Server, P2*Server — ver sección 7) del gateway BMW real, para calibrar los timeouts del `MockEcu` de forma realista.
- Validación final de cada fase antes de darla por cerrada (checklist manual, ya previsto en `PROJECT_PLAN.md` sección 8).

---

## 7. Timeouts, reconexión y recuperación ante errores

UDS define parámetros de temporización estándar que el `Protocol`/`Transport` deben respetar:

- **P2Server:** tiempo máximo que la ECU tarda en responder a una petición (por defecto ~50 ms en muchas implementaciones, pero varía por fabricante/ECU — a confirmar con captura real para BMW).
- **P2*Server (P2 extendido):** si la ECU necesita más tiempo, responde primero con `0x7F [SID] 0x78` (*"response pending"*) y el tester debe esperar hasta P2*Server (típicamente hasta varios segundos) antes de considerar timeout real.
- **S3 (tester present interval):** en sesión no-default, si no se manda `TesterPresent (0x3E)` con suficiente frecuencia (típicamente cada 2-3 s, por debajo de S3), la ECU vuelve sola a sesión default.

**Estrategia de manejo de errores en `Transport`/`Protocol`:**
- Timeout de red (`Transport`) vs. timeout de protocolo (`Protocol`, esperando `0x78`) son **errores distintos**, deben ser tipos distintos en `DiagnosticError` — un timeout de red probablemente significa cable desconectado; un `0x78` repetido es la ECU trabajando, no un fallo.
- Reconexión: ante pérdida de conexión TCP (ENET), un reintento automático único con backoff corto es razonable; más de eso debe ser una decisión visible del usuario (evitar reintentos silenciosos indefinidos que oculten un problema real de hardware).
- Toda `NRC` (Negative Response Code) se mapea a un tipo Rust explícito y a un mensaje entendible en la UI — nunca se descarta silenciosamente una respuesta negativa.
- Antes de cerrar cualquier fase que toque hardware real, el checklist manual (ya previsto en `PROJECT_PLAN.md`) debe incluir explícitamente: desconexión a mitad de operación, timeout simulado, y respuesta negativa forzada — no solo el camino feliz.

---

## 8. Riesgos y limitaciones detectados en esta investigación

| Riesgo | Detalle | Mitigación propuesta |
|---|---|---|
| Framing exacto de ENET no confirmado | Sección 2.6 — coexisten hipótesis (puerto propio BMW vs. DoIP estándar) sin confirmar para el F20 concreto del usuario | Captura de tráfico propia (Wireshark) en Fase 0B antes de escribir `EnetTransport` real, no basarse solo en fuentes de terceros. |
| DIDs específicos BMW no documentados públicamente de forma fiable | No existe una "hoja de DIDs oficial BMW" pública | Se documentan progresivamente en `docs/protocols/` a medida que se confirman por captura real; ningún DID se implementa "porque se ha visto en un foro" sin verificación propia. |
| Security Access (`0x27`) puede bloquear lecturas que se creían simples | Algunos DIDs de lectura pueden estar protegidos incluso para lectura | Se documenta caso por caso; si un dato de V1 requiere `0x27`, se implementa el mínimo (seed/key de solo lectura conocido/documentado), nunca algoritmos de seguridad orientados a escritura/coding. |
| Diferencias entre ECUs dentro del propio F20 | No todas las ECUs (motor, cuadro, gateway...) implementan el mismo subconjunto de servicios/DIDs | Cada capacidad se prueba y documenta por ECU objetivo, no se asume comportamiento uniforme. |
| Timings reales desconocidos hasta probar con hardware | P2Server/P2*Server exactos del gateway BMW no confirmados | `MockEcu` empieza con valores conservadores de la literatura ISO 14229 y se recalibra tras medir el vehículo real. |

---

## 9. Preguntas que responde este documento (checklist del encargo)

- ✅ ¿Qué servicios UDS necesita realmente la V1? → Sección 3 (`0x10`, `0x19`, `0x22`, `0x3E`, `0x27` condicional).
- ✅ ¿Qué diferencias hay entre un F20 y otros BMW? → Sección 2.3 (KWP2000 en E-Series vs UDS en F-Series) y nota en sección 8 (variación entre ECUs del propio F20).
- ⚠️ ¿Qué mensajes concretos habrá que implementar? → Servicios definidos (sección 3); los DIDs exactos quedan pendientes de captura real (no se puede responder con certeza sin hardware — ver sección 12).
- ✅ ¿Qué se puede probar sin coche? → Sección 6.
- ✅ ¿Qué limitaciones tiene ENET? → Secciones 2.6 y 8.
- ✅ ¿Cómo simular respuestas? → Sección 6 (`MockEcu`).
- ✅ ¿Cómo validar que la implementación es correcta? → Sección 7 (checklist de error) + validación cruzada contra captura real en Fase 0B.

---

## 10. Propuestas de mejora al plan

Estas propuestas **no se aplican directamente** a `PROJECT_PLAN.md` — se documentan aquí para tu aprobación explícita, tal como se pidió.

### 10.1 Insertar una nueva fase "Fase 0B — Investigación y simulador" entre la Fase 0 actual y la Fase 1
- **Qué cambiaría:** renombrar implícitamente el orden — Fase 0 (setup/gobernanza) se mantiene igual; se inserta Fase 0B con tareas: captura de tráfico real (Wireshark) para confirmar sección 2.6, construcción de `MockEcu` + `MockTransport` completos y con escenarios de error, validación de al menos un DID real (VIN) contra el vehículo antes de dar la fase por cerrada.
- **Por qué:** es exactamente lo que se ha pedido en esta conversación — separar investigación/simulación de implementación contra hardware real.
- **Impacto:** Fase 1 pasa a depender de Fase 0B, no directamente de Fase 0. Añade una fase más al roadmap pero reduce fuertemente el riesgo de tener que rehacer `EnetTransport`/`UdsProtocol` a mitad de Fase 1-2.

### 10.2 Norma explícita "ninguna fase depende exclusivamente del vehículo real"
- **Qué cambiaría:** añadir esta norma como texto explícito en la sección 6 de `PROJECT_PLAN.md` (Roadmap), aplicable a Fases 1-4: cada una de ellas debe tener una entrega intermedia "funciona con `MockTransport`" antes de la entrega final "funciona con el vehículo real".
- **Por qué:** permite seguir avanzando incluso sin acceso físico al coche en un momento dado.
- **Impacto:** mínimo en alcance, cambia el criterio de "hecho" de cada fase (se añade un sub-hito intermedio), no cambia arquitectura.

### 10.3 Mover `Security Access (0x27)` de "no mencionado" a "V1 parcial, condicional"
- **Qué cambiaría:** `PROJECT_PLAN.md` no mencionaba explícitamente este servicio. Esta investigación (sección 3) propone incluirlo como posible necesidad de V1, solo para lectura, si algún DID de interés lo exige.
- **Por qué:** omitirlo del todo podría descubrirse tarde como bloqueante (algún dato de lectura podría estar protegido).
- **Impacto:** bajo, pero importante dejarlo escrito para que no sea una sorpresa a mitad de Fase 2 — y para marcar explícitamente que su uso se limita a lectura, nunca como puerta de entrada a escritura/coding.

### 10.4 Añadir `docs/protocols/` como destino vivo de hallazgos de captura real
- **Qué cambiaría:** ya estaba previsto en la estructura de carpetas de `PROJECT_PLAN.md` (sección 4) pero sin contenido — esta investigación confirma que debe empezar a poblarse desde la Fase 0B, no desde Fase 1.
- **Por qué:** los DIDs concretos y el framing confirmado de ENET son el activo más valioso y más difícil de reproducir del proyecto — deben quedar documentados en cuanto se confirman, no solo vivir en el código.
- **Impacto:** ninguno en arquitectura, solo en disciplina de documentación.

---

## 11. Estado de este documento

- **No implementar nada todavía.** Este documento es la base de conocimiento, no luz verde para Fase 1.
- **Pendiente de aprobación del usuario:** tanto el contenido general como las propuestas de la sección 10.
- **Siguiente paso al aprobar:** si se aprueba la propuesta 10.1, actualizar `PROJECT_PLAN.md` insertando la Fase 0B, y solo entonces comenzar su ejecución (empezando por `MockEcu`/`MockTransport`, no por el vehículo real).

---

## 12. Nota de estado (post-aprobación)

Las 4 propuestas de la sección 10 fueron aprobadas por el usuario y ya están incorporadas en `PROJECT_PLAN.md` v3 (Fase 0B insertada entre Fase 0 y Fase 1, norma de no depender del coche real, Security Access resuelto como 12.9, `docs/protocols/` activo desde Fase 0B). Este documento sigue siendo la referencia técnica viva para la Fase 0B.
