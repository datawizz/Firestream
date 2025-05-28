//! Command execution logic
//!
//! This module implements the actual command execution for the CLI.

use super::args::{Cli, Command, ConfigCommand};
use crate::services::ServiceManager;
use crate::config::{ConfigManager, ServiceState, ServiceStatus, ResourceUsage};
use crate::core::{FirestreamError, Result};
use tracing::{info, error};

/// Execute the CLI command
pub async fn execute_command(cli: Cli) -> Result<()> {
    // Initialize services
    let service_manager = ServiceManager::new().await
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to initialize service manager: {}", e)))?;
    
    let config_manager = ConfigManager::new()
        .map_err(|e| FirestreamError::GeneralError(format!("Failed to initialize config manager: {}", e)))?;

    match cli.command {
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
        
        None => {
            // No command specified, launch TUI by default
            crate::tui::run().await?;
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
