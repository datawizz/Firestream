use keyring::Entry;
use thiserror::Error;

const SERVICE_NAME: &str = "com.example.multiplatformapp";

#[derive(Debug, Error)]
pub enum SecureStorageError {
    #[error("Failed to access keyring: {0}")]
    KeyringError(#[from] keyring::Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}

impl serde::Serialize for SecureStorageError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

fn get_entry(key: &str) -> Result<Entry, SecureStorageError> {
    Entry::new(SERVICE_NAME, key).map_err(SecureStorageError::from)
}

#[tauri::command]
pub async fn secure_storage_set(key: String, value: String) -> Result<(), SecureStorageError> {
    let entry = get_entry(&key)?;
    entry.set_password(&value)?;
    Ok(())
}

#[tauri::command]
pub async fn secure_storage_get(key: String) -> Result<String, SecureStorageError> {
    let entry = get_entry(&key)?;
    entry.get_password().map_err(|e| {
        if matches!(e, keyring::Error::NoEntry) {
            SecureStorageError::KeyNotFound(key)
        } else {
            SecureStorageError::from(e)
        }
    })
}

#[tauri::command]
pub async fn secure_storage_remove(key: String) -> Result<(), SecureStorageError> {
    let entry = get_entry(&key)?;
    entry.delete_credential()?;
    Ok(())
}

#[tauri::command]
pub async fn secure_storage_has(key: String) -> Result<bool, SecureStorageError> {
    let entry = get_entry(&key)?;
    match entry.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(e) => Err(SecureStorageError::from(e)),
    }
}
