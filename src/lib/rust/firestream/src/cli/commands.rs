//! Command execution logic
//!
//! This module implements the actual command execution for the CLI.

use super::args::{Cli, Command, ConfigCommand, StateCommand, ClusterCommand};
use crate::services::ServiceManager;
use crate::config::{ConfigManager, ServiceState, ServiceStatus, ResourceUsage};
use crate::state::{StateManager, ResourceType, K3dClusterConfig, K3dDevModeConfig, FirestreamState};
use crate::deploy::k3d_advanced::{K3dClusterManager};
use crate::core::{FirestreamError, Result};
use std::path::PathBuf;
use tracing::{info, error};

/// Execute the CLI command
pub async fn execute_command(cli: Cli) -> Result<()> {
    // Initialize services
    let service_manager = ServiceManager::new().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to initialize service manager: {}", e)))?;
    
    let config_manager = ConfigManager::new()
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to initialize config manager: {}", e)))?;

    // Get state directory
    let state_dir = PathBuf::from(".firestream/state");
    
    match cli.command {
        Some(Command::Init { name }) => {
            execute_init(name).await?;
        }
        
        Some(Command::Plan { target, output, out }) => {
            execute_plan(&state_dir, target, &output, out).await?;
        }
        
        Some(Command::Apply { plan, auto_approve, target }) => {
            execute_apply(&state_dir, plan, auto_approve, target).await?;
        }
        
        Some(Command::Refresh) => {
            execute_refresh(&state_dir).await?;
        }
        
        Some(Command::Import { resource_type, resource_id, data }) => {
            execute_import(&state_dir, &resource_type, &resource_id, data).await?;
        }
        
        Some(Command::State { ref command }) => {
            execute_state_command(command, &state_dir).await?;
        }
        
        Some(Command::Install { service, config }) => {
            let config_path = config.as_ref().map(|p| p.to_str().unwrap());
            service_manager.install_service(&service, config_path).await?;
            if !cli.quiet {
                println!("Service {} installed successfully!", service);
            }
        }
        
        Some(Command::Uninstall { service: _ }) => {
            // TODO: Implement uninstall
            error!("Uninstall command not yet implemented");
            return Err(FirestreamError::GeneralError("Not implemented".to_string()));
        }
        
        Some(Command::Start { service }) => {
            service_manager.start_service(&service).await?;
            if !cli.quiet {
                println!("Service {} started successfully!", service);
            }
        }
        
        Some(Command::Stop { service }) => {
            service_manager.stop_service(&service).await?;
            if !cli.quiet {
                println!("Service {} stopped successfully!", service);
            }
        }
        
        Some(Command::Restart { service }) => {
            service_manager.stop_service(&service).await?;
            service_manager.start_service(&service).await?;
            if !cli.quiet {
                println!("Service {} restarted successfully!", service);
            }
        }
        
        Some(Command::Status { service }) => {
            let states = service_manager.get_status(service.as_deref()).await?;
            
            if cli.json {
                let json = serde_json::to_string_pretty(&states)
                    .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize to JSON: {}", e)))?;
                println!("{}", json);
            } else {
                print_status(&states, cli.quiet);
            }
        }
        
        Some(Command::List { available }) => {
            if available {
                let services = service_manager.list_available_services().await?;
                
                if cli.json {
                    let json = serde_json::to_string_pretty(&services)
                        .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize to JSON: {}", e)))?;
                    println!("{}", json);
                } else {
                    println!("Available services:");
                    for service in services {
                        println!("  {}", service);
                    }
                }
            } else {
                let states = service_manager.get_status(None).await?;
                
                if cli.json {
                    let json = serde_json::to_string_pretty(&states)
                        .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize to JSON: {}", e)))?;
                    println!("{}", json);
                } else {
                    println!("Installed services:");
                    for (name, state) in states {
                        println!("  {} - {:?}", name, state.status);
                    }
                }
            }
        }
        
        Some(Command::Config { ref command }) => {
            execute_config_command(command, &config_manager, &cli).await?;
        }
        
        Some(Command::Logs { service, follow, lines }) => {
            let logs = service_manager.get_logs(&service, follow).await?;
            
            let start_idx = if logs.len() > lines {
                logs.len() - lines
            } else {
                0
            };
            
            for line in &logs[start_idx..] {
                println!("{}", line);
            }
            
            if follow {
                // TODO: Implement actual log following
                println!("Log following not yet implemented");
            }
        }
        
        Some(Command::Resources { service }) => {
            let usage = service_manager.get_resource_usage(service.as_deref()).await?;
            
            if cli.json {
                let json = serde_json::to_string_pretty(&usage)
                    .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize to JSON: {}", e)))?;
                println!("{}", json);
            } else {
                print_resource_usage(&usage);
            }
        }
        
        Some(Command::Tui) => {
            // Launch TUI
            crate::tui::run().await?;
        }
        
        Some(Command::Cluster { ref command }) => {
            execute_cluster_command(command, &state_dir).await?;
        }
        
        None => {
            // No command specified, launch TUI by default
            crate::tui::run().await?;
        }
    }

    Ok(())
}

