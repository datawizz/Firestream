//! State locking mechanism
//!
//! Provides file-based locking for state operations

use crate::core::{FirestreamError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};

/// Lock information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockInfo {
    /// Lock ID
    pub id: String,
    
    /// User holding the lock
    pub user: String,
    
    /// Hostname
    pub hostname: String,
    
    /// Process ID
    pub pid: u32,
    
    /// When the lock was acquired
    pub acquired_at: DateTime<Utc>,
    
    /// Lock expiry time (for stale lock detection)
    pub expires_at: DateTime<Utc>,
    
    /// Purpose of the lock
    pub operation: String,
}

/// State lock handle
pub struct StateLock {
    lock_file: PathBuf,
    info: LockInfo,
}

impl StateLock {
    /// Default lock timeout in seconds
    const LOCK_TIMEOUT_SECS: i64 = 300; // 5 minutes
    
    /// Acquire a state lock
    pub async fn acquire(state_dir: &Path) -> Result<Self> {
        let lock_file = state_dir.join("locks").join("state.lock");
        
        // Check for existing lock
        if lock_file.exists() {
            // Try to read existing lock
            match fs::read_to_string(&lock_file).await {
                Ok(content) => {
                    if let Ok(existing_lock) = serde_json::from_str::<LockInfo>(&content) {
                        // Check if lock is stale
                        if existing_lock.expires_at < Utc::now() {
                            warn!("Found stale lock from {}, removing", existing_lock.user);
                            let _ = fs::remove_file(&lock_file).await;
                        } else {
                            return Err(FirestreamError::GeneralError(format!(
                                "State is locked by {} on {} (PID: {})",
                                existing_lock.user, existing_lock.hostname, existing_lock.pid
                            )));
                        }
                    }
                }
                Err(_) => {
                    // Corrupted lock file, remove it
                    warn!("Found corrupted lock file, removing");
                    let _ = fs::remove_file(&lock_file).await;
                }
            }
        }
        
        // Create new lock
        let info = LockInfo {
            id: uuid::Uuid::new_v4().to_string(),
            user: whoami::username(),
            hostname: whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()),
            pid: std::process::id(),
            acquired_at: Utc::now(),
            expires_at: Utc::now() + chrono::Duration::seconds(Self::LOCK_TIMEOUT_SECS),
            operation: "state_operation".to_string(),
        };
        
        // Write lock file atomically
        let lock_content = serde_json::to_string_pretty(&info)?;
        let temp_file = lock_file.with_extension("tmp");
        
        fs::write(&temp_file, &lock_content).await
            .map_err(|e| FirestreamError::IoError(format!("Failed to write lock file: {}", e)))?;
        
        // Atomic rename
        fs::rename(&temp_file, &lock_file).await
            .map_err(|e| {
                // Clean up temp file
                let _ = fs::remove_file(&temp_file);
                FirestreamError::IoError(format!("Failed to acquire lock: {}", e))
            })?;
        
        info!("Acquired state lock (ID: {})", info.id);
        
        Ok(Self {
            lock_file,
            info,
        })
    }
    
    /// Force acquire a lock (breaks existing lock)
    pub async fn force_acquire(state_dir: &Path) -> Result<Self> {
        let lock_file = state_dir.join("locks").join("state.lock");
        
        // Remove existing lock if present
        if lock_file.exists() {
            warn!("Force-breaking existing lock");
            fs::remove_file(&lock_file).await?;
        }
        
        Self::acquire(state_dir).await
    }
    
    /// Refresh lock expiry time
    pub async fn refresh(&mut self) -> Result<()> {
        self.info.expires_at = Utc::now() + chrono::Duration::seconds(Self::LOCK_TIMEOUT_SECS);
        
        let lock_content = serde_json::to_string_pretty(&self.info)?;
        fs::write(&self.lock_file, lock_content).await?;
        
        debug!("Refreshed lock expiry");
        Ok(())
    }
    
    /// Release the lock
    pub async fn release(self) -> Result<()> {
        fs::remove_file(&self.lock_file).await
            .map_err(|e| FirestreamError::IoError(format!("Failed to release lock: {}", e)))?;
        
        info!("Released state lock (ID: {})", self.info.id);
        Ok(())
    }
    
    /// Check if a lock exists
    pub async fn exists(state_dir: &Path) -> bool {
        let lock_file = state_dir.join("locks").join("state.lock");
        
        if lock_file.exists() {
            // Check if it's stale
            if let Ok(content) = fs::read_to_string(&lock_file).await {
                if let Ok(lock_info) = serde_json::from_str::<LockInfo>(&content) {
                    return lock_info.expires_at > Utc::now();
                }
            }
        }
        
        false
    }
    
    /// Get information about existing lock
    pub async fn get_info(state_dir: &Path) -> Option<LockInfo> {
        let lock_file = state_dir.join("locks").join("state.lock");
        
        if let Ok(content) = fs::read_to_string(&lock_file).await {
            serde_json::from_str::<LockInfo>(&content).ok()
        } else {
            None
        }
    }
}

impl Drop for StateLock {
    fn drop(&mut self) {
        // Try to clean up lock file if dropped without release
        // This is a best-effort cleanup
        let lock_file = self.lock_file.clone();
        let _ = std::fs::remove_file(&lock_file);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    
    #[tokio::test]
    async fn test_lock_acquire_release() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path();
        
        // Create lock directory
        fs::create_dir_all(state_dir.join("locks")).await.unwrap();
        
        // Acquire lock
        let lock = StateLock::acquire(state_dir).await.unwrap();
        assert!(StateLock::exists(state_dir).await);
        
        // Try to acquire again (should fail)
        assert!(StateLock::acquire(state_dir).await.is_err());
        
        // Release lock
        lock.release().await.unwrap();
        assert!(!StateLock::exists(state_dir).await);
        
        // Should be able to acquire again
        let lock2 = StateLock::acquire(state_dir).await.unwrap();
        lock2.release().await.unwrap();
    }
    
    #[tokio::test]
    async fn test_stale_lock_removal() {
        let temp_dir = TempDir::new().unwrap();
        let state_dir = temp_dir.path();
        
        // Create lock directory
        fs::create_dir_all(state_dir.join("locks")).await.unwrap();
        
        // Create a stale lock
        let stale_info = LockInfo {
            id: "test".to_string(),
            user: "test_user".to_string(),
            hostname: "test_host".to_string(),
            pid: 12345,
            acquired_at: Utc::now() - chrono::Duration::hours(1),
            expires_at: Utc::now() - chrono::Duration::minutes(30),
            operation: "test".to_string(),
        };
        
        let lock_file = state_dir.join("locks").join("state.lock");
        fs::write(&lock_file, serde_json::to_string(&stale_info).unwrap()).await.unwrap();
        
        // Should be able to acquire lock (stale lock removed)
        let lock = StateLock::acquire(state_dir).await.unwrap();
        lock.release().await.unwrap();
    }
}
