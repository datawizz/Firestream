//! Progress reporting types for build operations

use std::sync::Arc;
use tokio::sync::Mutex;

/// Build phase enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildPhase {
    /// Discovering available containers
    Discovering,

    /// Resolving Nix flake inputs
    Resolving,

    /// Building container image
    Building {
        /// Current step in the build
        current: usize,
        /// Total number of steps
        total: usize,
    },

    /// Loading image into Docker
    Loading,

    /// Build complete
    Complete,

    /// Build failed
    Failed,
}

impl BuildPhase {
    /// Check if this phase represents completion (success or failure)
    pub fn is_terminal(&self) -> bool {
        matches!(self, BuildPhase::Complete | BuildPhase::Failed)
    }

    /// Get a human-readable description of the phase
    pub fn description(&self) -> &'static str {
        match self {
            BuildPhase::Discovering => "Discovering containers",
            BuildPhase::Resolving => "Resolving flake inputs",
            BuildPhase::Building { .. } => "Building image",
            BuildPhase::Loading => "Loading into Docker",
            BuildPhase::Complete => "Complete",
            BuildPhase::Failed => "Failed",
        }
    }
}

impl std::fmt::Display for BuildPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BuildPhase::Building { current, total } => {
                write!(f, "Building ({}/{})", current, total)
            }
            _ => write!(f, "{}", self.description()),
        }
    }
}

/// Progress update for a build operation
#[derive(Debug, Clone)]
pub struct BuildProgress {
    /// Container being built
    pub container: String,

    /// Current phase
    pub phase: BuildPhase,

    /// Human-readable message
    pub message: String,

    /// Optional percentage complete (0.0 to 100.0)
    pub percent: Option<f32>,
}

impl BuildProgress {
    /// Create a new progress update
    pub fn new(container: impl Into<String>, phase: BuildPhase, message: impl Into<String>) -> Self {
        Self {
            container: container.into(),
            phase,
            message: message.into(),
            percent: None,
        }
    }

    /// Create a progress update with percentage
    pub fn with_percent(
        container: impl Into<String>,
        phase: BuildPhase,
        message: impl Into<String>,
        percent: f32,
    ) -> Self {
        Self {
            container: container.into(),
            phase,
            message: message.into(),
            percent: Some(percent),
        }
    }

    /// Create a discovering phase progress
    pub fn discovering(container: impl Into<String>) -> Self {
        Self::new(container, BuildPhase::Discovering, "Discovering container...")
    }

    /// Create a resolving phase progress
    pub fn resolving(container: impl Into<String>) -> Self {
        Self::new(container, BuildPhase::Resolving, "Resolving flake inputs...")
    }

    /// Create a building phase progress
    pub fn building(container: impl Into<String>, current: usize, total: usize, message: impl Into<String>) -> Self {
        Self::new(
            container,
            BuildPhase::Building { current, total },
            message,
        )
    }

    /// Create a loading phase progress
    pub fn loading(container: impl Into<String>) -> Self {
        Self::new(container, BuildPhase::Loading, "Loading image into Docker...")
    }

    /// Create a complete phase progress
    pub fn complete(container: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(container, BuildPhase::Complete, message)
    }

    /// Create a failed phase progress
    pub fn failed(container: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(container, BuildPhase::Failed, message)
    }
}

impl std::fmt::Display for BuildProgress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(percent) = self.percent {
            write!(f, "[{}] {} ({:.1}%): {}", self.container, self.phase, percent, self.message)
        } else {
            write!(f, "[{}] {}: {}", self.container, self.phase, self.message)
        }
    }
}

/// Type alias for a progress callback function
pub type ProgressCallback = Box<dyn FnMut(BuildProgress) + Send>;

/// Thread-safe progress callback wrapper
pub type SharedProgressCallback<F> = Arc<Mutex<F>>;

/// No-op progress callback for when progress reporting is not needed
pub fn no_op_progress() -> impl FnMut(BuildProgress) + Send {
    |_| {}
}

/// Create a shared progress callback
pub fn shared_progress<F>(callback: F) -> SharedProgressCallback<F>
where
    F: FnMut(BuildProgress) + Send,
{
    Arc::new(Mutex::new(callback))
}

/// Multi-container build progress tracking
#[derive(Debug, Clone)]
pub struct MultiProgress {
    /// Total number of containers to build
    pub total: usize,

    /// Number of containers completed (success or failure)
    pub completed: usize,

    /// Number of containers successfully built
    pub succeeded: usize,

    /// Number of containers that failed
    pub failed: usize,

    /// Currently building containers
    pub in_progress: Vec<String>,
}

impl MultiProgress {
    /// Create a new multi-progress tracker
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            succeeded: 0,
            failed: 0,
            in_progress: Vec::new(),
        }
    }

    /// Mark a container as started
    pub fn start(&mut self, container: impl Into<String>) {
        self.in_progress.push(container.into());
    }

    /// Mark a container as completed successfully
    pub fn complete_success(&mut self, container: &str) {
        self.in_progress.retain(|c| c != container);
        self.completed += 1;
        self.succeeded += 1;
    }

    /// Mark a container as failed
    pub fn complete_failed(&mut self, container: &str) {
        self.in_progress.retain(|c| c != container);
        self.completed += 1;
        self.failed += 1;
    }

    /// Get overall progress percentage
    pub fn percent(&self) -> f32 {
        if self.total == 0 {
            100.0
        } else {
            (self.completed as f32 / self.total as f32) * 100.0
        }
    }

    /// Check if all containers are complete
    pub fn is_complete(&self) -> bool {
        self.completed >= self.total
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_phase_display() {
        assert_eq!(BuildPhase::Discovering.to_string(), "Discovering containers");
        assert_eq!(
            BuildPhase::Building { current: 1, total: 3 }.to_string(),
            "Building (1/3)"
        );
    }

    #[test]
    fn test_build_progress_display() {
        let progress = BuildProgress::new("redis", BuildPhase::Building { current: 1, total: 3 }, "Compiling...");
        assert!(progress.to_string().contains("redis"));
        assert!(progress.to_string().contains("1/3"));
    }

    #[test]
    fn test_multi_progress() {
        let mut mp = MultiProgress::new(3);
        assert_eq!(mp.percent(), 0.0);

        mp.start("redis");
        mp.complete_success("redis");
        // Use approximate comparison for floating point
        assert!((mp.percent() - 100.0 / 3.0).abs() < 0.001);

        mp.start("postgres");
        mp.complete_failed("postgres");
        assert_eq!(mp.succeeded, 1);
        assert_eq!(mp.failed, 1);
    }
}