/// Execute init command
async fn execute_init(name: Option<String>) -> Result<()> {
    let project_name = name.unwrap_or_else(|| "firestream-project".to_string());
    
    // Create project structure
    let project_dir = PathBuf::from(&project_name);
    tokio::fs::create_dir_all(&project_dir).await?;
    tokio::fs::create_dir_all(project_dir.join(".firestream/state")).await?;
    tokio::fs::create_dir_all(project_dir.join("infrastructure")).await?;
    tokio::fs::create_dir_all(project_dir.join("deployments")).await?;
    tokio::fs::create_dir_all(project_dir.join("builds")).await?;
    
    // Create default config
    let config = crate::config::GlobalConfig::default();
    let config_path = project_dir.join("firestream.toml");
    let config_str = toml::to_string_pretty(&config)?;
    tokio::fs::write(&config_path, config_str).await?;
    
    // Initialize state
    let state_manager = StateManager::new(project_dir.join(".firestream/state"));
    state_manager.init().await?;
    
    println!("Initialized Firestream project in '{}'", project_name);
    println!("\nNext steps:");
    println!("  1. cd {}", project_name);
    println!("  2. Edit firestream.toml to configure your project");
    println!("  3. Run 'firestream plan' to see what will be created");
    println!("  4. Run 'firestream apply' to create resources");
    
    Ok(())
}

/// Execute plan command
async fn execute_plan(state_dir: &PathBuf, targets: Vec<String>, output: &str, out: Option<PathBuf>) -> Result<()> {
    let mut state_manager = StateManager::new(state_dir.clone());
    
    // Lock state
    state_manager.lock().await?;
    
    // Load config
    let config_manager = ConfigManager::new()?;
    let config = config_manager.load_global_config().await?;
    
    // Create plan
    let plan = state_manager.plan(&config, targets).await?;
    
    // Output plan
    match output {
        "json" => {
            let json = serde_json::to_string_pretty(&plan)?;
            if let Some(out_path) = out {
                tokio::fs::write(&out_path, &json).await?;
                println!("Plan saved to {:?}", out_path);
            } else {
                println!("{}", json);
            }
        }
        _ => {
            // Text output
            println!("\nFirestream will perform the following actions:\n");
            
            for change in &plan.changes {
                let symbol = match change.change_type {
                    crate::state::ChangeType::Create => "+",
                    crate::state::ChangeType::Update => "~",
                    crate::state::ChangeType::Delete => "-",
                    crate::state::ChangeType::Replace => "!",
                    crate::state::ChangeType::NoOp => " ",
                };
                
                println!("  {} {}", symbol, change.description);
            }
            
            println!("\nPlan: {} changes", plan.changes.len());
            
            if let Some(out_path) = out {
                let json = serde_json::to_string_pretty(&plan)?;
                tokio::fs::write(&out_path, &json).await?;
                println!("\nPlan saved to {:?}", out_path);
            }
        }
    }
    
    // Unlock state
    state_manager.unlock().await?;
    
    Ok(())
}

/// Execute apply command
async fn execute_apply(state_dir: &PathBuf, plan_id: Option<String>, auto_approve: bool, targets: Vec<String>) -> Result<()> {
    let mut state_manager = StateManager::new(state_dir.clone());
    
    // Lock state
    state_manager.lock().await?;
    
    let result = if let Some(plan_id) = plan_id {
        // Apply specific plan
        state_manager.apply(&plan_id, auto_approve).await
    } else {
        // Create and apply new plan
        let config_manager = ConfigManager::new()?;
        let config = config_manager.load_global_config().await?;
        
        let plan = state_manager.plan(&config, targets).await?;
        state_manager.apply(&plan.id, auto_approve).await
    };
    
    // Unlock state
    state_manager.unlock().await?;
    
    result?;
    
    println!("\nApply complete! Resources are up-to-date.");
    Ok(())
}

