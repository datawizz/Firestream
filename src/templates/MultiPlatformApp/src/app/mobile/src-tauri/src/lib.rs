mod commands;

use commands::storage::{StorageState, storage_get, storage_set, storage_remove, storage_clear};
use commands::secure_storage::{secure_storage_get, secure_storage_set, secure_storage_remove, secure_storage_has};
use std::collections::HashMap;
use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // NOTE: tauri_plugin_deep_link::init() removed temporarily to isolate crash.
    // The plugin panics on iOS if CFBundleURLTypes isn't configured correctly.
    // Re-add once the app boots successfully.
    tauri::Builder::default()
        .manage(StorageState(Mutex::new(HashMap::new())))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            storage_get,
            storage_set,
            storage_remove,
            storage_clear,
            secure_storage_get,
            secure_storage_set,
            secure_storage_remove,
            secure_storage_has,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
