use crate::backend::{FirestreamBackend, MockClient};
use crate::event::{AppEvent, Event, EventHandler};
use crate::models::{Template, DeltaTable, BuildStatus, SecretInfo, ResourceType, TemplateConfiguration, TemplateVariable};
use crate::services::TemplateService;
use crate::views::View;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use std::sync::Arc;
use std::collections::HashMap;

/// Main application state
pub struct App {
    /// Is the application running?
    pub running: bool,
    
    /// Event handler
    pub events: EventHandler,
    
    /// Backend API client
    backend: Arc<dyn FirestreamBackend>,
    
    /// Current view/mode
    pub current_view: View,
    
    /// Current focused pane
    pub focused_pane: Pane,
    
    /// Resources in left pane
    pub resources: ResourceTree,
    
    /// Selected resource details
    pub selected_details: Option<ResourceDetails>,
    
    /// Logs for bottom pane
    pub logs: Vec<String>,
    
    /// Command palette input
    pub command_input: String,
    pub command_mode: bool,
    
    /// Search input
    pub search_input: String,
    pub search_mode: bool,
    
    /// Status bar message
    pub status_message: Option<String>,
    
    /// Loading states
    pub loading: LoadingState,
    
    /// Secret editing state
    pub secret_editing: Option<SecretEditingState>,
    
    /// Template configuration state
    pub template_config: Option<TemplateConfigState>,
    
    /// Template generation state
    pub template_generation: Option<TemplateGenerationState>,
    
    /// Template service
    template_service: Option<TemplateService>,
}

#[derive(Debug, Clone)]
pub struct SecretEditingState {
    pub secret_name: String,
    pub fields: Vec<SecretField>,
    pub selected_field: usize,
    pub is_editing: bool,
}

#[derive(Debug, Clone)]
pub struct SecretField {
    pub key: String,
    pub value: String,
    pub is_password: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pane {
    Resources,
    Details,
    Logs,
}

#[derive(Debug, Clone)]
pub struct ResourceTree {
    pub items: Vec<ResourceItem>,
    pub selected: usize,
    pub expanded: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ResourceItem {
    pub id: String,
    pub name: String,
    pub resource_type: ResourceType,
    pub parent: Option<String>,
    pub expandable: bool,
    pub depth: usize,
    pub status: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ResourceDetails {
    Deployment(crate::models::DeploymentDetail),
    Template(Template),
    Node(crate::models::NodeDetail),
    Table(DeltaTable),
    Build(BuildStatus),
    Secret(SecretInfo),
}

#[derive(Debug, Default)]
pub struct LoadingState {
    pub resources: bool,
    pub details: bool,
    pub logs: bool,
}

#[derive(Debug, Clone)]
pub struct TemplateConfigState {
    pub template_id: String,
    pub template_type: String,
    pub configuration: TemplateConfiguration,
    pub user_values: HashMap<String, serde_json::Value>,
    pub validation_errors: HashMap<String, String>,
    pub expanded_groups: Vec<String>,
    pub selected_group: usize,
    pub selected_field: usize,
    pub editing_field: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TemplateGenerationState {
    pub request: crate::models::TemplateGenerationRequest,
    pub status: GenerationStatus,
    pub output_files: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum GenerationStatus {
    Idle,
    Generating,
    Success,
    Failed,
}

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let backend = Arc::new(MockClient::new());
        
        Self {
            running: true,
            events: EventHandler::new(),
            backend,
            current_view: View::Splash,
            focused_pane: Pane::Resources,
            resources: ResourceTree {
                items: vec![],
                selected: 0,
                expanded: vec![], // Start with everything collapsed
            },
            selected_details: None,
            logs: vec![],
            command_input: String::new(),
            command_mode: false,
            search_input: String::new(),
            search_mode: false,
            status_message: None,
            loading: LoadingState::default(),
            secret_editing: None,
            template_config: None,
            template_generation: None,
            template_service: match TemplateService::new() {
                Ok(service) => Some(service),
                Err(e) => {
                    eprintln!("[App] Failed to initialize template service: {}", e);
                    None
                }
            },
        }
    }

    /// Run the application's main loop
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        // Don't load resources on splash screen
        
        // Main event loop
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            
            match self.events.next().await? {
                Event::Tick => self.tick().await,
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => self.handle_key_events(key_event).await?,
                    _ => {}
                },
                Event::App(app_event) => self.handle_app_event(app_event).await,
            }
        }
        Ok(())
    }