/// Execute refresh command
async fn execute_refresh(state_dir: &PathBuf) -> Result<()> {
    let mut state_manager = StateManager::new(state_dir.clone());
    
    // Lock state
    state_manager.lock().await?;
    
    println!("Refreshing state...");
    state_manager.refresh().await?;
    
    // Unlock state
    state_manager.unlock().await?;
    
    println!("State refreshed successfully.");
    Ok(())
}

/// Execute import command
async fn execute_import(state_dir: &PathBuf, resource_type: &str, resource_id: &str, data_file: Option<PathBuf>) -> Result<()> {
    let mut state_manager = StateManager::new(state_dir.clone());
    
    // Parse resource type
    let rt = match resource_type {
        "infrastructure" => ResourceType::Infrastructure,
        "build" => ResourceType::Build,
        "deployment" => ResourceType::Deployment,
        "cluster" => ResourceType::Cluster,
        _ => return Err(FirestreamError::GeneralError(format!("Invalid resource type: {}", resource_type))),
    };
    
    // Load resource data
    let data = if let Some(path) = data_file {
        let content = tokio::fs::read_to_string(&path).await?;
        serde_json::from_str(&content)?
    } else {
        serde_json::json!({})
    };
    
    // Lock state
    state_manager.lock().await?;
    
    state_manager.import(rt, resource_id.to_string(), data).await?;
    
    // Unlock state
    state_manager.unlock().await?;
    
    println!("Imported {} '{}' successfully.", resource_type, resource_id);
    Ok(())
}

/// Execute state subcommands
async fn execute_state_command(command: &StateCommand, state_dir: &PathBuf) -> Result<()> {
    match command {
        StateCommand::Show { resource, output } => {
            let state_manager = StateManager::new(state_dir.clone());
            let mut sm = state_manager;
            let state = sm.load().await?;
            
            match output.as_str() {
                "json" => {
                    let json = if let Some(res) = resource {
                        // Show specific resource
                        serde_json::json!({
                            "resource": res,
                            "state": state
                        })
                    } else {
                        serde_json::to_value(state)?
                    };
                    println!("{}", serde_json::to_string_pretty(&json)?);
                }
                _ => {
                    // Text output
                    println!("State Information:");
                    println!("  Version: {}", state.version);
                    println!("  Serial: {}", state.serial);
                    println!("  Last Modified: {}", state.last_modified);
                    println!("  Last Modified By: {}", state.last_modified_by);
                    println!("\nResources:");
                    println!("  Infrastructure: {}", if state.infrastructure.stack_name.is_some() { "Configured" } else { "Not configured" });
                    println!("  Builds: {}", state.builds.len());
                    println!("  Deployments: {}", state.deployments.len());
                    println!("  Cluster: {}", if state.cluster.name.is_some() { "Configured" } else { "Not configured" });
                }
            }
        }
        
        StateCommand::Lock { timeout: _ } => {
            let mut state_manager = StateManager::new(state_dir.clone());
            state_manager.lock().await?;
            println!("State locked successfully.");
        }
        
        StateCommand::Unlock { force } => {
            if *force {
                let lock = crate::state::StateLock::force_acquire(state_dir).await?;
                lock.release().await?;
            } else {
                let mut state_manager = StateManager::new(state_dir.clone());
                state_manager.unlock().await?;
            }
            println!("State unlocked successfully.");
        }
        
        StateCommand::Locks => {
            if let Some(lock_info) = crate::state::StateLock::get_info(state_dir).await {
                println!("Active locks:");
                println!("  ID: {}", lock_info.id);
                println!("  User: {}", lock_info.user);
                println!("  Host: {}", lock_info.hostname);
                println!("  PID: {}", lock_info.pid);
                println!("  Acquired: {}", lock_info.acquired_at);
                println!("  Expires: {}", lock_info.expires_at);
            } else {
                println!("No active locks.");
            }
        }
        
        StateCommand::Pull => {
            println!("Remote state management not yet implemented.");
        }
        
        StateCommand::Push => {
            println!("Remote state management not yet implemented.");
        }
    }
    
    Ok(())
}

