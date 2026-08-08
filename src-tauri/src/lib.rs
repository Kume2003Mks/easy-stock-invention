use std::sync::Mutex;
use tauri::Manager;

pub mod adapters;
pub mod domain;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            let conn = adapters::persistence::database::init_db(app_data_dir)
                .expect("Failed to initialize database");

            // Store the Mutex-wrapped connection in Tauri state
            app.manage(Mutex::new(conn));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            adapters::commands::get_settings,
            adapters::commands::save_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
