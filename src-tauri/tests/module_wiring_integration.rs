//! Plantilla de test de integración (Fase 0): verifica el cableado real
//! CommandBus -> SQLite (archivo temporal, no in-memory) + Transport, en
//! lugar de piezas aisladas. Vive en `src-tauri/tests/` (ubicación idiomática
//! de Cargo para integration tests, que compilan el crate como caja negra)
//! — distinto del `tests/unit/` y `tests/ui/` en la raíz del repo, que son
//! del lado TypeScript. Ver PROJECT_PLAN.md sección 8.
//!
//! Todavía no hay un módulo de negocio real (llega en Fase 2+), así que esto
//! demuestra el patrón — bus tipado despachando un comando que persiste en
//! una base de datos real — sin depender del vehículo.

use bmw_toolbox_lib::command_bus::{Command, CommandBus};
use bmw_toolbox_lib::db;
use bmw_toolbox_lib::transport::mock::MockTransport;
use bmw_toolbox_lib::transport::Transport;

struct RegisterVehicleCommand {
    vin: String,
    platform: String,
}

impl Command for RegisterVehicleCommand {
    type Output = i64;
}

#[test]
fn command_bus_dispatch_persists_through_real_sqlite_file() {
    let tmp_dir = std::env::temp_dir();
    let db_path = tmp_dir.join(format!("bmw_toolbox_test_{}.db", std::process::id()));
    let db_path_str = db_path.to_str().unwrap().to_string();

    let conn = db::open_and_migrate(&db_path_str).expect("migrations should apply cleanly");

    let mut bus = CommandBus::new();
    let conn = std::sync::Mutex::new(conn);
    bus.register::<RegisterVehicleCommand, _>(move |cmd| {
        let conn = conn.lock().unwrap();
        conn.execute(
            "INSERT INTO vehicles (vin, platform) VALUES (?1, ?2)",
            rusqlite::params![cmd.vin, cmd.platform],
        )
        .expect("insert should succeed");
        conn.last_insert_rowid()
    });

    let id = bus
        .dispatch(RegisterVehicleCommand {
            vin: "WBA00000000000000".to_string(),
            platform: "F-Series".to_string(),
        })
        .expect("dispatch should find the registered handler");

    assert_eq!(id, 1);

    std::fs::remove_file(&db_path).ok();
}

#[test]
fn mock_transport_requires_explicit_connect_before_use() {
    let mut transport = MockTransport::new();
    assert!(!transport.is_connected());
    transport.connect().unwrap();
    assert!(transport.is_connected());
}