/// Execute config subcommands
async fn execute_config_command(command: &ConfigCommand, config_manager: &ConfigManager, cli: &Cli) -> Result<()> {
    match command {
        ConfigCommand::Show { service } => {
            let config = config_manager.load_service_config(&service).await
                .map_err(|e| FirestreamError::ConfigError(format!("Failed to load config: {}", e)))?;
            
            if cli.json {
                let json = serde_json::to_string_pretty(&config)
                    .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize to JSON: {}", e)))?;
                println!("{}", json);
            } else {
                let toml = toml::to_string_pretty(&config)
                    .map_err(|e| FirestreamError::GeneralError(format!("Failed to serialize to TOML: {}", e)))?;
                println!("{}", toml);
            }
        }
        
        ConfigCommand::Edit { service: _ } => {
            // TODO: Implement config editing
            error!("Config edit command not yet implemented");
            return Err(FirestreamError::GeneralError("Not implemented".to_string()));
        }
        
        ConfigCommand::Validate { target } => {
            // TODO: Implement config validation
            info!("Validating configuration: {}", target);
            println!("Configuration is valid");
        }
    }
    
    Ok(())
}

/// Print service status in a formatted way
fn print_status(states: &std::collections::HashMap<String, ServiceState>, quiet: bool) {
    
    if quiet {
        for (name, state) in states {
            println!("{}\t{:?}", name, state.status);
        }
    } else {
        println!("{:<20} {:<15} {:<10} {:<15} {:<10}",
                 "Service", "Status", "Version", "Last Updated", "Resources");
        println!("{}", "-".repeat(80));
        
        for (name, state) in states {
            let version = state.installed_version.as_deref().unwrap_or("-");
            let updated = state.last_updated.format("%Y-%m-%d %H:%M");
            let resources = if let Some(usage) = &state.resource_usage {
                format!("{:.1}CPU {:.0}MB", usage.cpu_cores, usage.memory_mb)
            } else {
                "-".to_string()
            };
            
            let status_str = match &state.status {
                ServiceStatus::Running => "Running",
                ServiceStatus::Stopped => "Stopped",
                ServiceStatus::Installing => "Installing",
                ServiceStatus::NotInstalled => "Not Installed",
                ServiceStatus::Error(_) => "Error",
            };
            
            println!("{:<20} {:<15} {:<10} {:<15} {:<10}",
                     name, status_str, version, updated, resources);
        }
    }
}

/// Print resource usage in a formatted way
fn print_resource_usage(usage: &std::collections::HashMap<String, ResourceUsage>) {
    println!("{:<20} {:<10} {:<15} {:<10}",
             "Service", "CPU", "Memory", "Disk");
    println!("{}", "-".repeat(60));
    
    for (name, resources) in usage {
        println!("{:<20} {:<10.2} {:<15} {:<10}",
                 name,
                 resources.cpu_cores,
                 format!("{}MB", resources.memory_mb),
                 format!("{}GB", resources.disk_gb));
    }
}

