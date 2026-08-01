// pub: los tests de integración en src-tauri/tests/ compilan el crate como
// caja negra y necesitan acceso a estos módulos (ver
// module_wiring_integration.rs).
pub mod command_bus;
pub mod db;
pub mod domain;
pub mod module;
pub mod protocol;
pub mod transport;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
