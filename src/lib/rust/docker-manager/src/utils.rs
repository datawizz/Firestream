//! Utility functions for Docker operations

use humansize::{format_size, BINARY};
use std::collections::HashMap;
use bollard::container::Stats;

/// Format bytes into human-readable size
pub fn format_bytes(bytes: i64) -> String {
    format_size(bytes as u64, BINARY)
}

/// Parse docker labels from string format
pub fn parse_labels(labels: Vec<String>) -> HashMap<String, String> {
    labels
        .into_iter()
        .filter_map(|label| {
            let parts: Vec<&str> = label.splitn(2, '=').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Format container ports for display
pub fn format_ports(ports: &[bollard::models::Port]) -> String {
    ports
        .iter()
        .map(|port| {
            let private = port.private_port;
            let public = port.public_port;
            let protocol = port.typ.as_ref().map(|_| "tcp").unwrap_or("tcp");
            
            match public {
                Some(pub_port) => format!("{}->{}/{}", pub_port, private, protocol),
                None => format!("{}/{}", private, protocol),
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Format container state for display
pub fn format_container_state(state: &str) -> String {
    match state.to_lowercase().as_str() {
        "running" => "Running",
        "exited" => "Exited",
        "paused" => "Paused",
        "restarting" => "Restarting",
        "removing" => "Removing",
        "dead" => "Dead",
        "created" => "Created",
        _ => state,
    }
    .to_string()
}

/// Parse repository and tag from image name
pub fn parse_image_name(image: &str) -> (String, String) {
    let parts: Vec<&str> = image.rsplitn(2, ':').collect();
    if parts.len() == 2 {
        (parts[1].to_string(), parts[0].to_string())
    } else {
        (image.to_string(), "latest".to_string())
    }
}

/// Generate a container name if not provided
pub fn generate_container_name(prefix: &str) -> String {
    use chrono::Utc;
    format!("{}-{}", prefix, Utc::now().timestamp())
}

/// Check if a string is a valid Docker image name
pub fn is_valid_image_name(name: &str) -> bool {
    // Basic validation - can be enhanced
    !name.is_empty() && 
    !name.contains(' ') &&
    name.chars().all(|c| c.is_alphanumeric() || "/:.-_".contains(c))
}

/// Check if a string is a valid container name
pub fn is_valid_container_name(name: &str) -> bool {
    // Container names must match [a-zA-Z0-9][a-zA-Z0-9_.-]+
    if name.is_empty() {
        return false;
    }
    
    let first_char = name.chars().next().unwrap();
    if !first_char.is_alphanumeric() {
        return false;
    }
    
    name.chars().all(|c| c.is_alphanumeric() || "_.-".contains(c))
}

/// Format duration in seconds to human-readable format
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else if seconds < 86400 {
        format!("{}h", seconds / 3600)
    } else {
        format!("{}d", seconds / 86400)
    }
}

/// Calculate CPU usage percentage from container stats
pub fn calculate_cpu_percentage(stats: &Stats) -> f64 {
    let cpu_stats = &stats.cpu_stats;
    let precpu_stats = &stats.precpu_stats;
    
    let cpu_usage = &cpu_stats.cpu_usage;
    let precpu_usage = &precpu_stats.cpu_usage;
    
    if let (Some(system_usage), Some(pre_system_usage)) = 
        (&cpu_stats.system_cpu_usage, &precpu_stats.system_cpu_usage) {
        
        let total_usage = cpu_usage.total_usage;
        let pre_total_usage = precpu_usage.total_usage;
        
        let cpu_delta = total_usage as f64 - pre_total_usage as f64;
        let system_delta = *system_usage as f64 - *pre_system_usage as f64;
        
        if system_delta > 0.0 && cpu_delta > 0.0 {
            let cpu_count = cpu_stats.online_cpus.unwrap_or(1) as f64;
            return (cpu_delta / system_delta) * cpu_count * 100.0;
        }
    }
    
    0.0
}

/// Calculate memory usage from container stats
pub fn calculate_memory_usage(stats: &Stats) -> (u64, f64) {
    let memory_stats = &stats.memory_stats;
    let usage = memory_stats.usage.unwrap_or(0) as u64;
    let limit = memory_stats.limit.unwrap_or(0) as u64;
    
    let percentage = if limit > 0 {
        (usage as f64 / limit as f64) * 100.0
    } else {
        0.0
    };
    
    (usage, percentage)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_labels() {
        let labels = vec![
            "app=web".to_string(),
            "version=1.0".to_string(),
            "invalid_label".to_string(),
        ];
        
        let parsed = parse_labels(labels);
        assert_eq!(parsed.get("app"), Some(&"web".to_string()));
        assert_eq!(parsed.get("version"), Some(&"1.0".to_string()));
        assert_eq!(parsed.len(), 2);
    }
    
    #[test]
    fn test_parse_image_name() {
        assert_eq!(parse_image_name("nginx:latest"), ("nginx".to_string(), "latest".to_string()));
        assert_eq!(parse_image_name("nginx"), ("nginx".to_string(), "latest".to_string()));
        assert_eq!(parse_image_name("registry.com/nginx:1.19"), ("registry.com/nginx".to_string(), "1.19".to_string()));
    }
    
    #[test]
    fn test_valid_container_name() {
        assert!(is_valid_container_name("mycontainer"));
        assert!(is_valid_container_name("my-container"));
        assert!(is_valid_container_name("my_container"));
        assert!(is_valid_container_name("container123"));
        
        assert!(!is_valid_container_name(""));
        assert!(!is_valid_container_name("-container"));
        assert!(!is_valid_container_name("my container"));
        assert!(!is_valid_container_name("container@"));
    }
    
    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(30), "30s");
        assert_eq!(format_duration(90), "1m");
        assert_eq!(format_duration(3700), "1h");
        assert_eq!(format_duration(90000), "1d");
    }
}