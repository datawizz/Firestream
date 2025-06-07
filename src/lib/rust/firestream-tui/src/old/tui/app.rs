//! TUI application state and logic
//!
//! This module manages the TUI application state and handles user interactions.

use crate::services::ServiceManager;
use crate::config::{ServiceState, ServiceStatus, ConfigManager};
use crate::state::{StateManager, FirestreamState, ExecutionPlan, PlanStatus};
use crate::core::{FirestreamError, Result};
use std::path::PathBuf;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    collections::HashMap,
    io,
    time::{Duration, Instant},
};


/// TUI application state
pub struct App {
    /// Service manager instance
    service_manager: ServiceManager,
    
    /// State manager instance
    state_manager: StateManager,
    
    /// Config manager instance
    config_manager: ConfigManager,
    
    /// Current tab
    pub current_tab: Tab,
    
    /// Selected service index
    pub selected_service: usize,
    
    /// Service states
    pub service_states: HashMap<String, ServiceState>,
    
    /// Available services
    pub available_services: Vec<String>,
    
    /// Current firestream state
    pub firestream_state: Option<FirestreamState>,
    
    /// Current execution plan
    pub current_plan: Option<ExecutionPlan>,
    
    /// Selected item in state view
    pub selected_state_item: usize,
    
    /// State operation in progress
    pub state_operation: Option<String>,
    
    /// Should quit
    should_quit: bool,
    
    /// Last update time
    last_update: Instant,
}

/// Available tabs in the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Services,
    State,
    Plan,
    Deploy,
    Logs,
    Config,
    Help,
}

