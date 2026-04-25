// Prevents console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use commands::storage::{StorageState, storage_get, storage_set, storage_remove, storage_clear};
use commands::secure_storage::{secure_storage_get, secure_storage_set, secure_storage_remove, secure_storage_has};
use std::collections::HashMap;
use std::sync::Mutex;

fn main() {
    tauri::Builder::default()
        .manage(StorageState(Mutex::new(HashMap::new())))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_deep_link::init())
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
