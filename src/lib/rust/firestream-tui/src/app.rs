use crate::backend::{FirestreamBackend, MockClient};
use crate::event::{AppEvent, Event, EventHandler};
use crate::models::{Deployment, Template, Node, DeltaTable, BuildStatus, SecretInfo, ResourceType};
use crate::views::{ResourcesPane, DetailsPane, LogsPane, View};
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use std::sync::Arc;
use tokio::sync::RwLock;

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

impl App {
    /// Create a new application instance
    pub fn new() -> Self {
        let backend = Arc::new(MockClient::new());
        
        Self {
            running: true,
            events: EventHandler::new(),
            backend,
            current_view: View::Main,
            focused_pane: Pane::Resources,
            resources: ResourceTree {
                items: vec![],
                selected: 0,
                expanded: vec![
                    "deployments".to_string(),
                    "templates".to_string(),
                ],
            },
            selected_details: None,
            logs: vec![],
            command_input: String::new(),
            command_mode: false,
            search_input: String::new(),
            search_mode: false,
            status_message: None,
            loading: LoadingState::default(),
        }
    }

    /// Run the application's main loop
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        // Load initial data
        self.load_resources().await;
        
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
        // Global shortcuts
        if self.command_mode {
            self.handle_command_mode_keys(key).await?;
        } else if self.search_mode {
            self.handle_search_mode_keys(key).await?;
        } else {
            match key.code {
                // Quit
                KeyCode::Char('q') => self.events.send(AppEvent::Quit),
                KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                
                // Command palette
                KeyCode::Char(':') => {
                    self.command_mode = true;
                    self.command_input.clear();
                }
                
                // Search
                KeyCode::Char('/') => {
                    self.search_mode = true;
                    self.search_input.clear();
                }
                
                // Navigation between panes
                KeyCode::Left | KeyCode::Char('h') => self.focus_previous_pane(),
                KeyCode::Right | KeyCode::Char('l') => self.focus_next_pane(),
                KeyCode::Tab => self.focus_next_pane(),
                KeyCode::BackTab => self.focus_previous_pane(),
                
                // Help
                KeyCode::Char('?') => self.current_view = View::Help,
                KeyCode::Esc if self.current_view == View::Help => self.current_view = View::Main,
                
                // Pane-specific navigation
                _ => match self.focused_pane {
                    Pane::Resources => self.handle_resources_keys(key).await?,
                    Pane::Details => self.handle_details_keys(key).await?,
                    Pane::Logs => self.handle_logs_keys(key).await?,
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
            KeyCode::Up | KeyCode::Char('k') => self.select_previous_resource(),
            KeyCode::Down | KeyCode::Char('j') => self.select_next_resource(),
            KeyCode::Enter => self.load_selected_details().await,
            KeyCode::Char(' ') => self.toggle_expand_resource(),
            
            // Quick actions
            KeyCode::Char('n') => self.new_resource().await,
            KeyCode::Char('d') => self.deploy_or_delete().await,
            KeyCode::Char('r') => self.refresh_resources().await,
            KeyCode::Char('b') => self.build_resource().await,
            _ => {}
        }
        Ok(())
    }

    /// Handle details pane navigation
    async fn handle_details_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Char('s') => self.scale_deployment().await,
            KeyCode::Char('r') => self.restart_deployment().await,
            KeyCode::Char('l') => self.view_logs().await,
            KeyCode::Char('e') => self.edit_resource().await,
            _ => {}
        }
        Ok(())
    }

    /// Handle logs pane navigation
    async fn handle_logs_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Char('f') => self.follow_logs().await,
            KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => self.stop_logs(),
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
                if let Some(pos) = self.resources.expanded.iter().position(|x| x == &id) {
                    self.resources.expanded.remove(pos);
                } else {
                    self.resources.expanded.push(id);
                }
                // Mark for refresh
                self.events.send(AppEvent::RefreshData);
            }
        }
    }

    /// Load resources from backend
    async fn load_resources(&mut self) {
        self.loading.resources = true;
        
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
                let mut python_items = vec![];
                let mut nodejs_items = vec![];
                
                for template in templates {
                    let item = ResourceItem {
                        id: format!("template:{}", template.id),
                        name: template.name.clone(),
                        resource_type: ResourceType::Template,
                        parent: Some(format!("templates.{:?}", template.template_type).to_lowercase()),
                        expandable: false,
                        depth: 2,
                        status: None,
                    };
                    
                    match template.template_type {
                        crate::models::TemplateType::PySpark | crate::models::TemplateType::PySparkScala => pyspark_items.push(item),
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
        
        self.resources.items = items;
        self.loading.resources = false;
    }

    /// Load details for selected resource
    async fn load_selected_details(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            if item.expandable {
                self.toggle_expand_resource();
                return;
            }
            
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
                    _ => {}
                }
            }
            
            self.loading.details = false;
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
}

// Helper trait for shared event sender
impl App {
    pub fn events(&self) -> &EventHandler {
        &self.events
    }
}
