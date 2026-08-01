# 0005. Permisos entre módulos como manifest declarativo

**Fecha:** 2026-08-01
**Estado:** Aceptada

## Contexto

Un proyecto con módulos que tocan ECU, red, filesystem e IA necesita que quede claro y auditable qué puede tocar cada uno — especialmente crítico dado que V1 promete no escribir nunca en ninguna centralita (PROJECT_PLAN.md sección 1).

## Alternativas consideradas

- **Sandboxing real** (cada módulo en su propio proceso/hilo aislado, con permisos del sistema operativo o un runtime tipo WASM/WASI limitando syscalls): mucho más fuerte como garantía de seguridad, pero es sobreingeniería para V1 con un único desarrollador y sin superficie de amenaza de terceros todavía (no hay módulos de terceros instalables — ver ADR 0002). Descartado por ahora; queda como evolución natural si algún día se añaden plugins WASM reales.
- **Ningún control, confianza implícita en el código:** descartado — no deja rastro auditable de qué módulo pidió qué capacidad, y no ayuda a detectar un módulo futuro que empiece a pedir algo que no debería (ej. `needs_ecu_write`).

## Decisión

Cada módulo declara sus capacidades necesarias en su `ModuleManifest` (`needs_ecu`, `needs_sqlite`, `needs_network`, `needs_filesystem`, `needs_ai`, `needs_usb`, representadas como variantes del enum `Capability`). El `ModuleRegistry` las recopila al arrancar; son visibles y auditables (`registry.manifests()`), pero no se aplican mediante sandboxing en proceso separado.

## Consecuencias

- Cualquier módulo futuro que pidiera una capacidad de escritura sobre la ECU sería una señal explícita y revisable en su manifest — nunca un cambio de comportamiento silencioso enterrado en el código.
- No hay enforcement técnico real (un módulo *podría* técnicamente saltarse su propio manifest); la garantía es de disciplina de arquitectura y revisión, no de aislamiento de sistema operativo. Esto es aceptable para V1: documentado explícitamente para no generar falsa sensación de sandboxing real.
- Si en el futuro se añaden módulos de terceros no confiables (más allá de los propios del proyecto), esta decisión debe revisarse — ver ADR 0002, evolución hacia WASM/WASI como punto donde el sandboxing real pasaría a ser necesario.
