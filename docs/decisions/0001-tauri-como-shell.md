# 0001. Tauri como shell de escritorio

**Fecha:** 2026-08-01
**Estado:** Aceptada

## Contexto

BMW Toolbox necesita una aplicación de escritorio Windows con: UI moderna (el desarrollador domina React), y una capa de bajo nivel capaz de hablar TCP/serie con timing preciso para protocolos de diagnóstico (UDS/DoIP), donde ventanas de tiempo en milisegundos importan (ver `docs/research/ECU_COMMUNICATION_RESEARCH.md` sección 7).

## Alternativas consideradas

- **Electron (Node.js + Chromium empaquetado):** stack 100% JS/TS, cero curva de lenguaje nuevo. Descartada por: binarios pesados (~150MB+), runtime Node.js con GC no determinista, menos control sobre timing preciso en comunicación serie/socket.
- **Aplicación nativa .NET/WPF:** buen soporte Windows, pero el desarrollador no tiene experiencia en el stack y perdería la reutilización de React.

## Decisión

Tauri 2.x: backend en Rust, WebView nativo del sistema operativo (no empaqueta un motor de renderizado propio). Frontend en React + TypeScript, reutilizando el conocimiento existente del desarrollador.

## Consecuencias

- Binarios ligeros (~10-20MB) y consumo de RAM muy inferior a Electron.
- Rust da control seguro de bajo nivel (sockets, puertos serie) sin runtime con pausas de GC impredecibles — relevante para timings UDS (P2Server/P2*Server).
- Coste: el desarrollador debe aprender Rust para toda la capa `core`. Se acompaña con explicación de conceptos aplicados directamente al código (ver `PROJECT_PLAN.md`, riesgo "curva de aprendizaje de Rust").
- Comunicación UI↔Core vía comandos (`invoke`) y eventos (`emit`/`listen`), sin necesidad de un servidor HTTP local.
