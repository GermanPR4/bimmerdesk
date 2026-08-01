# 0004. Command Bus interno tipado

**Fecha:** 2026-08-01
**Estado:** Aceptada

## Contexto

Con la arquitectura de módulos (ADR 0002), los módulos necesitan poder disparar acciones que otro módulo ejecuta sin acoplarse directamente a su tipo concreto (evita que `Dashboard` importe y llame directamente funciones internas de `Diagnostics`, por ejemplo). A medio plazo se prevén cientos de acciones distintas.

## Alternativas consideradas

- **Bus genérico basado en strings** (`bus.dispatch("scan_dtcs", json!({...}))`): flexible, pero pierde toda garantía del compilador — un typo en el nombre del comando o un payload mal formado solo se detecta en runtime. Descartado explícitamente por PROJECT_PLAN.md ("Comunicación entre módulos: Command Bus interno, tipado, no genérico por strings").
- **Llamadas directas entre módulos** (un módulo importa el crate/tipo de otro y lo invoca directamente): descartado — rompe la regla de frontera de ADR 0002 y hace imposible sustituir un módulo sin tocar quien lo llama.

## Decisión

`CommandBus` con un trait `Command` (`type Output`) y `dispatch::<C>(command) -> Result<C::Output, BusError>`. Internamente usa `TypeId`/`Any` para almacenar handlers heterogéneos en un mapa, pero cada punto de llamada es completamente tipado — el compilador verifica que el comando y su tipo de salida coinciden, no hay strings mágicos.

## Consecuencias

- Un módulo publica un comando tipado; otro registra el handler; ninguno conoce el tipo concreto del otro más allá del propio tipo de comando (que puede vivir en un crate/módulo de "contratos" compartido si hiciera falta más adelante).
- Coste: `TypeId`/`Any` internamente implica algo de "magia" de tipos — documentado aquí y en los comentarios del propio `command_bus/mod.rs` para que no sorprenda a quien lo lea dentro de cinco años (PROJECT_PRINCIPLES.md, prueba del "dentro de cinco años").
- No se ha añadido ninguna dependencia externa para esto (Principio 11) — `std::any` es suficiente para el alcance actual.
