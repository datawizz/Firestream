//! State diffing and comparison
//!
//! Calculate differences between states for plan generation

use crate::state::schema::*;
use serde_json::Value;
use std::collections::HashMap;

/// State difference representation
pub struct StateDiff {
    /// List of changes (resource_type, resource_id, change_type, old_value, new_value)
    pub changes: Vec<(ResourceType, String, ChangeType, Option<Value>, Option<Value>)>,
}

impl StateDiff {
    /// Calculate diff between current and desired states
    pub fn calculate(current: &FirestreamState, desired: &FirestreamState) -> Self {
        let mut changes = Vec::new();
        
        // Compare infrastructure
        changes.extend(Self::diff_infrastructure(&current.infrastructure, &desired.infrastructure));
        
        // Compare builds
        changes.extend(Self::diff_builds(&current.builds, &desired.builds));
        
        // Compare deployments
        changes.extend(Self::diff_deployments(&current.deployments, &desired.deployments));
        
        // Compare cluster
        changes.extend(Self::diff_cluster(&current.cluster, &desired.cluster));
        
        Self { changes }
    }
    
    /// Diff infrastructure state
    fn diff_infrastructure(current: &InfrastructureState, desired: &InfrastructureState) -> Vec<(ResourceType, String, ChangeType, Option<Value>, Option<Value>)> {
        let mut changes = Vec::new();
        
        // Check if infrastructure needs to be created
        if current.stack_name.is_none() && desired.stack_name.is_some() {
            changes.push((
                ResourceType::Infrastructure,
                desired.stack_name.clone().unwrap(),
                ChangeType::Create,
                None,
                Some(serde_json::to_value(desired).unwrap()),
            ));
        }
        // Check if infrastructure needs to be updated
        else if current.stack_name.is_some() && desired.stack_name.is_some() {
            if serde_json::to_value(current).unwrap() != serde_json::to_value(desired).unwrap() {
                changes.push((
                    ResourceType::Infrastructure,
                    desired.stack_name.clone().unwrap(),
                    ChangeType::Update,
                    Some(serde_json::to_value(current).unwrap()),
                    Some(serde_json::to_value(desired).unwrap()),
                ));
            }
        }
        // Check if infrastructure needs to be deleted
        else if current.stack_name.is_some() && desired.stack_name.is_none() {
            changes.push((
                ResourceType::Infrastructure,
                current.stack_name.clone().unwrap(),
                ChangeType::Delete,
                Some(serde_json::to_value(current).unwrap()),
                None,
            ));
        }
        
        changes
    }
    
    /// Diff builds
    fn diff_builds(current: &HashMap<String, BuildState>, desired: &HashMap<String, BuildState>) -> Vec<(ResourceType, String, ChangeType, Option<Value>, Option<Value>)> {
        let mut changes = Vec::new();
        
        // Check for new builds
        for (name, desired_build) in desired {
            if !current.contains_key(name) {
                changes.push((
                    ResourceType::Build,
                    name.clone(),
                    ChangeType::Create,
                    None,
                    Some(serde_json::to_value(desired_build).unwrap()),
                ));
            } else {
                let current_build = &current[name];
                if current_build.tag != desired_build.tag || current_build.build_config != desired_build.build_config {
                    changes.push((
                        ResourceType::Build,
                        name.clone(),
                        ChangeType::Update,
                        Some(serde_json::to_value(current_build).unwrap()),
                        Some(serde_json::to_value(desired_build).unwrap()),
                    ));
                }
            }
        }
        
        // Check for removed builds
        for (name, current_build) in current {
            if !desired.contains_key(name) {
                changes.push((
                    ResourceType::Build,
                    name.clone(),
                    ChangeType::Delete,
                    Some(serde_json::to_value(current_build).unwrap()),
                    None,
                ));
            }
        }
        
        changes
    }
    
