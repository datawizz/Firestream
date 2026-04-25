use std::collections::HashMap;
use std::sync::Mutex;
use tauri::State;

pub struct StorageState(pub Mutex<HashMap<String, String>>);

#[tauri::command]
pub fn storage_get(key: String, state: State<StorageState>) -> Option<String> {
    let storage = state.0.lock().unwrap();
    storage.get(&key).cloned()
}

#[tauri::command]
pub fn storage_set(key: String, value: String, state: State<StorageState>) {
    let mut storage = state.0.lock().unwrap();
    storage.insert(key, value);
}

#[tauri::command]
pub fn storage_remove(key: String, state: State<StorageState>) {
    let mut storage = state.0.lock().unwrap();
    storage.remove(&key);
}

#[tauri::command]
pub fn storage_clear(state: State<StorageState>) {
    let mut storage = state.0.lock().unwrap();
    storage.clear();
}
