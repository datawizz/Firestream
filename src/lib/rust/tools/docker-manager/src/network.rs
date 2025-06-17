//! Network management operations

use crate::{DockerManager, DockerManagerError, Result};
use bollard::network::{
    CreateNetworkOptions, ListNetworksOptions, ConnectNetworkOptions,
    DisconnectNetworkOptions,
};
use bollard::models::{Network, EndpointSettings};
use std::collections::HashMap;
use tracing::{debug, info};

/// Network operations
impl DockerManager {
    /// List all networks
    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        let options = ListNetworksOptions::<String> {
            ..Default::default()
        };
        
        let networks = self.client.list_networks(Some(options)).await?;
        debug!("Found {} networks", networks.len());
        Ok(networks)
    }
    
    /// Get network by name or ID
    pub async fn get_network(&self, name_or_id: &str) -> Result<Network> {
        let networks = self.list_networks().await?;
        
        networks
            .into_iter()
            .find(|n| {
                n.id.as_ref().map(|id| id.starts_with(name_or_id)).unwrap_or(false) ||
                n.name.as_ref().map(|name| name == name_or_id).unwrap_or(false)
            })
            .ok_or_else(|| DockerManagerError::NetworkNotFound(name_or_id.to_string()))
    }
    
    /// Create a new network
    pub async fn create_network(
        &self,
        name: &str,
        driver: Option<&str>,
        internal: bool,
        attachable: bool,
        labels: Option<HashMap<String, String>>,
    ) -> Result<String> {
        let options = CreateNetworkOptions {
            name: name.to_string(),
            driver: driver.unwrap_or("bridge").to_string(),
            internal,
            attachable,
            labels: labels.unwrap_or_default(),
            ..Default::default()
        };
        
        let response = self.client.create_network(options).await?;
        let network_id = response.id.unwrap_or_else(|| name.to_string());
        info!("Created network '{}' with ID: {}", name, network_id);
        Ok(network_id)
    }
    
    /// Remove a network
    pub async fn remove_network(&self, name_or_id: &str) -> Result<()> {
        self.client.remove_network(name_or_id).await?;
        info!("Removed network '{}'", name_or_id);
        Ok(())
    }
    
    /// Connect a container to a network
    pub async fn connect_container_to_network(
        &self,
        container_name_or_id: &str,
        network_name_or_id: &str,
        aliases: Option<Vec<String>>,
    ) -> Result<()> {
        let endpoint_config = EndpointSettings {
            aliases: aliases,
            ..Default::default()
        };
        
        let options = ConnectNetworkOptions {
            container: container_name_or_id,
            endpoint_config,
        };
        
        self.client.connect_network(network_name_or_id, options).await?;
        info!("Connected container '{}' to network '{}'", container_name_or_id, network_name_or_id);
        Ok(())
    }
    
    /// Disconnect a container from a network
    pub async fn disconnect_container_from_network(
        &self,
        container_name_or_id: &str,
        network_name_or_id: &str,
        force: bool,
    ) -> Result<()> {
        let options = DisconnectNetworkOptions {
            container: container_name_or_id,
            force,
        };
        
        self.client.disconnect_network(network_name_or_id, options).await?;
        info!("Disconnected container '{}' from network '{}'", container_name_or_id, network_name_or_id);
        Ok(())
    }
    
    /// Inspect a network
    pub async fn inspect_network(&self, name_or_id: &str) -> Result<Network> {
        let network = self.client.inspect_network::<String>(name_or_id, None).await?;
        Ok(network)
    }
    
    /// Prune unused networks
    pub async fn prune_networks(&self) -> Result<Vec<String>> {
        let result = self.client.prune_networks::<String>(None).await?;
        let networks_deleted = result.networks_deleted.unwrap_or_default();
        
        info!("Pruned {} networks", networks_deleted.len());
        Ok(networks_deleted)
    }
    
    /// Check if network exists
    pub async fn network_exists(&self, name: &str) -> bool {
        self.get_network(name).await.is_ok()
    }
    
    /// Get containers connected to a network
    pub async fn get_network_containers(&self, network_name_or_id: &str) -> Result<Vec<String>> {
        let network = self.inspect_network(network_name_or_id).await?;
        
        let containers = network
            .containers
            .unwrap_or_default()
            .into_keys()
            .collect();
        
        Ok(containers)
    }
    
    /// List networks used by a container
    pub async fn get_container_networks(&self, container_name_or_id: &str) -> Result<Vec<String>> {
        let inspect = self.client.inspect_container(container_name_or_id, None).await?;
        
        let networks = inspect
            .network_settings
            .and_then(|ns| ns.networks)
            .unwrap_or_default()
            .into_keys()
            .collect();
        
        Ok(networks)
    }
}