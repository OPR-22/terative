pub mod adapters;
pub mod application;
pub mod commands;
pub mod domain;

use std::fs;
use std::path::PathBuf;

use commands::{
    client_commands::{client_create, client_delete, client_get, client_list, client_update},
    service_commands::{service_create, service_delete, service_list, service_update},
    settings_commands::{
        settings_get, settings_update_app_preferences, settings_update_currency,
        settings_update_seller_profile,
    },
    AppState,
};
use tauri::Manager;

fn resolve_db_path(app: &tauri::AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_data_dir()
        .expect("resolve app data dir");
    fs::create_dir_all(&dir).expect("create app data dir");
    dir.join("terative.sqlite")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let db_path = resolve_db_path(app.handle());
            let db = adapters::sqlite::open(&db_path)
                .unwrap_or_else(|e| panic!("open sqlite at {db_path:?}: {e}"));
            app.manage(AppState::new(db));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            client_create,
            client_update,
            client_delete,
            client_list,
            client_get,
            service_create,
            service_update,
            service_delete,
            service_list,
            settings_get,
            settings_update_seller_profile,
            settings_update_currency,
            settings_update_app_preferences,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
