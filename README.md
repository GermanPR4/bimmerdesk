# BMW Toolbox

Aplicación de escritorio de diagnóstico, monitorización y mantenimiento de vehículos. V1 centrada en BMW F-Series, solo lectura — no escribe nada en ninguna centralita.

Ver [`PROJECT_PLAN.md`](PROJECT_PLAN.md) (arquitectura, roadmap) y [`PROJECT_PRINCIPLES.md`](PROJECT_PRINCIPLES.md) (principios de desarrollo) antes de contribuir.

## Stack

Tauri 2 (Rust) + React + TypeScript + Vite + SQLite.

## Desarrollo

```
npm install
npm run tauri dev
```

Requiere [Rust](https://www.rust-lang.org/tools/install) instalado (`cargo`/`rustc` en el `PATH`).

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
