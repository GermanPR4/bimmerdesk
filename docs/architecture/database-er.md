# Modelo entidad-relación

Vivo — se actualiza en cada migración nueva (`src-tauri/migrations/NNN_*.sql`). Una tabla pertenece siempre a un módulo; ver `PROJECT_PLAN.md` sección 9.

## Migración `001_init`

```
vehicles
├── id            INTEGER PK AUTOINCREMENT
├── vin           TEXT UNIQUE NOT NULL
├── platform      TEXT NOT NULL        -- "F-Series", "E-Series"...
└── created_at    TEXT NOT NULL DEFAULT now()
```

**Propietario:** módulo Vehicle Info. Cualquier otro módulo (Diagnostics, Live Data, Maintenance...) referencia un vehículo únicamente por `vehicles.id`, y solo a través de la API pública de Vehicle Info — nunca con un JOIN directo a esta tabla desde otro módulo.

## Próximas migraciones (a añadir cuando el módulo correspondiente las necesite)

- `diagnostics_history` (módulo Diagnostics, Fase 4).
- `live_data_sessions` (módulo Live Data, Fase 3).
- `maintenance_records` (módulo Maintenance, Fase 6).
- Ficha ampliada de vehículo — fotos/notas/extras (módulo Vehicle Info, Fase 2).
