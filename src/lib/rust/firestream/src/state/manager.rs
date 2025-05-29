//! State management implementation
//!
//! Core state management with plan/apply workflow

use crate::core::{FirestreamError, Result};
use crate::config::GlobalConfig;
use super::schema::*;
use super::lock::StateLock;
use super::diff::StateDiff;
use std::path::PathBuf;
use tokio::fs;
use tracing::{debug, info, error};
use serde_json;

/// State manager handles all state operations
pub struct StateManager {
    /// State directory path
    state_dir: PathBuf,
    
    /// Current state
    current_state: Option<FirestreamState>,
    
    /// State lock
    lock: Option<StateLock>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(state_dir: PathBuf) -> Self {
        Self {
            state_dir,
            current_state: None,
            lock: None,
        }
    }
    
    /// Initialize state directory
    pub async fn init(&self) -> Result<()> {
        fs::create_dir_all(&self.state_dir).await
            .map_err(|e| FirestreamError::IoError(format!("Failed to create state directory: {}", e)))?;
        
        // Create subdirectories
        fs::create_dir_all(self.state_dir.join("plans")).await?;
        fs::create_dir_all(self.state_dir.join("backups")).await?;
        fs::create_dir_all(self.state_dir.join("locks")).await?;
        
        Ok(())
    }
    
    /// Lock state for exclusive access
    pub async fn lock(&mut self) -> Result<()> {
        if self.lock.is_some() {
            return Ok(()); // Already locked
        }
        
        let lock = StateLock::acquire(&self.state_dir).await?;
        self.lock = Some(lock);
        Ok(())
    }
    
    /// Unlock state
    pub async fn unlock(&mut self) -> Result<()> {
        if let Some(lock) = self.lock.take() {
            lock.release().await?;
        }
        Ok(())
    }
    
    /// Load current state from disk
    pub async fn load(&mut self) -> Result<&FirestreamState> {
        let state_file = self.state_dir.join("firestream.state.json");
        
        if state_file.exists() {
            let content = fs::read_to_string(&state_file).await
                .map_err(|e| FirestreamError::IoError(format!("Failed to read state file: {}", e)))?;
            
            let state: FirestreamState = serde_json::from_str(&content)
                .map_err(|e| FirestreamError::GeneralError(format!("Failed to parse state: {}", e)))?;
            
            self.current_state = Some(state);
        } else {
            // Create new state
            info!("No existing state found, creating new state");
            self.current_state = Some(FirestreamState::new());
            self.save().await?;
        }
        
        Ok(self.current_state.as_ref().unwrap())
    }
    
    /// Save current state to disk
    pub async fn save(&self) -> Result<()> {
        let state = self.current_state.as_ref()
            .ok_or_else(|| FirestreamError::GeneralError("No state loaded".to_string()))?;
        
        let state_file = self.state_dir.join("firestream.state.json");
        
        // Create backup of existing state
        if state_file.exists() {
            self.backup_state().await?;
        }
        
        // Write new state
        let content = serde_json::to_string_pretty(state)
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize state: {}", e)))?;
        
        fs::write(&state_file, content).await
            .map_err(|e| FirestreamError::IoError(format!("Failed to write state file: {}", e)))?;
        
        debug!("State saved successfully (serial: {})", state.serial);
        Ok(())
    }
    
    /// Create a backup of current state
    async fn backup_state(&self) -> Result<()> {
        let state_file = self.state_dir.join("firestream.state.json");
        let backup_dir = self.state_dir.join("backups");
        
        if state_file.exists() {
            let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
            let backup_file = backup_dir.join(format!("firestream.state.{}.json", timestamp));
            
            fs::copy(&state_file, &backup_file).await
                .map_err(|e| FirestreamError::IoError(format!("Failed to backup state: {}", e)))?;
            
            debug!("State backed up to {:?}", backup_file);
        }
        
        Ok(())
    }
    
