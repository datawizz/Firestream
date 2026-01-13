use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a deployed Helm release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    /// Release name
    pub name: String,
    
    /// Namespace
    pub namespace: String,
    
    /// Chart name
    pub chart: String,
    
    /// Chart version
    pub chart_version: String,
    
    /// App version
    pub app_version: Option<String>,
    
    /// Release revision
    pub revision: u32,
    
    /// Release status
    pub status: ReleaseStatus,
    
    /// Last deployment time
    pub updated: DateTime<Utc>,
    
    /// Release notes
    pub notes: Option<String>,
    
    /// Values used for the release
    pub values: Option<serde_json::Value>,
}

/// Release status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseStatus {
    /// Successfully deployed
    Deployed,
    
    /// Uninstalling
    Uninstalling,
    
    /// Uninstall failed
    UninstallFailed,
    
    /// Pending install
    PendingInstall,
    
    /// Pending upgrade
    PendingUpgrade,
    
    /// Pending rollback
    PendingRollback,
    
    /// Failed deployment
    Failed,
    
    /// Superseded by a newer release
    Superseded,
    
    /// Unknown status
    Unknown,
}

impl ReleaseStatus {
    /// Check if the release is in a healthy state
    pub fn is_healthy(&self) -> bool {
        matches!(self, ReleaseStatus::Deployed)
    }
    
    /// Check if the release is in a pending state
    pub fn is_pending(&self) -> bool {
        matches!(
            self,
            ReleaseStatus::PendingInstall 
            | ReleaseStatus::PendingUpgrade 
            | ReleaseStatus::PendingRollback
            | ReleaseStatus::Uninstalling
        )
    }
    
    /// Check if the release is in a failed state
    pub fn is_failed(&self) -> bool {
        matches!(
            self,
            ReleaseStatus::Failed | ReleaseStatus::UninstallFailed
        )
    }
}

/// Detailed release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    /// Basic release information
    pub release: Release,
    
    /// Resources created by the release
    pub resources: Vec<Resource>,
    
    /// Release history
    pub history: Vec<ReleaseRevision>,
}

/// Kubernetes resource created by a release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource kind (e.g., "Deployment", "Service")
    pub kind: String,
    
    /// API version
    pub api_version: String,
    
    /// Resource name
    pub name: String,
    
    /// Namespace
    pub namespace: String,
}

/// Release revision history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseRevision {
    /// Revision number
    pub revision: u32,
    
    /// Deployment time
    pub updated: DateTime<Utc>,
    
    /// Status
    pub status: ReleaseStatus,
    
    /// Chart version
    pub chart_version: String,
    
    /// Description
    pub description: Option<String>,
}

impl std::fmt::Display for ReleaseStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let status = match self {
            ReleaseStatus::Deployed => "deployed",
            ReleaseStatus::Uninstalling => "uninstalling",
            ReleaseStatus::UninstallFailed => "uninstall-failed",
            ReleaseStatus::PendingInstall => "pending-install",
            ReleaseStatus::PendingUpgrade => "pending-upgrade",
            ReleaseStatus::PendingRollback => "pending-rollback",
            ReleaseStatus::Failed => "failed",
            ReleaseStatus::Superseded => "superseded",
            ReleaseStatus::Unknown => "unknown",
        };
        write!(f, "{}", status)
    }
}

impl std::str::FromStr for ReleaseStatus {
    type Err = crate::Error;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deployed" => Ok(ReleaseStatus::Deployed),
            "uninstalling" => Ok(ReleaseStatus::Uninstalling),
            "uninstall-failed" | "uninstalled-failed" => Ok(ReleaseStatus::UninstallFailed),
            "pending-install" => Ok(ReleaseStatus::PendingInstall),
            "pending-upgrade" => Ok(ReleaseStatus::PendingUpgrade),
            "pending-rollback" => Ok(ReleaseStatus::PendingRollback),
            "failed" => Ok(ReleaseStatus::Failed),
            "superseded" => Ok(ReleaseStatus::Superseded),
            _ => Ok(ReleaseStatus::Unknown),
        }
    }
}