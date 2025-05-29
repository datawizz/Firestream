//! Integration tests for state management

use firestream::state::{StateManager, FirestreamState};
use tempfile::TempDir;

#[tokio::test]
async fn test_state_init_and_load() {
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path().to_path_buf();
    
    // Initialize state
    let mut state_manager = StateManager::new(state_dir.clone());
    state_manager.init().await.unwrap();
    
    // Load state (should create new)
    let state = state_manager.load().await.unwrap();
    assert_eq!(state.serial, 0);
    assert_eq!(state.version, "1.0.0");
}

#[tokio::test]
async fn test_plan_creation() {
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path().to_path_buf();
    
    // Initialize
    let mut state_manager = StateManager::new(state_dir.clone());
    state_manager.init().await.unwrap();
    
    // Create config
    let config = firestream::config::GlobalConfig::default();
    
    // Lock state
    state_manager.lock().await.unwrap();
    
    // Create plan
    let plan = state_manager.plan(&config, vec![]).await.unwrap();
    
    // Should have no changes for empty state and default config
    assert_eq!(plan.changes.len(), 0);
    
    // Unlock
    state_manager.unlock().await.unwrap();
}

#[tokio::test]
async fn test_state_locking() {
    use firestream::state::StateLock;
    
    let temp_dir = TempDir::new().unwrap();
    let state_dir = temp_dir.path();
    
    // Create locks directory
    tokio::fs::create_dir_all(state_dir.join("locks")).await.unwrap();
    
    // Acquire lock
    let lock1 = StateLock::acquire(state_dir).await.unwrap();
    
    // Try to acquire again (should fail)
    assert!(StateLock::acquire(state_dir).await.is_err());
    
    // Release lock
    lock1.release().await.unwrap();
    
    // Should be able to acquire again
    let lock2 = StateLock::acquire(state_dir).await.unwrap();
    lock2.release().await.unwrap();
}