    /// Create execution plan
    pub async fn plan(&mut self, config: &GlobalConfig, targets: Vec<String>) -> Result<ExecutionPlan> {
        // Ensure state is locked
        if self.lock.is_none() {
            return Err(FirestreamError::GeneralError("State must be locked before planning".to_string()));
        }
        
        // Load current state
        self.load().await?;
        
        // Get current state reference
        let current = self.current_state.as_ref()
            .ok_or_else(|| FirestreamError::GeneralError("No state loaded".to_string()))?;
        
        // Create desired state based on config
        let desired = self.create_desired_state(config, current).await?;
        
        // Calculate diff
        let diff = StateDiff::calculate(current, &desired);
        
        // Convert diff to execution plan
        let plan = self.create_execution_plan(diff, current.serial, targets)?;
        
        // Save plan
        self.save_plan(&plan).await?;
        
        info!("Created execution plan with {} changes", plan.changes.len());
        Ok(plan)
    }
    
    /// Apply execution plan
    pub async fn apply(&mut self, plan_id: &str, auto_approve: bool) -> Result<()> {
        // Ensure state is locked
        if self.lock.is_none() {
            return Err(FirestreamError::GeneralError("State must be locked before applying".to_string()));
        }
        
        // Load plan
        let mut plan = self.load_plan(plan_id).await?;
        
        // Check plan status
        if plan.status != PlanStatus::Pending && plan.status != PlanStatus::Approved {
            return Err(FirestreamError::GeneralError(
                format!("Plan {} is not pending or approved", plan_id)
            ));
        }
        
        // Get approval if needed
        if !auto_approve && plan.status == PlanStatus::Pending {
            if !self.get_user_approval(&plan).await? {
                plan.status = PlanStatus::Cancelled;
                self.save_plan(&plan).await?;
                return Err(FirestreamError::GeneralError("Plan cancelled by user".to_string()));
            }
            plan.status = PlanStatus::Approved;
            self.save_plan(&plan).await?;
        }
        
        // Apply changes
        plan.status = PlanStatus::Applying;
        self.save_plan(&plan).await?;
        
        let mut applied_changes = 0;
        let total_changes = plan.changes.len();
        
        for change in &plan.changes {
            info!("Applying change {}/{}: {}", applied_changes + 1, total_changes, change.description);
            
            match self.apply_change(change).await {
                Ok(_) => {
                    applied_changes += 1;
                }
                Err(e) => {
                    error!("Failed to apply change: {}", e);
                    plan.status = PlanStatus::Failed;
                    self.save_plan(&plan).await?;
                    return Err(e);
                }
            }
        }
        
        // Update state serial
        if let Some(state) = &mut self.current_state {
            state.increment_serial();
        }
        
        // Save updated state
        self.save().await?;
        
        // Mark plan as applied
        plan.status = PlanStatus::Applied;
        self.save_plan(&plan).await?;
        
        info!("Successfully applied {} changes", applied_changes);
        Ok(())
    }
    
    /// Create desired state from configuration
    async fn create_desired_state(&self, config: &GlobalConfig, current: &FirestreamState) -> Result<FirestreamState> {
        let mut desired = current.clone();
        
        // Update cluster info
        desired.cluster.name = Some(config.cluster.name.clone());
        desired.cluster.cluster_type = Some("k3d".to_string());
        desired.cluster.managed = true;
        
        // Check if we have k3d configuration in the config file
        // This would need to be added to the GlobalConfig if using TOML config
        // For now, we'll check if there's already k3d config in current state
        // In a real implementation, you'd parse the k3d config from the TOML file
        
        // TODO: Read service configs and update deployments
        // TODO: Read infrastructure config and update infrastructure
        
        Ok(desired)
    }
    
