# 0003. SQLite embebido como base de datos

**Fecha:** 2026-08-01
**Estado:** Aceptada

## Contexto

V1 es 100% local, sin cuentas ni sincronización en la nube (ver `PROJECT_PLAN.md` sección 2). Se necesita persistir: histórico de diagnósticos, mantenimiento, sesiones de datos en vivo, y en el futuro la ficha ampliada del vehículo.

## Alternativas consideradas

- **Servidor de base de datos separado (PostgreSQL/MySQL local):** exigiría instalar y gestionar un proceso de servidor en la máquina del usuario, incompatible con "aplicación de escritorio simple, sin backend propio".
- **Archivos planos JSON:** sin transacciones, sin queries relacionales, difícil de escalar con volumen de datos en tiempo real (logging de RPM/sensores a varias muestras por segundo).

## Decisión

SQLite embebido vía `rusqlite` con feature `bundled` (compila SQLite dentro del binario — no depende de que el usuario tenga `sqlite3` instalado en el sistema). Migraciones numeradas y secuenciales, aplicadas mediante un runner propio (`src-tauri/src/db/migrations.rs`), sin librería externa de migraciones.

## Consecuencias

- Un único archivo `.db` portable, transaccional, cero configuración para el usuario.
- Migraciones nunca se editan tras aplicarse — solo se añaden nuevas (PROJECT_PRINCIPLES.md, Principio 12: compatibilidad antes que ruptura).
- Un esquema por módulo: cada módulo es dueño de sus tablas y de su propio `Repository`; el resto de módulos accede solo vía la API pública del módulo dueño, nunca con SQL directo cruzado (`PROJECT_PLAN.md` sección 9).
- Se evaluó y descartó un crate externo de migraciones (ej. `refinery`, `sqlx::migrate!`) por Principio 11 (dependencias antes que comodidad): el runner necesario es simple (una tabla `schema_migrations` + lista ordenada de archivos `.sql` embebidos con `include_str!`) y no justifica una dependencia adicional en esta fase.
