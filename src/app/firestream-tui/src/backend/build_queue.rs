//! Build queue with dependency ordering and cancellation support.
//!
//! Manages sequential Nix builds with a FIFO queue, dependency-aware ordering,
//! and cancellation via tokio_util::sync::CancellationToken.

use nix_container_builder::BuildPhase;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;
use tokio_util::sync::CancellationToken;

/// A request to build a container.
#[derive(Debug, Clone)]
pub struct BuildRequest {
    pub container: String,
    pub package_name: String,
    pub queued_at: Instant,
}

/// State of an actively running build.
#[derive(Debug, Clone)]
pub struct ActiveBuild {
    pub container: String,
    pub phase: BuildPhase,
    pub log_lines: Vec<String>,
    pub started_at: Instant,
}

/// Result of a completed build.
#[derive(Debug, Clone)]
pub struct CompletedBuild {
    pub container: String,
    pub success: bool,
    pub message: String,
    pub duration_secs: u64,
}

/// Build queue managing sequential builds with dependency ordering.
pub struct BuildQueue {
    queue: VecDeque<BuildRequest>,
    active: Option<ActiveBuild>,
    completed: Vec<CompletedBuild>,
    cancel_tokens: HashMap<String, CancellationToken>,
}

impl BuildQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            active: None,
            completed: Vec::new(),
            cancel_tokens: HashMap::new(),
        }
    }

    /// Enqueue a build request. Returns the queue position (0-indexed).
    pub fn enqueue(&mut self, container: String, package_name: String) -> usize {
        // Don't enqueue duplicates
        if self.queue.iter().any(|r| r.container == container) {
            if let Some(pos) = self.queue.iter().position(|r| r.container == container) {
                return pos;
            }
        }
        if self.active.as_ref().map_or(false, |a| a.container == container) {
            return 0; // Already building
        }

        let request = BuildRequest {
            container,
            package_name,
            queued_at: Instant::now(),
        };
        self.queue.push_back(request);
        self.queue.len() - 1
    }

    /// Dequeue the next build request (FIFO).
    pub fn dequeue(&mut self) -> Option<BuildRequest> {
        self.queue.pop_front()
    }

    /// Set the currently active build.
    pub fn set_active(&mut self, container: String, cancel_token: CancellationToken) {
        self.cancel_tokens.insert(container.clone(), cancel_token);
        self.active = Some(ActiveBuild {
            container,
            phase: BuildPhase::Resolving,
            log_lines: Vec::new(),
            started_at: Instant::now(),
        });
    }

    /// Update the active build's phase.
    pub fn update_phase(&mut self, phase: BuildPhase) {
        if let Some(ref mut active) = self.active {
            active.phase = phase;
        }
    }

    /// Append a log line to the active build.
    pub fn append_log(&mut self, line: String) {
        if let Some(ref mut active) = self.active {
            // Cap at 1000 lines to prevent unbounded growth
            if active.log_lines.len() < 1000 {
                active.log_lines.push(line);
            }
        }
    }

    /// Mark the active build as complete.
    pub fn complete_active(&mut self, success: bool, message: String) {
        if let Some(active) = self.active.take() {
            let duration = active.started_at.elapsed().as_secs();
            self.cancel_tokens.remove(&active.container);
            self.completed.push(CompletedBuild {
                container: active.container,
                success,
                message,
                duration_secs: duration,
            });
        }
    }

    /// Cancel a build (active or queued).
    pub fn cancel(&mut self, container: &str) -> bool {
        // Cancel active build
        if let Some(token) = self.cancel_tokens.get(container) {
            token.cancel();
            return true;
        }
        // Remove from queue
        if let Some(pos) = self.queue.iter().position(|r| r.container == container) {
            self.queue.remove(pos);
            return true;
        }
        false
    }

    /// Get the currently active build (if any).
    pub fn active(&self) -> Option<&ActiveBuild> {
        self.active.as_ref()
    }

    /// Get all queued requests.
    pub fn queued(&self) -> &VecDeque<BuildRequest> {
        &self.queue
    }

    /// Get completed builds.
    pub fn completed(&self) -> &[CompletedBuild] {
        &self.completed
    }

    /// Check if any build is active or queued.
    pub fn is_busy(&self) -> bool {
        self.active.is_some() || !self.queue.is_empty()
    }

    /// Get the total number of items (active + queued).
    pub fn len(&self) -> usize {
        (if self.active.is_some() { 1 } else { 0 }) + self.queue.len()
    }
}

/// Snapshot of the build queue state for the UI.
#[derive(Debug, Clone)]
pub struct BuildQueueSnapshot {
    pub active: Option<ActiveBuild>,
    pub queued: Vec<BuildRequest>,
    pub recent_completed: Vec<CompletedBuild>,
}

impl BuildQueue {
    /// Take a snapshot for UI rendering.
    pub fn snapshot(&self) -> BuildQueueSnapshot {
        BuildQueueSnapshot {
            active: self.active.clone(),
            queued: self.queue.iter().cloned().collect(),
            recent_completed: self.completed.iter().rev().take(5).cloned().collect(),
        }
    }
}