    /// Create execution plan from diff
    fn create_execution_plan(&self, diff: StateDiff, from_serial: u64, targets: Vec<String>) -> Result<ExecutionPlan> {
        let mut changes = Vec::new();
        
        // Convert diff to planned changes
        for (resource_type, resource_id, change_type, old, new) in diff.changes {
            let change = PlannedChange {
                id: uuid::Uuid::new_v4().to_string(),
                resource_type,
                resource_id: resource_id.clone(),
                change_type,
                description: self.describe_change(&resource_type, &resource_id, &change_type),
                old_value: old,
                new_value: new,
                requires_confirmation: matches!(change_type, ChangeType::Delete | ChangeType::Replace),
                impact: self.assess_impact(&resource_type, &change_type),
            };
            
            // Apply targeting filter
            if !targets.is_empty() && !targets.contains(&resource_id) {
                continue;
            }
            
            changes.push(change);
        }
        
        let plan = ExecutionPlan {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: chrono::Utc::now(),
            created_by: whoami::username(),
            changes,
            from_serial,
            to_serial: from_serial + 1,
            status: PlanStatus::Pending,
            options: PlanOptions::default(),
        };
        
        Ok(plan)
    }
    
    /// Apply a single change
    async fn apply_change(&self, change: &PlannedChange) -> Result<()> {
        match (&change.resource_type, &change.change_type) {
            (ResourceType::Infrastructure, _) => {
                self.apply_infrastructure_change(change).await?;
            }
            (ResourceType::Build, _) => {
                self.apply_build_change(change).await?;
            }
            (ResourceType::Deployment, _) => {
                self.apply_deployment_change(change).await?;
            }
            (ResourceType::Cluster, _) => {
                self.apply_cluster_change(change).await?;
            }
        }
        
        Ok(())
    }
    
    /// Apply infrastructure change (calls Python Pulumi)
    async fn apply_infrastructure_change(&self, change: &PlannedChange) -> Result<()> {
        info!("Applying infrastructure change: {}", change.description);
        
        // Prepare Pulumi config
        let pulumi_config = self.prepare_pulumi_config(change)?;
        
        // Call Python Pulumi entrypoint
        let _result = self.call_pulumi_entrypoint(&pulumi_config).await?;
        
        // Update state with Pulumi outputs
        if let Some(_state) = &self.current_state {
            // TODO: Parse Pulumi result and update infrastructure state
        }
        
        Ok(())
    }
    
    /// Prepare configuration for Pulumi
    fn prepare_pulumi_config(&self, change: &PlannedChange) -> Result<serde_json::Value> {
        let config = serde_json::json!({
            "operation": change.change_type,
            "resource_id": change.resource_id,
            "old_value": change.old_value,
            "new_value": change.new_value,
            "state_dir": self.state_dir,
        });
        
        Ok(config)
    }
    
    /// Call Python Pulumi entrypoint
    async fn call_pulumi_entrypoint(&self, config: &serde_json::Value) -> Result<serde_json::Value> {
        use tokio::process::Command;
        
        // Write config to temp file
        let config_file = self.state_dir.join("pulumi_config.json");
        let config_str = serde_json::to_string_pretty(config)?;
        fs::write(&config_file, config_str).await?;
        
        // Call Python script
        let output = Command::new("python3")
            .arg("infrastructure/pulumi_entrypoint.py")
            .arg(&config_file)
            .output()
            .await
            .map_err(|e| FirestreamError::GeneralError(format!("Failed to execute Pulumi: {}", e)))?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(FirestreamError::GeneralError(format!("Pulumi failed: {}", stderr)));
        }
        
        // Parse output
        let stdout = String::from_utf8_lossy(&output.stdout);
        let result: serde_json::Value = serde_json::from_str(&stdout)?;
        
        // Clean up config file
        let _ = fs::remove_file(&config_file).await;
        
