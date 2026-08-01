# Convenciones de desarrollo

Detalle práctico de lo ya decidido en `PROJECT_PLAN.md` sección 10.2. Léase junto a `PROJECT_PRINCIPLES.md` antes de tocar código.

## Ramas

- `main` siempre desplegable — no rompe la build, no tiene tests en rojo.
- Trabajo en curso en `feature/<modulo>-<descripcion>` (ej. `feature/diagnostics-read-dtcs`).
- Sin commits directos a `main` una vez el CI esté activo (ya lo está desde Fase 0) — todo pasa por PR, aunque el desarrollo sea individual: el CI es la red de seguridad contra regresiones.

## Commits

[Conventional Commits](https://www.conventionalcommits.org/): `feat:`, `fix:`, `refactor:`, `docs:`, `test:`, `chore:`. Cuerpo del commit explica el *por qué*, no el *qué* (el diff ya dice qué cambió). Commits pequeños y frecuentes — cada uno debe dejar el árbol compilando y con tests en verde.

## Versionado

[SemVer](https://semver.org/) (`MAJOR.MINOR.PATCH`) desde la primera release interna (`0.1.0`). V1 completo no llega a `1.0.0` hasta cerrar Fase 11 — antes de eso, todo es `0.x.y`.

## Releases

Tag Git por versión + GitHub Release con el instalador `.msi` adjunto. Manual en V1 (pipeline de publicación automática se activa en Fase 11, ver `PROJECT_PLAN.md` 10.3).

## Antes de cada commit

- `npm run lint && npx tsc --noEmit && npm run test` (frontend).
- `cargo clippy --all-targets -- -D warnings && cargo test` en `src-tauri/` (backend).
- El mismo CI (`.github/workflows/ci.yml`) corre esto en cada push/PR — correrlo en local antes evita descubrir el fallo en GitHub.

## Antes de dar una fase por cerrada

Ver `PROJECT_PLAN.md` sección 8 (estrategia de testing) y el "Objetivo medible de V1" (sección 1.1) — "funciona en el caso feliz" no es el criterio de aceptación.
