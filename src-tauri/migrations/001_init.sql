-- Vehicles: owned by the Vehicle Info module. Other modules reference a
-- vehicle only by its id, through Vehicle Info's public API — never with a
-- direct cross-module JOIN. See PROJECT_PLAN.md section 9.
CREATE TABLE vehicles (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    vin TEXT NOT NULL UNIQUE,
    platform TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
