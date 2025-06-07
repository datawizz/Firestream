use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStatus {
    #[serde(rename = "buildId")]
    pub build_id: String,
    pub status: BuildState,
    pub progress: f64,
    #[serde(rename = "currentStep")]
    pub current_step: Option<String>,
    #[serde(rename = "totalSteps")]
    pub total_steps: Option<u32>,
    #[serde(rename = "startedAt")]
    pub started_at: DateTime<Utc>,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<DateTime<Utc>>,
    pub image: Option<String>,
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum BuildState {
    Pending,
    Building,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushResult {
    pub image: String,
    pub registry: String,
    pub digest: String,
    pub size: u64,
    pub pushed: bool,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStatus {
    #[serde(rename = "exportId")]
    pub export_id: String,
    pub status: ExportState,
    pub progress: f64,
    pub images: Vec<String>,
    #[serde(rename = "outputFile")]
    pub output_file: Option<String>,
    #[serde(rename = "sizeInBytes")]
    pub size_in_bytes: Option<u64>,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ExportState {
    Preparing,
    Exporting,
    Completed,
    Failed,
}
