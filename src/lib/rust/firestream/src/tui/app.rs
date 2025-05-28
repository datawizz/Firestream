//! TUI application state and logic
//!
//! This module manages the TUI application state and handles user interactions.

use crate::services::ServiceManager;
use crate::config::{ServiceState, ServiceStatus};
use crate::core::{FirestreamError, Result};
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
    
    /// Current tab
    pub current_tab: Tab,
    
    /// Selected service index
    pub selected_service: usize,
    
    /// Service states
    pub service_states: HashMap<String, ServiceState>,
    
    /// Available services
    pub available_services: Vec<String>,
    
    /// Should quit
    should_quit: bool,
    
    /// Last update time
    last_update: Instant,
}

/// Available tabs in the TUI
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Services,
    Logs,
    Config,
    Help,
}

impl App {
    /// Create a new TUI application
    pub async fn new() -> anyhow::Result<Self> {
        let service_manager = ServiceManager::new().await?;
        let service_states = service_manager.get_status(None).await
            .unwrap_or_default();
        let available_services = service_manager.list_available_services().await
            .unwrap_or_default();

        Ok(Self {
            service_manager,
            current_tab: Tab::Services,
            selected_service: 0,
            service_states,
            available_services,
            should_quit: false,
            last_update: Instant::now(),
        })
    }

    /// Run the TUI application
    pub async fn run(&mut self) -> Result<()> {
        // Setup terminal
        enable_raw_mode().map_err(|e| FirestreamError::IoError(e))?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)
            .map_err(|e| FirestreamError::IoError(e))?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)
            .map_err(|e| FirestreamError::IoError(e))?;

        // Run the app
        let res = self.run_app(&mut terminal).await;

        // Restore terminal
        disable_raw_mode().map_err(|e| FirestreamError::IoError(e))?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        ).map_err(|e| FirestreamError::IoError(e))?;
        terminal.show_cursor().map_err(|e| FirestreamError::IoError(e))?;

        res
    }

    /// Main application loop
    async fn run_app<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let mut last_update = Instant::now();
        let update_duration = Duration::from_secs(5);

        loop {
            // Draw UI
            terminal.draw(|f| crate::tui::ui::draw(f, self))
                .map_err(|e| FirestreamError::IoError(e))?;

            // Handle events with timeout
            if event::poll(Duration::from_millis(100))
                .map_err(|e| FirestreamError::IoError(e))? {
                if let Event::Key(key) = event::read()
                    .map_err(|e| FirestreamError::IoError(e))? {
                    match key.code {
                        KeyCode::Char('q') => self.should_quit = true,
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.should_quit = true;
                        }
                        KeyCode::Tab => self.next_tab(),
                        KeyCode::BackTab => self.previous_tab(),
                        KeyCode::Up | KeyCode::Char('k') => self.previous_service(),
                        KeyCode::Down | KeyCode::Char('j') => self.next_service(),
                        KeyCode::Char('i') => self.install_service().await?,
                        KeyCode::Char('s') => self.toggle_service().await?,
                        KeyCode::Char('r') => self.restart_service().await?,
                        KeyCode::Char('l') => self.current_tab = Tab::Logs,
                        KeyCode::Char('?') => self.current_tab = Tab::Help,
                        _ => {}
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
            Tab::Services => Tab::Logs,
            Tab::Logs => Tab::Config,
            Tab::Config => Tab::Help,
            Tab::Help => Tab::Services,
        };
    }

    /// Move to the previous tab
    fn previous_tab(&mut self) {
        self.current_tab = match self.current_tab {
            Tab::Services => Tab::Help,
            Tab::Logs => Tab::Services,
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
}