    /// Diff deployments
    fn diff_deployments(current: &HashMap<String, DeploymentState>, desired: &HashMap<String, DeploymentState>) -> Vec<(ResourceType, String, ChangeType, Option<Value>, Option<Value>)> {
        let mut changes = Vec::new();
        
        // Check for new deployments
        for (name, desired_deployment) in desired {
            if !current.contains_key(name) {
                changes.push((
                    ResourceType::Deployment,
                    name.clone(),
                    ChangeType::Create,
                    None,
                    Some(serde_json::to_value(desired_deployment).unwrap()),
                ));
            } else {
                let current_deployment = &current[name];
                
                // Check if deployment needs update
                let needs_update = current_deployment.chart != desired_deployment.chart
                    || current_deployment.chart_version != desired_deployment.chart_version
                    || current_deployment.values != desired_deployment.values
                    || current_deployment.namespace != desired_deployment.namespace;
                
                if needs_update {
                    // Determine if it's an update or replace
                    let change_type = if current_deployment.chart != desired_deployment.chart {
                        ChangeType::Replace
                    } else {
                        ChangeType::Update
                    };
                    
                    changes.push((
                        ResourceType::Deployment,
                        name.clone(),
                        change_type,
                        Some(serde_json::to_value(current_deployment).unwrap()),
                        Some(serde_json::to_value(desired_deployment).unwrap()),
                    ));
                }
            }
        }
        
        // Check for removed deployments
        for (name, current_deployment) in current {
            if !desired.contains_key(name) {
                changes.push((
                    ResourceType::Deployment,
                    name.clone(),
                    ChangeType::Delete,
                    Some(serde_json::to_value(current_deployment).unwrap()),
                    None,
                ));
            }
        }
        
        changes
    }
    
    /// Diff cluster state
    fn diff_cluster(current: &ClusterState, desired: &ClusterState) -> Vec<(ResourceType, String, ChangeType, Option<Value>, Option<Value>)> {
        let mut changes = Vec::new();
        
        // Only consider managed clusters
        if desired.managed {
            if current.name.is_none() && desired.name.is_some() {
                changes.push((
                    ResourceType::Cluster,
                    desired.name.clone().unwrap(),
                    ChangeType::Create,
                    None,
                    Some(serde_json::to_value(desired).unwrap()),
                ));
            } else if current.name.is_some() && desired.name.is_some() {
                let needs_update = current.cluster_type != desired.cluster_type
                    || current.kubernetes_version != desired.kubernetes_version;
                
                if needs_update {
                    changes.push((
                        ResourceType::Cluster,
                        desired.name.clone().unwrap(),
                        ChangeType::Update,
                        Some(serde_json::to_value(current).unwrap()),
                        Some(serde_json::to_value(desired).unwrap()),
                    ));
                }
            }
        }
        
        changes
    }
    
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }
    
    /// Get summary of changes
    pub fn summary(&self) -> String {
        let mut creates = 0;
        let mut updates = 0;
        let mut deletes = 0;
        let mut replaces = 0;
        
        for (_, _, change_type, _, _) in &self.changes {
            match change_type {
                ChangeType::Create => creates += 1,
                ChangeType::Update => updates += 1,
                ChangeType::Delete => deletes += 1,
                ChangeType::Replace => replaces += 1,
                ChangeType::NoOp => {}
            }
        }
        
        format!(
            "{} to create, {} to update, {} to delete, {} to replace",
            creates, updates, deletes, replaces
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_empty_diff() {
        let state1 = FirestreamState::new();
        let state2 = state1.clone();
        
        let diff = StateDiff::calculate(&state1, &state2);
        assert!(!diff.has_changes());
    }
    
    #[test]
    fn test_deployment_diff() {
        let state1 = FirestreamState::new();
        let mut state2 = state1.clone();
        
        // Add a deployment to desired state
        state2.deployments.insert("test-app".to_string(), DeploymentState {
            release_name: "test-app".to_string(),
            chart: "bitnami/postgresql".to_string(),
            chart_version: "12.0.0".to_string(),
            namespace: "default".to_string(),
            status: DeploymentStatus::Pending,
            values: serde_json::json!({}),
            deployed_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            revision: 1,
            resources: vec![],
        });
        
        let diff = StateDiff::calculate(&state1, &state2);
        assert!(diff.has_changes());
        assert_eq!(diff.changes.len(), 1);
        
        let (resource_type, resource_id, change_type, _, _) = &diff.changes[0];
        assert!(matches!(resource_type, ResourceType::Deployment));
        assert_eq!(resource_id, "test-app");
        assert!(matches!(change_type, ChangeType::Create));
    }
}