        Ok(result)
    }
    
    /// Apply build change
    async fn apply_build_change(&self, change: &PlannedChange) -> Result<()> {
        info!("Applying build change: {}", change.description);
        
        // TODO: Implement actual build logic
        match change.change_type {
            ChangeType::Create | ChangeType::Update => {
                // Build container image
                // builder::build_image(...).await?;
            }
            ChangeType::Delete => {
                // Remove container image
                // docker::remove_image(...).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Apply deployment change
    async fn apply_deployment_change(&self, change: &PlannedChange) -> Result<()> {
        info!("Applying deployment change: {}", change.description);
        
        match change.change_type {
            ChangeType::Create => {
                // Install Helm chart
                // helm::install_chart(...).await?;
            }
            ChangeType::Update => {
                // Upgrade Helm release
                // helm::upgrade_release(...).await?;
            }
            ChangeType::Delete => {
                // Uninstall Helm release
                // helm::uninstall_release(...).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Apply cluster change
    async fn apply_cluster_change(&self, change: &PlannedChange) -> Result<()> {
        use crate::deploy::{k3d, k3d_advanced::K3dClusterManager};
        
        info!("Applying cluster change: {}", change.description);
        
        match change.change_type {
            ChangeType::Create => {
                // Check if we have k3d configuration
                if let Some(state) = &self.current_state {
                    if let Some(k3d_config) = &state.cluster.k3d_config {
                        // Use advanced k3d manager
                        let manager = K3dClusterManager::new(k3d_config.clone());
                        manager.setup_cluster().await?;
                    } else {
                        // Fall back to simple setup
                        k3d::setup_cluster().await?;
                    }
                }
            }
            ChangeType::Update => {
                // Handle k3d cluster updates
                if let Some(new_config) = &change.new_value {
                    if let Ok(k3d_config) = serde_json::from_value::<crate::state::schema::K3dClusterConfig>(new_config.clone()) {
                        let manager = K3dClusterManager::new(k3d_config);
                        // For updates, we might need to recreate the cluster
                        // or apply specific changes
                        manager.setup_cluster().await?;
                    }
                }
            }
            ChangeType::Delete => {
                // Delete K3D cluster
                if let Some(state) = &self.current_state {
                    if let Some(k3d_config) = &state.cluster.k3d_config {
                        let manager = K3dClusterManager::new(k3d_config.clone());
                        manager.delete_cluster().await?;
                    } else if let Some(name) = &state.cluster.name {
                        k3d::delete_cluster(name).await?;
                    }
                }
            }
            _ => {}
        }
        
        Ok(())
    }
    
    /// Describe a change in human-readable format
    fn describe_change(&self, resource_type: &ResourceType, resource_id: &str, change_type: &ChangeType) -> String {
        match change_type {
            ChangeType::Create => format!("Create {} '{}'", resource_type_name(resource_type), resource_id),
            ChangeType::Update => format!("Update {} '{}'", resource_type_name(resource_type), resource_id),
            ChangeType::Delete => format!("Delete {} '{}'", resource_type_name(resource_type), resource_id),
            ChangeType::Replace => format!("Replace {} '{}'", resource_type_name(resource_type), resource_id),
            ChangeType::NoOp => format!("No changes to {} '{}'", resource_type_name(resource_type), resource_id),
        }
    }
    
    /// Assess impact of a change
    fn assess_impact(&self, resource_type: &ResourceType, change_type: &ChangeType) -> ChangeImpact {
        match (resource_type, change_type) {
            (_, ChangeType::NoOp) => ChangeImpact::None,
            (ResourceType::Infrastructure, ChangeType::Delete) => ChangeImpact::Critical,
            (ResourceType::Infrastructure, _) => ChangeImpact::High,
            (ResourceType::Deployment, ChangeType::Delete) => ChangeImpact::High,
            (ResourceType::Deployment, _) => ChangeImpact::Medium,
            (ResourceType::Build, _) => ChangeImpact::Low,
            (ResourceType::Cluster, _) => ChangeImpact::Critical,
        }
    }
    
    /// Get user approval for plan
    async fn get_user_approval(&self, plan: &ExecutionPlan) -> Result<bool> {
        use std::io::{self, Write};
        
        println!("\nExecution Plan:");
        println!("===============");
        println!("Plan ID: {}", plan.id);
        println!("Changes: {}", plan.changes.len());
        println!();
        
        for (_i, change) in plan.changes.iter().enumerate() {
            let symbol = match change.change_type {
                ChangeType::Create => "+",
                ChangeType::Update => "~",
                ChangeType::Delete => "-",
                ChangeType::Replace => "!",
                ChangeType::NoOp => " ",
            };
            
            let color = match change.impact {
                ChangeImpact::None => "",
                ChangeImpact::Low => "\x1b[32m",      // Green
                ChangeImpact::Medium => "\x1b[33m",    // Yellow
                ChangeImpact::High => "\x1b[31m",      // Red
                ChangeImpact::Critical => "\x1b[35m",  // Magenta
            };
            
            println!("  {}{} {}\x1b[0m", color, symbol, change.description);
        }
        
        println!();
        print!("Do you want to apply these changes? (yes/no): ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        
        Ok(input.trim().to_lowercase() == "yes" || input.trim().to_lowercase() == "y")
    }
    
    /// Save plan to disk
    async fn save_plan(&self, plan: &ExecutionPlan) -> Result<()> {
        let plan_file = self.state_dir.join("plans").join(format!("{}.json", plan.id));
        
        let content = serde_json::to_string_pretty(plan)?;
        fs::write(&plan_file, content).await?;
        
        Ok(())
    }
    
    /// Load plan from disk
    async fn load_plan(&self, plan_id: &str) -> Result<ExecutionPlan> {
        let plan_file = self.state_dir.join("plans").join(format!("{}.json", plan_id));
        
        let content = fs::read_to_string(&plan_file).await?;
        let plan: ExecutionPlan = serde_json::from_str(&content)?;
        
        Ok(plan)
    }
    
    /// Refresh state from actual resources
    pub async fn refresh(&mut self) -> Result<()> {
        info!("Refreshing state from actual resources");
        
        // Ensure state is locked
        if self.lock.is_none() {
            return Err(FirestreamError::GeneralError("State must be locked before refreshing".to_string()));
        }
        
        let _state = self.load().await?;
        
        // TODO: Refresh infrastructure state from Pulumi
        // TODO: Refresh deployment state from Kubernetes
        // TODO: Refresh build state from registry
        
        self.save().await?;
        
        Ok(())
    }
    
    /// Import existing resources into state
    pub async fn import(&mut self, resource_type: ResourceType, resource_id: String, _resource_data: serde_json::Value) -> Result<()> {
        info!("Importing {} '{}'", resource_type_name(&resource_type), resource_id);
        
        // Ensure state is locked
        if self.lock.is_none() {
            return Err(FirestreamError::GeneralError("State must be locked before importing".to_string()));
        }
        
        let state = self.current_state.as_mut()
            .ok_or_else(|| FirestreamError::GeneralError("No state loaded".to_string()))?;
        
        match resource_type {
            ResourceType::Infrastructure => {
                // TODO: Import infrastructure resource
            }
            ResourceType::Build => {
                // TODO: Import build
            }
            ResourceType::Deployment => {
                // TODO: Import deployment
            }
            ResourceType::Cluster => {
                // TODO: Import cluster
            }
        }
        
        state.increment_serial();
        self.save().await?;
        
        Ok(())
    }
    
    /// Update cluster configuration in state
    pub async fn update_cluster_state(&mut self, k3d_config: Option<K3dClusterConfig>) -> Result<()> {
        // Load current state if not already loaded
        if self.current_state.is_none() {
            self.load().await?;
        }
        
        let state = self.current_state.as_mut()
            .ok_or_else(|| FirestreamError::GeneralError("No state loaded".to_string()))?;
        
        if let Some(config) = k3d_config {
            // Set cluster configuration
            state.cluster.name = Some(config.name.clone());
            state.cluster.cluster_type = Some("k3d".to_string());
            state.cluster.managed = true;
            state.cluster.k3d_config = Some(config);
        } else {
            // Clear cluster configuration
            state.cluster.name = None;
            state.cluster.cluster_type = None;
            state.cluster.managed = false;
            state.cluster.k3d_config = None;
        }
        
        state.increment_serial();
        self.save().await?;
        
        Ok(())
    }
}

/// Get human-readable resource type name
fn resource_type_name(rt: &ResourceType) -> &'static str {
    match rt {
        ResourceType::Infrastructure => "infrastructure",
        ResourceType::Build => "build",
        ResourceType::Deployment => "deployment",
        ResourceType::Cluster => "cluster",
    }
}