    /// Handle key events
    pub async fn handle_key_events(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        // Clear old expand/collapse status messages on navigation keys
        match key.code {
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right | 
            KeyCode::Tab | KeyCode::BackTab => {
                if let Some(msg) = &self.status_message {
                    if msg.contains("Collapsed") || msg.contains("Expanding") || msg.contains("Expanded") {
                        self.status_message = None;
                    }
                }
            }
            _ => {}
        }
        
        // Global shortcuts
        if self.command_mode {
            self.handle_command_mode_keys(key).await?;
        } else if self.search_mode {
            self.handle_search_mode_keys(key).await?;
        } else if self.current_view == View::Splash {
            // Any key on splash screen advances to main view
            self.current_view = View::Main;
            self.load_resources().await;
        } else {
            match key.code {
                // Quit - only Ctrl+C
                KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                
                // Remove command palette and search shortcuts
                
                // Navigation and actions based on current pane
                _ => match self.focused_pane {
                    Pane::Resources => {
                        // In resources pane, Left/Right handle expand/collapse
                        match key.code {
                            KeyCode::Left => {
                                if let Some(item) = self.resources.items.get(self.resources.selected) {
                                    if item.expandable && self.resources.expanded.contains(&item.id) {
                                        self.toggle_expand_resource();
                                    }
                                }
                            }
                            KeyCode::Right => {
                                if let Some(item) = self.resources.items.get(self.resources.selected) {
                                    if item.expandable && !self.resources.expanded.contains(&item.id) {
                                        self.toggle_expand_resource();
                                    }
                                }
                            }
                            KeyCode::Tab => self.focus_next_pane(),
                            KeyCode::BackTab => self.focus_previous_pane(),
                            _ => self.handle_resources_keys(key).await?,
                        }
                    }
                    _ => {
                        // In other panes, Left/Right can navigate panes
                        match key.code {
                            KeyCode::Left => self.focus_previous_pane(),
                            KeyCode::Right => self.focus_next_pane(),
                            KeyCode::Tab => self.focus_next_pane(),
                            KeyCode::BackTab => self.focus_previous_pane(),
                            _ => match self.focused_pane {
                                Pane::Details => self.handle_details_keys(key).await?,
                                Pane::Logs => self.handle_logs_keys(key).await?,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Handle command mode keys
    async fn handle_command_mode_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.command_mode = false;
                self.command_input.clear();
            }
            KeyCode::Enter => {
                self.execute_command().await;
                self.command_mode = false;
                self.command_input.clear();
            }
            KeyCode::Backspace => {
                self.command_input.pop();
            }
            KeyCode::Char(c) => {
                self.command_input.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle search mode keys
    async fn handle_search_mode_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.search_mode = false;
                self.search_input.clear();
            }
            KeyCode::Enter => {
                self.perform_search().await;
                self.search_mode = false;
            }
            KeyCode::Backspace => {
                self.search_input.pop();
            }
            KeyCode::Char(c) => {
                self.search_input.push(c);
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle resources pane navigation
    async fn handle_resources_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Up => self.select_previous_resource(),
            KeyCode::Down => self.select_next_resource(),
            KeyCode::Enter => {
                if let Some(item) = self.resources.items.get(self.resources.selected) {
                    if !item.expandable {
                        // Only load details for non-expandable items
                        self.load_selected_details().await;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle details pane navigation
    async fn handle_details_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        // If we're configuring a template, handle template configuration input
        if let Some(ref mut config_state) = self.template_config {
            if let Some(ref editing_field) = config_state.editing_field {
                // We're editing a field
                match key.code {
                    KeyCode::Esc => {
                        config_state.editing_field = None;
                    }
                    KeyCode::Enter => {
                        config_state.editing_field = None;
                        // TODO: Validate the field
                    }
                    KeyCode::Backspace => {
                        // Find the variable being edited and its type
                        let var_type = config_state.configuration.variables
                            .get(editing_field)
                            .map(|v| &v.var_type);
                        
                        if let Some(value) = config_state.user_values.get_mut(editing_field) {
                            match var_type {
                                Some(crate::models::VariableType::Integer) => {
                                    // For integers, convert to string, modify, then back to number
                                    let mut str_val = value.as_i64().unwrap_or(0).to_string();
                                    str_val.pop();
                                    if let Ok(num) = str_val.parse::<i64>() {
                                        *value = serde_json::json!(num);
                                    } else if str_val.is_empty() {
                                        *value = serde_json::json!(0);
                                    }
                                }
                                _ => {
                                    // For strings and other types
                                    if let Some(s) = value.as_str() {
                                        let mut new_str = s.to_string();
                                        new_str.pop();
                                        *value = serde_json::json!(new_str);
                                    }
                                }
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        // Find the variable being edited and its type
                        let var_type = config_state.configuration.variables
                            .get(editing_field)
                            .map(|v| &v.var_type);
                        
                        if let Some(value) = config_state.user_values.get_mut(editing_field) {
                            match var_type {
                                Some(crate::models::VariableType::Integer) => {
                                    // For integers, only allow digits
                                    if c.is_ascii_digit() {
                                        let mut str_val = value.as_i64().unwrap_or(0).to_string();
                                        str_val.push(c);
                                        if let Ok(num) = str_val.parse::<i64>() {
                                            *value = serde_json::json!(num);
                                        }
                                    }
                                }
                                _ => {
                                    // For strings and other types
                                    if let Some(s) = value.as_str() {
                                        let mut new_str = s.to_string();
                                        new_str.push(c);
                                        *value = serde_json::json!(new_str);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            } else {
                // Navigation mode
                match key.code {
                    KeyCode::Up => {
                        // Find current position in groups
                        let mut current_group_idx = None;
                        let mut field_in_group = 0;
                        let mut absolute_field_idx = 0;
                        
                        for (group_idx, group) in config_state.configuration.variable_groups.iter().enumerate() {
                            if config_state.expanded_groups.contains(&group.name) {
                                let vars_in_group = config_state.configuration.variables.iter()
                                    .filter(|(_, v)| v.group == group.name)
                                    .count();
                                
                                if absolute_field_idx <= config_state.selected_field && 
                                   config_state.selected_field < absolute_field_idx + vars_in_group {
                                    current_group_idx = Some(group_idx);
                                    field_in_group = config_state.selected_field - absolute_field_idx;
                                    break;
                                }
                                absolute_field_idx += vars_in_group;
                            }
                        }
                        
                        if let Some(group_idx) = current_group_idx {
                            if field_in_group == 0 && group_idx > 0 {
                                // At first field of group, move to previous group
                                let current_group = &config_state.configuration.variable_groups[group_idx];
                                let current_group_name = current_group.display_name.clone();
                                config_state.expanded_groups.retain(|g| g != &current_group.name);
                                
                                // Find and expand previous group
                                let prev_group = &config_state.configuration.variable_groups[group_idx - 1];
                                let prev_group_name = prev_group.display_name.clone();
                                if !config_state.expanded_groups.contains(&prev_group.name) {
                                    config_state.expanded_groups.push(prev_group.name.clone());
                                }
                                
                                self.status_message = Some(format!("Navigated to {}", prev_group_name));
                                
                                // Recalculate selected_field for the last field of previous group
                                let mut new_field_idx = 0;
                                for (g_idx, g) in config_state.configuration.variable_groups.iter().enumerate() {
                                    if g_idx < group_idx - 1 {
                                        // Skip groups before the previous group
                                        continue;
                                    } else if g_idx == group_idx - 1 {
                                        // Count fields in the previous group to find the last one
                                        let vars_count = config_state.configuration.variables.iter()
                                            .filter(|(_, v)| v.group == g.name)
                                            .count();
                                        new_field_idx = vars_count.saturating_sub(1);
                                        break;
                                    }
                                }
                                config_state.selected_field = new_field_idx;
                            } else {
                                // Normal navigation within group
                                config_state.selected_field = config_state.selected_field.saturating_sub(1);
                            }
                        }
                    }
                    KeyCode::Down => {
                        // Find current position in groups
                        let mut current_group_idx = None;
                        let mut field_in_group = 0;
                        let mut absolute_field_idx = 0;
                        
                        for (group_idx, group) in config_state.configuration.variable_groups.iter().enumerate() {
                            if config_state.expanded_groups.contains(&group.name) {
                                let vars_in_group = config_state.configuration.variables.iter()
                                    .filter(|(_, v)| v.group == group.name)
                                    .count();
                                
                                if absolute_field_idx <= config_state.selected_field && 
                                   config_state.selected_field < absolute_field_idx + vars_in_group {
                                    current_group_idx = Some(group_idx);
                                    field_in_group = config_state.selected_field - absolute_field_idx;
                                    break;
                                }
                                absolute_field_idx += vars_in_group;
                            }
                        }
                        
                        if let Some(group_idx) = current_group_idx {
                            let current_group = &config_state.configuration.variable_groups[group_idx];
                            let vars_in_current_group = config_state.configuration.variables.iter()
                                .filter(|(_, v)| v.group == current_group.name)
                                .count();
                            
                            if field_in_group == vars_in_current_group - 1 && 
                               group_idx < config_state.configuration.variable_groups.len() - 1 {
                                // At last field of group, move to next group
                                let current_group_name = current_group.display_name.clone();
                                config_state.expanded_groups.retain(|g| g != &current_group.name);
                                
                                // Find and expand next group
                                let next_group = &config_state.configuration.variable_groups[group_idx + 1];
                                let next_group_name = next_group.display_name.clone();
                                if !config_state.expanded_groups.contains(&next_group.name) {
                                    config_state.expanded_groups.push(next_group.name.clone());
                                }
                                
                                self.status_message = Some(format!("Navigated to {}", next_group_name));
                                
                                // Move to first field of next group
                                config_state.selected_field = 0;
                            } else {
                                // Normal navigation within group or no more groups
                                let total_visible_fields = config_state.configuration.variables.iter()
                                    .filter(|(_, v)| config_state.expanded_groups.contains(&v.group))
                                    .count();
                                
                                if config_state.selected_field < total_visible_fields.saturating_sub(1) {
                                    config_state.selected_field += 1;
                                }
                            }
                        } else {
                            // No expanded groups, expand the first one
                            if !config_state.configuration.variable_groups.is_empty() {
                                let first_group = &config_state.configuration.variable_groups[0];
                                config_state.expanded_groups.push(first_group.name.clone());
                                config_state.selected_field = 0;
                            }
                        }
                    }
                    KeyCode::Enter | KeyCode::Char(' ') => {
                        // Find the variable at the current selected field
                        let mut field_index = 0;
                        let mut found_var: Option<(&String, &TemplateVariable)> = None;
                        
                        // Find the variable by iterating through all variables in order
                        for group in &config_state.configuration.variable_groups {
                            if config_state.expanded_groups.contains(&group.name) {
                                let vars_in_group: Vec<_> = config_state.configuration.variables.iter()
                                    .filter(|(_, v)| v.group == group.name)
                                    .collect();
                                
                                for (var_name, var) in vars_in_group {
                                    if field_index == config_state.selected_field {
                                        found_var = Some((var_name, var));
                                        break;
                                    }
                                    field_index += 1;
                                }
                            }
                            if found_var.is_some() {
                                break;
                            }
                        }
                        
                        if let Some((var_name, var)) = found_var {
                            match &var.var_type {
                                crate::models::VariableType::Boolean => {
                                    // Toggle boolean value
                                    if let Some(value) = config_state.user_values.get_mut(var_name) {
                                        if let Some(b) = value.as_bool() {
                                            *value = serde_json::json!(!b);
                                        }
                                    }
                                }
                                crate::models::VariableType::String | 
                                crate::models::VariableType::Integer | 
                                crate::models::VariableType::Float => {
                                    // Start editing text field
                                    config_state.editing_field = Some(var_name.clone());
                                }
                                _ => {}
                            }
                        }
                    }

                    KeyCode::Char('g' | 'G') => {
                        // Generate template
                        self.generate_template();
                    }
                    KeyCode::Char('r' | 'R') => {
                        // Reset to defaults
                        self.reset_template_config();
                    }
                    KeyCode::Char('v' | 'V') => {
                        // Validate configuration
                        self.validate_template_config();
                    }
                    _ => {}
                }
            }
            return Ok(());
        }
        
        // If we're editing a secret, handle text input
        if let Some(ref mut secret_state) = self.secret_editing {
            if secret_state.is_editing {
                match key.code {
                    KeyCode::Esc => {
                        secret_state.is_editing = false;
                    }
                    KeyCode::Enter => {
                        secret_state.is_editing = false;
                        // TODO: Save the secret value
                        self.status_message = Some("Secret saved".to_string());
                    }
                    KeyCode::Backspace => {
                        if let Some(field) = secret_state.fields.get_mut(secret_state.selected_field) {
                            field.value.pop();
                        }
                    }
                    KeyCode::Char(c) => {
                        if let Some(field) = secret_state.fields.get_mut(secret_state.selected_field) {
                            field.value.push(c);
                        }
                    }
                    _ => {}
                }
            } else {
                // Navigate between fields when not editing
                match key.code {
                    KeyCode::Up => {
                        if secret_state.selected_field > 0 {
                            secret_state.selected_field -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if secret_state.selected_field < secret_state.fields.len() - 1 {
                            secret_state.selected_field += 1;
                        }
                    }
                    KeyCode::Enter => {
                        secret_state.is_editing = true;
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Handle logs pane navigation
    async fn handle_logs_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            _ => {}
        }
        Ok(())
    }

    /// Handle application events
    async fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.quit(),
            AppEvent::RefreshData => self.refresh_all().await,
            AppEvent::LoadComplete(resource_type) => self.handle_load_complete(resource_type),
        }
    }

    /// Periodic tick
    pub async fn tick(&self) {
        // Update any time-based UI elements
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Focus previous pane
    fn focus_previous_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            Pane::Resources => Pane::Logs,
            Pane::Details => Pane::Resources,
            Pane::Logs => Pane::Details,
        };
    }

    /// Focus next pane
    fn focus_next_pane(&mut self) {
        self.focused_pane = match self.focused_pane {
            Pane::Resources => Pane::Details,
            Pane::Details => Pane::Logs,
            Pane::Logs => Pane::Resources,
        };
    }

    /// Select previous resource
    fn select_previous_resource(&mut self) {
        if self.resources.selected > 0 {
            self.resources.selected -= 1;
        }
    }

    /// Select next resource
    fn select_next_resource(&mut self) {
        if self.resources.selected < self.resources.items.len().saturating_sub(1) {
            self.resources.selected += 1;
        }
    }

    /// Toggle expand/collapse for current resource
    fn toggle_expand_resource(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            if item.expandable {
                let id = item.id.clone();
                let name = item.name.clone();
                
                // Log the action
                self.logs.push(format!("[{}] Toggle expand/collapse for '{}' (id: {})", 
                    chrono::Local::now().format("%H:%M:%S"), name, id));
                
                // Check current state and toggle
                let is_expanded = self.resources.expanded.contains(&id);
                self.logs.push(format!("[{}] Current state: {} (expanded list has {} items)", 
                    chrono::Local::now().format("%H:%M:%S"), 
                    if is_expanded { "EXPANDED" } else { "COLLAPSED" },
                    self.resources.expanded.len()));
                
                if is_expanded {
                    // Collapsing
                    self.resources.expanded.retain(|x| x != &id);
                    self.status_message = Some(format!("Collapsed {}", name));
                    self.logs.push(format!("[{}] Collapsing '{}' - removed from expanded list", 
                        chrono::Local::now().format("%H:%M:%S"), name));
                    // Collapse: Remove children from the tree immediately
                    self.collapse_resource(&id);
                } else {
                    // Expanding
                    self.resources.expanded.push(id.clone());
                    self.status_message = Some(format!("Expanding {}", name));
                    self.logs.push(format!("[{}] Expanding '{}' - added to expanded list", 
                        chrono::Local::now().format("%H:%M:%S"), name));
                    self.logs.push(format!("[{}] Expanded list now contains: {:?}", 
                        chrono::Local::now().format("%H:%M:%S"), self.resources.expanded));
                    // Expand: Trigger a refresh to load children
                    self.logs.push(format!("[{}] Sending RefreshData event", 
                        chrono::Local::now().format("%H:%M:%S")));
                    self.events.send(AppEvent::RefreshData);
                }
                
                // Keep only the last 100 log entries
                if self.logs.len() > 100 {
                    self.logs.drain(0..self.logs.len() - 100);
                }
            } else {
                self.logs.push(format!("[{}] Item '{}' is not expandable", 
                    chrono::Local::now().format("%H:%M:%S"), item.name));
            }
        } else {
            self.logs.push(format!("[{}] No item selected", 
                chrono::Local::now().format("%H:%M:%S")));
        }
    }

    /// Collapse a resource by removing its children from the tree
    fn collapse_resource(&mut self, parent_id: &str) {
        self.logs.push(format!("[{}] collapse_resource called for '{}'", 
            chrono::Local::now().format("%H:%M:%S"), parent_id));
            
        // Find the parent index
        if let Some(parent_idx) = self.resources.items.iter().position(|item| item.id == parent_id) {
            let parent_depth = self.resources.items[parent_idx].depth;
            self.logs.push(format!("[{}] Found parent at index {} with depth {}", 
                chrono::Local::now().format("%H:%M:%S"), parent_idx, parent_depth));
            
            // Count how many items to remove
            let mut remove_count = 0;
            let mut i = parent_idx + 1;
            
            // Find all items that should be removed (children at any depth)
            while i < self.resources.items.len() {
                let item = &self.resources.items[i];
                
                // If we hit an item at the same or lower depth as parent, we're done
                if item.depth <= parent_depth {
                    break;
                }
                
                remove_count += 1;
                i += 1;
            }
            
            self.logs.push(format!("[{}] Will remove {} children", 
                chrono::Local::now().format("%H:%M:%S"), remove_count));
            
            // Remove the items
            for _ in 0..remove_count {
                let removed = self.resources.items.remove(parent_idx + 1);
                self.logs.push(format!("[{}] Removed item: '{}'", 
                    chrono::Local::now().format("%H:%M:%S"), removed.name));
                
                // Adjust selection if needed
                if self.resources.selected > parent_idx {
                    self.resources.selected = self.resources.selected.saturating_sub(1);
                }
            }
        } else {
            self.logs.push(format!("[{}] Could not find parent '{}' in tree", 
                chrono::Local::now().format("%H:%M:%S"), parent_id));
        }
    }


    /// Load resources from backend
    async fn load_resources(&mut self) {
        self.loading.resources = true;
        
        // Save current selection ID to restore after refresh
        let selected_id = self.resources.items.get(self.resources.selected)
            .map(|item| item.id.clone());
        
        // Build resource tree
        let mut items = vec![];
        
        // Deployments section
        items.push(ResourceItem {
            id: "deployments".to_string(),
            name: "deployments".to_string(),
            resource_type: ResourceType::Deployment,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
        });
        
        if self.resources.expanded.contains(&"deployments".to_string()) {
            if let Ok(deployments) = self.backend.list_deployments().await {
                for deployment in deployments {
                    items.push(ResourceItem {
                        id: format!("deployment:{}", deployment.id),
                        name: deployment.name.clone(),
                        resource_type: ResourceType::Deployment,
                        parent: Some("deployments".to_string()),
                        expandable: false,
                        depth: 1,
                        status: Some(format!("{:?}", deployment.status)),
                    });
                }
            }
        }
        
        // Templates section
        items.push(ResourceItem {
            id: "templates".to_string(),
            name: "templates".to_string(),
            resource_type: ResourceType::Template,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
        });
        
        if self.resources.expanded.contains(&"templates".to_string()) {
            if let Ok(templates) = self.backend.list_templates().await {
                // Group templates by type
                let mut pyspark_items = vec![];
                let mut scala_items = vec![];
                let mut python_items = vec![];
                let mut nodejs_items = vec![];
                
                for template in templates {
                    let item = ResourceItem {
                        id: format!("template:{}", template.id),
                        name: template.name.clone(),
                        resource_type: ResourceType::Template,
                        parent: Some(match template.template_type {
                            crate::models::TemplateType::PySpark => "templates.pyspark".to_string(),
                            crate::models::TemplateType::PySparkScala => "templates.scala".to_string(),
                            crate::models::TemplateType::Python => "templates.python".to_string(),
                            crate::models::TemplateType::NodeJs => "templates.nodejs".to_string(),
                        }),
                        expandable: false,
                        depth: 2,
                        status: None,
                    };
                    
                    match template.template_type {
                        crate::models::TemplateType::PySpark => pyspark_items.push(item),
                        crate::models::TemplateType::PySparkScala => scala_items.push(item),
                        crate::models::TemplateType::Python => python_items.push(item),
                        crate::models::TemplateType::NodeJs => nodejs_items.push(item),
                    }
                }
                
                // Add template type groups
                if !pyspark_items.is_empty() {
                    items.push(ResourceItem {
                        id: "templates.pyspark".to_string(),
                        name: "pyspark".to_string(),
                        resource_type: ResourceType::Template,
                        parent: Some("templates".to_string()),
                        expandable: true,
                        depth: 1,
                        status: None,
                    });
                    if self.resources.expanded.contains(&"templates.pyspark".to_string()) {
                        items.extend(pyspark_items);
                    }
                }
                
                if !scala_items.is_empty() {
                    items.push(ResourceItem {
                        id: "templates.scala".to_string(),
                        name: "scala".to_string(),
                        resource_type: ResourceType::Template,
                        parent: Some("templates".to_string()),
                        expandable: true,
                        depth: 1,
                        status: None,
                    });
                    if self.resources.expanded.contains(&"templates.scala".to_string()) {
                        items.extend(scala_items);
                    }
                }
                
                if !python_items.is_empty() {
                    items.push(ResourceItem {
                        id: "templates.python".to_string(),
                        name: "python".to_string(),
                        resource_type: ResourceType::Template,
                        parent: Some("templates".to_string()),
                        expandable: true,
                        depth: 1,
                        status: None,
                    });
                    if self.resources.expanded.contains(&"templates.python".to_string()) {
                        items.extend(python_items);
                    }
                }
                
                if !nodejs_items.is_empty() {
                    items.push(ResourceItem {
                        id: "templates.nodejs".to_string(),
                        name: "nodejs".to_string(),
                        resource_type: ResourceType::Template,
                        parent: Some("templates".to_string()),
                        expandable: true,
                        depth: 1,
                        status: None,
                    });
                    if self.resources.expanded.contains(&"templates.nodejs".to_string()) {
                        items.extend(nodejs_items);
                    }
                }
            }
        }
        
        // Nodes section
        items.push(ResourceItem {
            id: "nodes".to_string(),
            name: "nodes".to_string(),
            resource_type: ResourceType::Node,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
        });
        
        if self.resources.expanded.contains(&"nodes".to_string()) {
            if let Ok(nodes) = self.backend.list_nodes().await {
                for node in nodes {
                    items.push(ResourceItem {
                        id: format!("node:{}", node.id),
                        name: node.name.clone(),
                        resource_type: ResourceType::Node,
                        parent: Some("nodes".to_string()),
                        expandable: false,
                        depth: 1,
                        status: Some(format!("{:?}", node.status)),
                    });
                }
            }
        }
        
        // Data section
        items.push(ResourceItem {
            id: "data".to_string(),
            name: "data".to_string(),
            resource_type: ResourceType::Data,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
        });
        
        // Builds section
        items.push(ResourceItem {
            id: "builds".to_string(),
            name: "builds".to_string(),
            resource_type: ResourceType::Build,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
        });
        
        // Secrets section
        items.push(ResourceItem {
            id: "secrets".to_string(),
            name: "secrets".to_string(),
            resource_type: ResourceType::Secret,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
        });
        
        if self.resources.expanded.contains(&"secrets".to_string()) {
            if let Ok(secrets) = self.backend.list_secrets().await {
                for secret in secrets {
                    items.push(ResourceItem {
                        id: format!("secret:{}", secret.name),
                        name: secret.name.clone(),
                        resource_type: ResourceType::Secret,
                        parent: Some("secrets".to_string()),
                        expandable: false,
                        depth: 1,
                        status: None,
                    });
                }
            }
        }
        
        self.resources.items = items;
        
        // Restore selection if possible
        if let Some(id) = selected_id {
            if let Some(pos) = self.resources.items.iter().position(|item| item.id == id) {
                self.resources.selected = pos;
            }
        }
        
        self.loading.resources = false;
    }

    /// Load details for selected resource
    async fn load_selected_details(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            // Only process non-expandable items
            if !item.expandable {
                self.loading.details = true;
            
            // Parse resource ID
            if let Some((resource_type, id)) = item.id.split_once(':') {
                match resource_type {
                    "deployment" => {
                        if let Ok(details) = self.backend.get_deployment(id).await {
                            self.selected_details = Some(ResourceDetails::Deployment(details));
                            self.focused_pane = Pane::Details;
                        }
                    }
                    "template" => {
                        if let Ok(template) = self.backend.get_template(id).await {
                            // Load template configuration
                            if let Some(ref service) = self.template_service {
                                let template_type = match template.template_type {
                                    crate::models::TemplateType::PySpark => "pyspark",
                                    crate::models::TemplateType::PySparkScala => "spark-scala",
                                    _ => "pyspark", // default
                                };
                                
                                if let Ok(config) = service.get_configuration(template_type) {
                                    // Initialize template config state
                                    let mut user_values = HashMap::new();
                                    let expanded_groups = vec!["core".to_string()]; // Start with core expanded
                                    
                                    // Add default values
                                    for (var_name, var) in &config.variables {
                                        if let Some(default) = &var.default_value {
                                            user_values.insert(var_name.clone(), default.clone());
                                        }
                                    }
                                    
                                    self.template_config = Some(TemplateConfigState {
                                        template_id: template.id.clone(),
                                        template_type: template_type.to_string(),
                                        configuration: config.clone(),
                                        user_values,
                                        validation_errors: HashMap::new(),
                                        expanded_groups,
                                        selected_group: 0,
                                        selected_field: 0,
                                        editing_field: None,
                                    });
                                }
                            }
                            
                            self.selected_details = Some(ResourceDetails::Template(template));
                            self.focused_pane = Pane::Details;
                        }
                    }
                    "node" => {
                        if let Ok(details) = self.backend.get_node(id).await {
                            self.selected_details = Some(ResourceDetails::Node(details));
                            self.focused_pane = Pane::Details;
                        }
                    }
                    "secret" => {
                        if let Ok(secret) = self.backend.get_secret(id).await {
                            // Initialize secret editing state
                            self.secret_editing = Some(SecretEditingState {
                                secret_name: secret.name.clone(),
                                fields: secret.keys.iter().map(|key| SecretField {
                                    key: key.clone(),
                                    value: String::new(),
                                    is_password: key.contains("password") || key.contains("secret") || key.contains("key"),
                                }).collect(),
                                selected_field: 0,
                                is_editing: false,
                            });
                            self.selected_details = Some(ResourceDetails::Secret(secret));
                            self.focused_pane = Pane::Details;
                        }
                    }
                    _ => {
                        // Clear secret editing state if we're not selecting a secret
                        self.secret_editing = None;
                    }
                }
            }
            
                self.loading.details = false;
            }
        }
    }

    /// Execute command from command palette
    async fn execute_command(&mut self) {
        let parts: Vec<String> = self.command_input.split_whitespace().map(|s| s.to_string()).collect();
        if parts.is_empty() {
            return;
        }
        
        match parts[0].as_str() {
            "deploy" => {
                if parts.len() > 1 {
                    self.deploy_template(&parts[1]).await;
                } else {
                    self.current_view = View::NewDeployment;
                }
            }
            "scale" => {
                if parts.len() > 2 {
                    if let Ok(replicas) = parts[2].parse::<u32>() {
                        self.scale_deployment_to(&parts[1], replicas).await;
                    }
                }
            }
            "logs" => {
                if parts.len() > 1 {
                    self.view_logs_for(&parts[1]).await;
                }
            }
            _ => {
                self.status_message = Some(format!("Unknown command: {}", parts[0]));
            }
        }
    }

    /// Perform search
    async fn perform_search(&mut self) {
        self.current_view = View::Search(self.search_input.clone());
        // TODO: Implement search functionality
    }

    // Stub methods for various actions
    async fn new_resource(&mut self) {
        // TODO: Show new resource dialog
        self.status_message = Some("New resource dialog not implemented".to_string());
    }

    async fn deploy_or_delete(&mut self) {
        // TODO: Implement deploy/delete
        self.status_message = Some("Deploy/delete not implemented".to_string());
    }

    async fn refresh_resources(&mut self) {
        self.load_resources().await;
    }

    async fn build_resource(&mut self) {
        // TODO: Implement build
        self.status_message = Some("Build not implemented".to_string());
    }

    async fn scale_deployment(&mut self) {
        // TODO: Show scale dialog
        self.status_message = Some("Scale dialog not implemented".to_string());
    }

    async fn restart_deployment(&mut self) {
        // TODO: Implement restart
        self.status_message = Some("Restart not implemented".to_string());
    }

    async fn view_logs(&mut self) {
        // TODO: Load logs for selected resource
        self.focused_pane = Pane::Logs;
        self.logs = vec![
            "Loading logs...".to_string(),
            "Logs will appear here".to_string(),
        ];
    }

    async fn edit_resource(&mut self) {
        // TODO: Show edit dialog
        self.status_message = Some("Edit not implemented".to_string());
    }

    async fn follow_logs(&mut self) {
        // TODO: Implement log following
        self.status_message = Some("Following logs...".to_string());
    }

    fn stop_logs(&mut self) {
        self.status_message = Some("Stopped following logs".to_string());
    }

    async fn refresh_all(&mut self) {
        // Clear any stale status messages before refresh
        if let Some(msg) = &self.status_message {
            if msg.starts_with("Expanding") {
                // Update to show expansion completed
                if let Some(name) = msg.strip_prefix("Expanding ") {
                    self.status_message = Some(format!("Expanded {}", name));
                }
            }
        }
        
        self.load_resources().await;
        if self.selected_details.is_some() {
            self.load_selected_details().await;
        }
    }

    fn handle_load_complete(&mut self, resource_type: ResourceType) {
        self.status_message = Some(format!("Loaded {}", resource_type));
    }

    async fn deploy_template(&mut self, template_name: &str) {
        // TODO: Implement template deployment
        self.status_message = Some(format!("Deploying template: {}", template_name));
    }

    async fn scale_deployment_to(&mut self, deployment_name: &str, replicas: u32) {
        // TODO: Implement scaling
        self.status_message = Some(format!("Scaling {} to {} replicas", deployment_name, replicas));
    }

    async fn view_logs_for(&mut self, resource_name: &str) {
        // TODO: Load logs for specific resource
        self.status_message = Some(format!("Loading logs for {}", resource_name));
    }
    
    /// Generate template with current configuration
    fn generate_template(&mut self) {
        // First check if template service is available
        if self.template_service.is_none() {
            let error_msg = "Template service not available - failed to initialize";
            self.status_message = Some(format!("Generation failed: {}", error_msg));
            self.logs.push(format!(
                "[{}] ERROR: {}",
                chrono::Local::now().format("%H:%M:%S"),
                error_msg
            ));
            self.logs.push(format!(
                "[{}] This usually means the template files could not be loaded",
                chrono::Local::now().format("%H:%M:%S")
            ));
            self.focused_pane = Pane::Logs;
            return;
        }
        
        // Check if template config is available
        if self.template_config.is_none() {
            let error_msg = "No template configuration loaded";
            self.status_message = Some(format!("Generation failed: {}", error_msg));
            self.logs.push(format!(
                "[{}] ERROR: {}",
                chrono::Local::now().format("%H:%M:%S"),
                error_msg
            ));
            self.focused_pane = Pane::Logs;
            return;
        }
        
        if let (Some(ref mut service), Some(ref config_state)) = (&mut self.template_service, &self.template_config) {
            // Create generation request
            let app_name = config_state.user_values.get("app_name")
                .and_then(|v| v.as_str())
                .unwrap_or("generated-app")
                .to_string()
                .to_lowercase()
                .replace(' ', "-");
            
            // Create output path - use current directory if /workspace/out doesn't exist
            let output_path = if std::path::Path::new("/workspace/out").exists() {
                format!("/workspace/out/{}", app_name)
            } else {
                format!("./generated/{}", app_name)
            };
            
            let request = crate::models::TemplateGenerationRequest {
                template_id: config_state.template_id.clone(),
                template_type: config_state.template_type.clone(),
                name: app_name.clone(),
                output_path,
                variables: config_state.user_values.clone(),
            };
            
            self.logs.push(format!(
                "[{}] Generating template: type={}, app_name={}",
                chrono::Local::now().format("%H:%M:%S"),
                config_state.template_type,
                app_name
            ));
            self.logs.push(format!(
                "[{}] Output path: {}",
                chrono::Local::now().format("%H:%M:%S"),
                request.output_path
            ));
            
            // Generate
            match service.generate(&request) {
                Ok(result) => {
                    self.status_message = Some(format!(
                        "Generated {} files in {}", 
                        result.generated_files.len(),
                        result.output_path
                    ));
                    self.logs.push(format!(
                        "[{}] Template generated successfully: {} files",
                        chrono::Local::now().format("%H:%M:%S"),
                        result.generated_files.len()
                    ));
                    for file in &result.generated_files {
                        self.logs.push(format!("  - {}", file.path));
                    }
                    
                    // Keep only the last 100 log entries
                    if self.logs.len() > 100 {
                        self.logs.drain(0..self.logs.len() - 100);
                    }
                }
                Err(e) => {
                    // IMMEDIATE LOG TO ENSURE THIS PATH IS HIT
                    self.logs.push("ERROR PATH HIT!".to_string());
                    
                    // Check if error already contains "Generation failed:"
                    let error_str = e.to_string();
                    let error_msg = if error_str.starts_with("Generation failed:") {
                        error_str
                    } else {
                        format!("Generation failed: {}", e)
                    };
                    self.status_message = Some(error_msg.clone());
                    
                    // Log detailed error to logs
                    self.logs.push(format!(
                        "[{}] ERROR: Template generation failed",
                        chrono::Local::now().format("%H:%M:%S")
                    ));
                    self.logs.push(format!("[{}] Error details: {}", 
                        chrono::Local::now().format("%H:%M:%S"),
                        e
                    ));
                    
                    // Check if it's a directory creation issue
                    if error_msg.contains("workspace") || error_msg.contains("directory") {
                        self.logs.push(format!(
                            "[{}] Note: Ensure /workspace/out directory exists and is writable",
                            chrono::Local::now().format("%H:%M:%S")
                        ));
                    }
                    
                    // Keep only the last 100 log entries
                    if self.logs.len() > 100 {
                        self.logs.drain(0..self.logs.len() - 100);
                    }
                    
                    // Focus on logs pane to show the error
                    self.focused_pane = Pane::Logs;
                }
            }
        } else {
            // This should never happen given our checks above
            self.logs.push(format!(
                "[{}] ERROR: Unexpected state - template service or config is None after checks",
                chrono::Local::now().format("%H:%M:%S")
            ));
        }
    }
    
    /// Reset template configuration to defaults
    fn reset_template_config(&mut self) {
        if let Some(ref mut config_state) = self.template_config {
            // Reset all values to defaults
            config_state.user_values.clear();
            for (var_name, var) in &config_state.configuration.variables {
                if let Some(default) = &var.default_value {
                    config_state.user_values.insert(var_name.clone(), default.clone());
                }
            }
            config_state.validation_errors.clear();
            self.status_message = Some("Reset to default values".to_string());
        }
    }
    
    /// Validate template configuration
    fn validate_template_config(&mut self) {
        if let (Some(ref service), Some(ref mut config_state)) = (&self.template_service, &mut self.template_config) {
            match service.validate_configuration(&config_state.template_type, &config_state.user_values) {
                Ok(errors) => {
                    config_state.validation_errors.clear();
                    for error in errors {
                        config_state.validation_errors.insert(error.field, error.message);
                    }
                    
                    if config_state.validation_errors.is_empty() {
                        self.status_message = Some("Configuration is valid".to_string());
                    } else {
                        self.status_message = Some(format!(
                            "Found {} validation errors",
                            config_state.validation_errors.len()
                        ));
                    }
                }
                Err(e) => {
                    self.status_message = Some(format!("Validation failed: {}", e));
                }
            }
        }
    }


}

// Helper trait for shared event sender
impl App {
    pub fn events(&self) -> &EventHandler {
        &self.events
    }
}