impl App {
    /// Create a new TUI application
    pub async fn new() -> anyhow::Result<Self> {
        let service_manager = ServiceManager::new().await?;
        let state_dir = PathBuf::from(".firestream/state");
        
        // Initialize state manager and directories
        let state_manager = StateManager::new(state_dir);
        state_manager.init().await
            .map_err(|e| anyhow::anyhow!("Failed to initialize state manager: {}", e))?;
        
        let config_manager = ConfigManager::new()?;
        
        let service_states = service_manager.get_status(None).await
            .unwrap_or_default();
        let available_services = service_manager.list_available_services().await
            .unwrap_or_default();

        Ok(Self {
            service_manager,
            state_manager,
            config_manager,
            current_tab: Tab::Services,
            selected_service: 0,
            service_states,
            available_services,
            firestream_state: None,
            current_plan: None,
            selected_state_item: 0,
            state_operation: None,
            should_quit: false,
            last_update: Instant::now(),
        })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode().map_err(|e| FirestreamError::IoError(e.to_string()))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| FirestreamError::IoError(e.to_string()))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| FirestreamError::IoError(e.to_string()))?;

        // Run the app
        let res = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode().map_err(|e| FirestreamError::IoError(e.to_string()))?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).map_err(|e| FirestreamError::IoError(e.to_string()))?;
        terminal.show_cursor().map_err(|e| FirestreamError::IoError(e.to_string()))?;

        res
    }

    /// Main application loop
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut last_update = Instant::now();
        let update_duration = Duration::from_secs(5);

        loop {
            // Draw UI
            terminal.draw(|f| crate::tui::ui::draw(f, self))
                .map_err(|e| FirestreamError::IoError(e.to_string()))?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(100))
                .map_err(|e| FirestreamError::IoError(e.to_string()))? {
                if let Event::Key(key) = event::read()
                    .map_err(|e| FirestreamError::IoError(e.to_string()))? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        KeyCode::Tab => self.next_tab(),
                        KeyCode::BackTab => self.previous_tab(),
                        
                        // Tab-specific controls
                        _ => match self.current_tab {
                            Tab::Services => self.handle_services_key(key.code).await?,
                            Tab::State => self.handle_state_key(key.code).await?,
                            Tab::Plan => self.handle_plan_key(key.code).await?,
                            Tab::Deploy => self.handle_deploy_key(key.code).await?,
                            _ => {}
                        }
                    }
                }
            }

            if self.should_quit {
                return Ok(());
            }

            // Update service states periodically
            if last_update.elapsed() >= update_duration {
                self.update_states().await?;
                last_update = Instant::now();
            }
        }
    }

    /// Move to the next tab
    fn next_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Services => Tab::State,
            Tab::State => Tab::Plan,
            Tab::Plan => Tab::Deploy,
            Tab::Deploy => Tab::Logs,
            Tab::Logs => Tab::Config,
            Tab::Config => Tab::Help,
            Tab::Help => Tab::Services,
        };
    }

    /// Move to the previous tab
    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Services => Tab::Help,
            Tab::State => Tab::Services,
            Tab::Plan => Tab::State,
            Tab::Deploy => Tab::Plan,
            Tab::Logs => Tab::Deploy,
            Tab::Config => Tab::Logs,
            Tab::Help => Tab::Config,
        };
    }

    /// Select the previous service
    fn previous_service(&mut self) {
        if self.selected_service > 0 {
            self.selected_service -= 1;
        }
    }

    /// Select the next service
    fn next_service(&mut self) {
        let total_services = self.service_states.len() + self.available_services.len();
        if self.selected_service < total_services.saturating_sub(1) {
            self.selected_service += 1;
        }
    }

    /// Get the currently selected service name
    pub fn selected_service_name(&self) -> Option<&str> {
        let installed: Vec<_> = self.service_states.keys().collect();
        let available: Vec<_> = self.available_services.iter()
            .filter(|s| !self.service_states.contains_key(*s))
            .collect();
        
        if self.selected_service < installed.len() {
            Some(installed[self.selected_service])
        } else {
            let idx = self.selected_service - installed.len();
            available.get(idx).map(|s| s.as_str())
        }
    }

    /// Install the selected service
    async fn install_service(&mut self) -> Result<()> {
        if let Some(service_name) = self.selected_service_name() {
            if !self.service_states.contains_key(service_name) {
                // TODO: Show configuration dialog
                self.service_manager.install_service(service_name, None).await?;
                self.update_states().await?;
            }
        }
        Ok(())
    }

    /// Toggle the selected service (start/stop)
    async fn toggle_service(&mut self) -> Result<()> {
        if let Some(service_name) = self.selected_service_name() {
            if let Some(state) = self.service_states.get(service_name) {
                match state.status {
                    ServiceStatus::Running => {
                        self.service_manager.stop_service(service_name).await?;
                    }
                    ServiceStatus::Stopped => {
                        self.service_manager.start_service(service_name).await?;
                    }
                    _ => {}
                }
                self.update_states().await?;
            }
        }
        Ok(())
    }

    /// Restart the selected service
    async fn restart_service(&mut self) -> Result<()> {
        if let Some(service_name) = self.selected_service_name() {
            if self.service_states.contains_key(service_name) {
                self.service_manager.stop_service(service_name).await?;
                self.service_manager.start_service(service_name).await?;
                self.update_states().await?;
            }
        }
        Ok(())
    }

    /// Update service states
    async fn update_states(&mut self) -> Result<()> {
        self.service_states = self.service_manager.get_status(None).await
            .unwrap_or_default();
        self.last_update = Instant::now();
        Ok(())
    }
    
    /// Handle keys in Services tab
    async fn handle_services_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Up | KeyCode::Char('k') => self.previous_service(),
            KeyCode::Down | KeyCode::Char('j') => self.next_service(),
            KeyCode::Char('i') => self.install_service().await?,
            KeyCode::Char('s') => self.toggle_service().await?,
            KeyCode::Char('r') => self.restart_service().await?,
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in State tab
    async fn handle_state_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('r') => self.refresh_state().await?,
            KeyCode::Char('l') => self.lock_state().await?,
            KeyCode::Char('u') => self.unlock_state().await?,
            KeyCode::Up | KeyCode::Char('k') => self.previous_state_item(),
            KeyCode::Down | KeyCode::Char('j') => self.next_state_item(),
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in Plan tab
    async fn handle_plan_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('p') => self.create_plan().await?,
            KeyCode::Char('d') => self.discard_plan(),
            KeyCode::Up | KeyCode::Char('k') => self.previous_plan_item(),
            KeyCode::Down | KeyCode::Char('j') => self.next_plan_item(),
            _ => {}
        }
        Ok(())
    }
    
    /// Handle keys in Deploy tab
    async fn handle_deploy_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('a') => self.apply_plan().await?,
            KeyCode::Char('y') => self.approve_plan(),
            KeyCode::Char('n') => self.reject_plan(),
            _ => {}
        }
        Ok(())
    }
    
    /// Refresh state from actual resources
    async fn refresh_state(&mut self) -> Result<()> {
        self.state_operation = Some("Refreshing state...".to_string());
        
        // Lock state first
        if let Err(e) = self.state_manager.lock().await {
            self.state_operation = Some(format!("Failed to lock state: {}", e));
            return Ok(());
        }
        
        // Load current state
        match self.state_manager.load().await {
            Ok(state) => {
                self.firestream_state = Some(state.clone());
                self.state_operation = Some("State loaded successfully".to_string());
            }
            Err(e) => {
                self.state_operation = Some(format!("Failed to load state: {}", e));
            }
        }
        
        // Unlock state
        let _ = self.state_manager.unlock().await;
        
        Ok(())
    }
    
    /// Lock state
    async fn lock_state(&mut self) -> Result<()> {
        match self.state_manager.lock().await {
            Ok(_) => {
                self.state_operation = Some("State locked successfully".to_string());
            }
            Err(e) => {
                self.state_operation = Some(format!("Failed to lock state: {}", e));
            }
        }
        Ok(())
    }
    
    /// Unlock state
    async fn unlock_state(&mut self) -> Result<()> {
        match self.state_manager.unlock().await {
            Ok(_) => {
                self.state_operation = Some("State unlocked successfully".to_string());
            }
            Err(e) => {
                self.state_operation = Some(format!("Failed to unlock state: {}", e));
            }
        }
        Ok(())
    }
    
    /// Create execution plan
    async fn create_plan(&mut self) -> Result<()> {
        self.state_operation = Some("Creating execution plan...".to_string());
        
        // Lock state first
        if let Err(e) = self.state_manager.lock().await {
            self.state_operation = Some(format!("Failed to lock state: {}", e));
            return Ok(());
        }
        
        // Load config
        match self.config_manager.load_global_config().await {
            Ok(config) => {
                // Create plan
                match self.state_manager.plan(&config, vec![]).await {
                    Ok(plan) => {
                        self.current_plan = Some(plan);
                        self.state_operation = Some("Plan created successfully".to_string());
                    }
                    Err(e) => {
                        self.state_operation = Some(format!("Failed to create plan: {}", e));
                    }
                }
            }
            Err(e) => {
                self.state_operation = Some(format!("Failed to load config: {}", e));
            }
        }
        
        // Unlock state
        let _ = self.state_manager.unlock().await;
        
        Ok(())
    }
    
    /// Discard current plan
    fn discard_plan(&mut self) {
        self.current_plan = None;
        self.state_operation = Some("Plan discarded".to_string());
    }
    
    /// Apply current plan
    async fn apply_plan(&mut self) -> Result<()> {
        if let Some(plan) = &self.current_plan {
            if plan.status != PlanStatus::Approved && plan.status != PlanStatus::Pending {
                self.state_operation = Some("Plan must be approved before applying".to_string());
                return Ok(());
            }
            
            self.state_operation = Some("Applying plan...".to_string());
            
            // Lock state first
            if let Err(e) = self.state_manager.lock().await {
                self.state_operation = Some(format!("Failed to lock state: {}", e));
                return Ok(());
            }
            
            // Apply plan
            match self.state_manager.apply(&plan.id, plan.status == PlanStatus::Approved).await {
                Ok(_) => {
                    self.state_operation = Some("Plan applied successfully".to_string());
                    self.current_plan = None;
                    // Reload state
                    let _ = self.refresh_state().await;
                }
                Err(e) => {
                    self.state_operation = Some(format!("Failed to apply plan: {}", e));
                }
            }
            
            // Unlock state
            let _ = self.state_manager.unlock().await;
        } else {
            self.state_operation = Some("No plan to apply".to_string());
        }
        
        Ok(())
    }
    
    /// Approve current plan
    fn approve_plan(&mut self) {
        if let Some(plan) = &mut self.current_plan {
            if plan.status == PlanStatus::Pending {
                plan.status = PlanStatus::Approved;
                self.state_operation = Some("Plan approved".to_string());
            }
        }
    }
    
    /// Reject current plan
    fn reject_plan(&mut self) {
        if let Some(plan) = &mut self.current_plan {
            plan.status = PlanStatus::Cancelled;
            self.state_operation = Some("Plan rejected".to_string());
        }
    }
    
    /// Navigate state items
    fn previous_state_item(&mut self) {
        if self.selected_state_item > 0 {
            self.selected_state_item -= 1;
        }
    }
    
    fn next_state_item(&mut self) {
        // TODO: Implement proper bounds checking based on state content
        self.selected_state_item += 1;
    }
    
    /// Navigate plan items
    fn previous_plan_item(&mut self) {
        if self.selected_state_item > 0 {
            self.selected_state_item -= 1;
        }
    }
    
    fn next_plan_item(&mut self) {
        if let Some(plan) = &self.current_plan {
            if self.selected_state_item < plan.changes.len().saturating_sub(1) {
                self.selected_state_item += 1;
            }
        }
    }
}