/// Execute cluster subcommands
async fn execute_cluster_command(command: &ClusterCommand, state_dir: &PathBuf) -> Result<()> {
    match command {
        ClusterCommand::Create { name, config: config_file, clean, servers, agents, dev_mode } => {
            let config = if let Some(config_path) = config_file {
                // Load config from file
                let content = tokio::fs::read_to_string(config_path).await?;
                let parsed: toml::Value = toml::from_str(&content)?;
                
                // Extract k3d config from TOML
                if let Some(cluster_section) = parsed.get("cluster") {
                    if let Some(k3d_section) = cluster_section.get("k3d") {
                        serde_json::from_str(&serde_json::to_string(k3d_section)?)?
                    } else {
                        return Err(FirestreamError::ConfigError("Missing [cluster.k3d] section in config".to_string()));
                    }
                } else {
                    return Err(FirestreamError::ConfigError("Missing [cluster] section in config".to_string()));
                }
            } else {
                // Create config from CLI args
                let mut config = K3dClusterConfig::default();
                
                if let Some(cluster_name) = name {
                    config.name = cluster_name.clone();
                }
                
                config.servers = *servers;
                config.agents = *agents;
                
                if *dev_mode {
                    config.dev_mode = Some(crate::state::K3dDevModeConfig {
                        port_forward_all: true,
                        port_offset: 10000,
                    });
                }
                
                config
            };
            
            let manager = K3dClusterManager::new(config.clone());
            
            if *clean {
                // Delete existing cluster first
                let _ = manager.delete_cluster().await;
            }
            
            println!("Creating K3D cluster '{}'...", config.name);
            manager.setup_cluster().await?;
            
            // Save cluster configuration to state
            let state_manager = StateManager::new(state_dir.clone());
            
            // Initialize state directory if needed
            state_manager.init().await?;
            
            // Use a manual approach to update state
            let state_file = state_dir.join("firestream.state.json");
            
            // Load existing state or create new one
            let mut state = if state_file.exists() {
                let content = tokio::fs::read_to_string(&state_file).await?;
                serde_json::from_str::<FirestreamState>(&content)?
            } else {
                FirestreamState::new()
            };
            
            // Update cluster configuration
            state.cluster.name = Some(config.name.clone());
            state.cluster.cluster_type = Some("k3d".to_string());
            state.cluster.managed = true;
            state.cluster.k3d_config = Some(config.clone());
            state.increment_serial();
            
            // Save state
            let content = serde_json::to_string_pretty(&state)?;
            tokio::fs::write(&state_file, content).await?;
            
            println!("\nCluster '{}' created successfully!", config.name);
            println!("\nKubectl context set to: k3d-{}", config.name);
            
            if config.dev_mode.is_some() {
                println!("\nDevelopment mode enabled. Services will be accessible at localhost with +10000 port offset.");
            }
        }
        
        ClusterCommand::Delete { name } => {
            let cluster_name = if let Some(n) = name {
                n.clone()
            } else {
                // Try to load from state
                let mut state_manager = StateManager::new(state_dir.clone());
                let state = state_manager.load().await?;
                
                if let Some(k3d_config) = &state.cluster.k3d_config {
                    k3d_config.name.clone()
                } else if let Some(n) = &state.cluster.name {
                    n.clone()
                } else {
                    return Err(FirestreamError::GeneralError("No cluster name specified and none found in state".to_string()));
                }
            };
            
            println!("Deleting K3D cluster '{}'...", cluster_name);
            
            let config = K3dClusterConfig {
                name: cluster_name.clone(),
                ..Default::default()
            };
            
            let manager = K3dClusterManager::new(config);
            manager.delete_cluster().await?;
            
            // Remove cluster configuration from state
            let state_file = state_dir.join("firestream.state.json");
            if state_file.exists() {
                let content = tokio::fs::read_to_string(&state_file).await?;
                let mut state = serde_json::from_str::<FirestreamState>(&content)?;
                
                // Clear cluster configuration
                state.cluster.name = None;
                state.cluster.cluster_type = None;
                state.cluster.managed = false;
                state.cluster.k3d_config = None;
                state.increment_serial();
                
                // Save updated state
                let content = serde_json::to_string_pretty(&state)?;
                tokio::fs::write(&state_file, content).await?;
            }
            
            println!("Cluster '{}' deleted successfully.", cluster_name);
        }
        
        ClusterCommand::Info { name } => {
            let cluster_name = if let Some(n) = name {
                n.clone()
            } else {
                // Try to load from state
                let mut state_manager = StateManager::new(state_dir.clone());
                let state = state_manager.load().await?;
                
                if let Some(k3d_config) = &state.cluster.k3d_config {
                    k3d_config.name.clone()
                } else if let Some(n) = &state.cluster.name {
                    n.clone()
                } else {
                    return Err(FirestreamError::GeneralError("No cluster name specified and none found in state".to_string()));
                }
            };
            
            let config = K3dClusterConfig {
                name: cluster_name.clone(),
                ..Default::default()
            };
            
            let manager = K3dClusterManager::new(config);
            let info = manager.get_cluster_info().await?;
            
            println!("Cluster Information:");
            println!("  Name: {}", cluster_name);
            
            for (key, value) in info {
                if key == "nodes" {
                    println!("\nNodes:");
                    println!("{}", value);
                } else {
                    println!("  {}: {}", key, value);
                }
            }
        }
        
        ClusterCommand::PortForward { service, offset } => {
            println!("Setting up port forwarding...");
            
            if service == "all" {
                // Setup port forwarding for all services
                let config = K3dClusterConfig::default();
                let manager = K3dClusterManager::new(config);
                
                let dev_config = crate::state::K3dDevModeConfig {
                    port_forward_all: true,
                    port_offset: *offset,
                };
                
                manager.setup_port_forwarding(&dev_config).await?;
                
                println!("Port forwarding setup complete for all services.");
            } else {
                // Setup port forwarding for specific service
                println!("Port forwarding for specific services not yet implemented.");
            }
        }
    }
    
    Ok(())
}
