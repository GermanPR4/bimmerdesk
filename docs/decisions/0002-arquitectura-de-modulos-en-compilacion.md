# 0002. Arquitectura de módulos registrados en tiempo de compilación (no plugins dinámicos)

**Fecha:** 2026-08-01
**Estado:** Aceptada

## Contexto

El proyecto necesita que cada área funcional (Vehicle Info, Live Data, Diagnostics, Maintenance, Reports, AI, Coding, ECU Flash...) sea una unidad desacoplada: sustituible, testeable de forma aislada, y que no pueda "hacer trampas" accediendo directamente al estado interno de otra (PROJECT_PRINCIPLES.md).

## Alternativas consideradas

- **Plugins dinámicos verdaderos** (`.dll`/`.so` cargados en runtime vía `libloading`, instalables sin recompilar — lo que sugeriría literalmente la palabra "plugin" o un "marketplace interno"). Descartada para V1: Rust no tiene ABI estable entre versiones de compilador ni entre distintos flags de compilación. Cargar código así es una fuente conocida de crashes difíciles de depurar y de incompatibilidades silenciosas.
- **Todo en un único crate sin fronteras internas** ("un módulo grande"). Descartada: viola directamente el principio de modularidad — imposible sustituir o testear una pieza sin arrastrar el resto.

## Decisión

Cada módulo implementa un trait `Module` común (`manifest()`, `register()`, `on_startup()`) y se registra explícitamente en una lista en `main.rs`/`lib.rs` (`ModuleRegistry`), en tiempo de compilación. Añadir un módulo nuevo es: crear su carpeta implementando `Module` + añadir una línea a la lista de módulos activos — sin descubrimiento dinámico.

## Consecuencias

- Mismo desacoplamiento (frontera clara, sustituible, testeable) que un sistema de plugins real, sin el riesgo de ABI inestable.
- Un módulo nuevo requiere recompilar la aplicación — aceptable porque no hay marketplace real de terceros en V1 ni previsto a corto plazo.
- Camino de evolución explícito si algún día se necesitan plugins de terceros instalables sin recompilar: runtime WASM (ej. Wasmtime, con Component Model/WASI como interfaz de sandbox). Queda anotado en `PROJECT_PLAN.md` como Post-V1, no se persigue ahora.
