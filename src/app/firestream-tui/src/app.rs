use crate::backend::{BootstrapState, BootstrapChecker, FirestreamBackend};
use crate::event::{AppEvent, Event, EventHandler};
use crate::models::{Template, DeltaTable, BuildStatus, SecretInfo, ResourceType};
use crate::views::View;
use ratatui::{
    DefaultTerminal,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Whether the backend has been initialized yet
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BackendState {
    /// Not yet initialized — showing splash screen
    Pending,
    /// Bootstrap + backend are ready
    Ready,
}

/// Main application state
pub struct App {
    /// Is the application running?
    pub running: bool,

    /// Event handler
    pub events: EventHandler,

    /// Event sender (cloned to share with LocalBackend)
    event_sender: mpsc::UnboundedSender<Event>,

    /// Backend API client
    backend: Arc<dyn FirestreamBackend>,

    /// Whether backend has been initialized
    backend_state: BackendState,

    /// Bootstrap state (Docker environment info)
    pub bootstrap: BootstrapState,

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
    pub secondary_status: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ResourceDetails {
    Deployment(crate::models::DeploymentDetail),
    Template(Template),
    Node(crate::models::NodeDetail),
    Table(DeltaTable),
    Build(BuildStatus),
    Secret(SecretInfo),
    Container(crate::backend::ContainerManifest),
}

#[derive(Debug, Default)]
pub struct LoadingState {
    pub resources: bool,
    pub details: bool,
    pub logs: bool,
}

impl App {
    /// Create a new application instance (instant — no blocking I/O)
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let events = EventHandler::from_parts(sender.clone(), receiver);

        Self {
            running: true,
            events,
            event_sender: sender,
            backend: Arc::new(crate::backend::MockClient::new()),
            backend_state: BackendState::Pending,
            bootstrap: BootstrapState::default(),
            current_view: View::Splash,
            focused_pane: Pane::Resources,
            resources: ResourceTree {
                items: vec![],
                selected: 0,
                expanded: vec![],
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

    /// Initialize the real backend (called when leaving splash screen)
    async fn bootstrap_backend(&mut self) {
        self.status_message = Some("Checking Docker environment...".into());

        let bootstrap = BootstrapChecker::check(None).await;

        if bootstrap.docker_available {
            match docker_manager::DockerManager::new().await {
                Ok(docker) => {
                    match nix_container_builder::NixContainerBuilder::with_config(
                        nix_container_builder::BuildConfig::default(),
                    ).await {
                        Ok(builder) => {
                            let repo_root = bootstrap.repo_root.clone()
                                .unwrap_or_else(|| PathBuf::from("."));
                            self.backend = Arc::new(
                                crate::backend::LocalBackend::new(
                                    docker, builder, repo_root,
                                    bootstrap.clone(), self.event_sender.clone(),
                                ).await,
                            );
                        }
                        Err(e) => {
                            self.logs.push(format!(
                                "[{}] NixContainerBuilder init failed: {}",
                                chrono::Local::now().format("%H:%M:%S"), e
                            ));
                        }
                    }
                }
                Err(e) => {
                    self.logs.push(format!(
                        "[{}] Docker connection failed: {}",
                        chrono::Local::now().format("%H:%M:%S"), e
                    ));
                }
            }
        }

        self.bootstrap = bootstrap;
        self.backend_state = BackendState::Ready;
        self.status_message = None;
    }

    /// Run the application's main loop
    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

            match self.events.next().await? {
                Event::Tick => self.tick().await,
                Event::Crossterm(event) => match event {
                    crossterm::event::Event::Key(key_event) => {
                        self.handle_key_events(key_event).await?
                    }
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
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
            | KeyCode::Tab | KeyCode::BackTab => {
                if let Some(msg) = &self.status_message {
                    if msg.contains("Collapsed")
                        || msg.contains("Expanding")
                        || msg.contains("Expanded")
                    {
                        self.status_message = None;
                    }
                }
            }
            _ => {}
        }

        if self.current_view == View::Splash {
            // Any key on splash screen advances to main view
            self.current_view = View::Main;
            if self.backend_state == BackendState::Pending {
                self.bootstrap_backend().await;
            }
            self.load_resources().await;
        } else {
            match key.code {
                // Quit - only Ctrl+C
                KeyCode::Char('c' | 'C') if key.modifiers == KeyModifiers::CONTROL => {
                    self.events.send(AppEvent::Quit)
                }
                // Navigation based on current pane
                _ => match self.focused_pane {
                    Pane::Resources => match key.code {
                        KeyCode::Left => {
                            if let Some(item) =
                                self.resources.items.get(self.resources.selected)
                            {
                                if item.expandable
                                    && self.resources.expanded.contains(&item.id)
                                {
                                    self.toggle_expand_resource();
                                }
                            }
                        }
                        KeyCode::Right => {
                            if let Some(item) =
                                self.resources.items.get(self.resources.selected)
                            {
                                if item.expandable
                                    && !self.resources.expanded.contains(&item.id)
                                {
                                    self.toggle_expand_resource();
                                }
                            }
                        }
                        KeyCode::Tab => self.focus_next_pane(),
                        KeyCode::BackTab => self.focus_previous_pane(),
                        _ => self.handle_resources_keys(key).await?,
                    },
                    _ => match key.code {
                        KeyCode::Left => self.focus_previous_pane(),
                        KeyCode::Right => self.focus_next_pane(),
                        KeyCode::Tab => self.focus_next_pane(),
                        KeyCode::BackTab => self.focus_previous_pane(),
                        _ => match self.focused_pane {
                            Pane::Details => self.handle_details_keys(key).await?,
                            Pane::Logs => self.handle_logs_keys(key).await?,
                            _ => {}
                        },
                    },
                },
            }
        }
        Ok(())
    }

    /// Handle resources pane navigation
    async fn handle_resources_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        match key.code {
            KeyCode::Up => self.select_previous_resource(),
            KeyCode::Down => self.select_next_resource(),
            KeyCode::Char(' ') => {
                if let Some(item) = self.resources.items.get(self.resources.selected) {
                    if item.name.contains("(coming soon)") {
                        self.status_message = Some("Coming in a future release".into());
                    } else {
                        self.toggle_expand_resource();
                    }
                }
            }
            KeyCode::Char('*') => self.expand_all_under_current(),
            KeyCode::Enter => {
                if let Some(item) = self.resources.items.get(self.resources.selected) {
                    if item.name.contains("(coming soon)") {
                        self.status_message = Some("Coming in a future release".into());
                    } else if item.expandable {
                        self.toggle_expand_resource();
                    } else {
                        self.load_selected_details().await;
                    }
                }
            }
            KeyCode::Char('b' | 'B') => {
                let build_name = self.resources.items.get(self.resources.selected)
                    .filter(|item| item.resource_type == ResourceType::Container && !item.expandable)
                    .and_then(|item| item.id.strip_prefix("container:").map(|s| s.to_string()));
                if let Some(name) = build_name {
                    self.trigger_build(&name).await;
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Handle details pane navigation
    async fn handle_details_keys(&mut self, key: KeyEvent) -> color_eyre::Result<()> {
        // Build trigger when viewing container details
        if let Some(ResourceDetails::Container(ref manifest)) = self.selected_details {
            if key.code == KeyCode::Char('b') || key.code == KeyCode::Char('B') {
                let name = manifest.name.clone();
                self.trigger_build(&name).await;
                return Ok(());
            }
        }
        Ok(())
    }

    /// Handle logs pane navigation
    async fn handle_logs_keys(&mut self, _key: KeyEvent) -> color_eyre::Result<()> {
        Ok(())
    }

    /// Handle application events
    async fn handle_app_event(&mut self, event: AppEvent) {
        match event {
            AppEvent::Quit => self.quit(),
            AppEvent::RefreshData => self.refresh_all().await,
            AppEvent::LoadComplete(resource_type) => self.handle_load_complete(resource_type),
            // Build events
            AppEvent::BuildQueued { container, position } => {
                self.status_message =
                    Some(format!("Build queued: {} (position {})", container, position));
            }
            AppEvent::BuildStarted {
                container,
                build_id: _,
            } => {
                self.status_message = Some(format!("Building: {}", container));
            }
            AppEvent::BuildPhaseChanged { container, phase } => {
                self.status_message = Some(format!("{}: {}", container, phase));
            }
            AppEvent::BuildLogLine { container: _, line } => {
                self.logs.push(line);
                if self.logs.len() > 1000 {
                    self.logs.drain(0..100);
                }
            }
            AppEvent::DependenciesResolved { container, deps } => {
                self.status_message = Some(format!(
                    "{}: deps resolved ({})",
                    container,
                    deps.join(", ")
                ));
            }
            AppEvent::BuildComplete {
                container,
                success,
                message,
                duration_secs,
            } => {
                self.status_message = Some(format!(
                    "{}: {} in {}s - {}",
                    container,
                    if success { "SUCCESS" } else { "FAILED" },
                    duration_secs,
                    message,
                ));
                self.refresh_all().await;
            }
            AppEvent::BuildCancelled { container, reason } => {
                self.status_message =
                    Some(format!("{}: cancelled - {}", container, reason));
            }
            AppEvent::ServiceStateChanged { service, running } => {
                self.status_message = Some(format!(
                    "{}: {}",
                    service,
                    if running { "started" } else { "stopped" },
                ));
                self.refresh_all().await;
            }
        }
    }

    /// Periodic tick
    pub async fn tick(&self) {}

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
            self.check_and_clear_details();
        }
    }

    /// Select next resource
    fn select_next_resource(&mut self) {
        if self.resources.selected < self.resources.items.len().saturating_sub(1) {
            self.resources.selected += 1;
            self.check_and_clear_details();
        }
    }

    /// Clear details if current selection is expandable
    fn check_and_clear_details(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            if item.expandable {
                self.selected_details = None;
            }
        }
    }

    /// Toggle expand/collapse for current resource
    fn toggle_expand_resource(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            if item.expandable {
                let id = item.id.clone();
                let name = item.name.clone();

                self.logs.push(format!(
                    "[{}] Toggle expand/collapse for '{}'",
                    chrono::Local::now().format("%H:%M:%S"),
                    name
                ));

                let is_expanded = self.resources.expanded.contains(&id);

                if is_expanded {
                    self.resources.expanded.retain(|x| x != &id);
                    self.status_message = Some(format!("Collapsed {}", name));
                    self.collapse_resource(&id);
                } else {
                    self.resources.expanded.push(id.clone());
                    self.status_message = Some(format!("Expanding {}", name));
                    self.events.send(AppEvent::RefreshData);
                }

                if self.logs.len() > 100 {
                    self.logs.drain(0..self.logs.len() - 100);
                }
            }
        }
    }

    /// Collapse a resource by removing its children from the tree
    fn collapse_resource(&mut self, parent_id: &str) {
        if let Some(parent_idx) = self
            .resources
            .items
            .iter()
            .position(|item| item.id == parent_id)
        {
            let parent_depth = self.resources.items[parent_idx].depth;
            let mut remove_count = 0;
            let mut i = parent_idx + 1;

            while i < self.resources.items.len() {
                if self.resources.items[i].depth <= parent_depth {
                    break;
                }
                remove_count += 1;
                i += 1;
            }

            for _ in 0..remove_count {
                self.resources.items.remove(parent_idx + 1);
                if self.resources.selected > parent_idx {
                    self.resources.selected = self.resources.selected.saturating_sub(1);
                }
            }
        }
    }

    /// Expand all nodes under the current selection
    fn expand_all_under_current(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            let parent_id = item.id.clone();
            let parent_depth = item.depth;

            if item.expandable && !self.resources.expanded.contains(&parent_id) {
                self.resources.expanded.push(parent_id.clone());
            }

            let mut expanded_count = 0;
            for i in (self.resources.selected + 1)..self.resources.items.len() {
                let child = &self.resources.items[i];
                if child.depth <= parent_depth {
                    break;
                }
                if child.expandable && !self.resources.expanded.contains(&child.id) {
                    self.resources.expanded.push(child.id.clone());
                    expanded_count += 1;
                }
            }

            self.status_message = Some(format!(
                "Expanded {} items under {}",
                expanded_count + 1,
                item.name
            ));
            self.events.send(AppEvent::RefreshData);
        }
    }

    /// Load resources from backend
    async fn load_resources(&mut self) {
        self.loading.resources = true;

        let selected_id = self
            .resources
            .items
            .get(self.resources.selected)
            .map(|item| item.id.clone());

        let mut items = vec![];

        // ── Containers (REAL data from bootstrap) ──
        items.push(ResourceItem {
            id: "containers".into(),
            name: "containers".into(),
            resource_type: ResourceType::Container,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
            secondary_status: None,
        });

        if self
            .resources
            .expanded
            .contains(&"containers".to_string())
        {
            for manifest in &self.bootstrap.available_containers {
                let is_built = self.bootstrap.built_images.contains_key(&manifest.name);
                let run_state = self.bootstrap.running_containers.get(&manifest.name).cloned();
                items.push(ResourceItem {
                    id: format!("container:{}", manifest.name),
                    name: manifest.name.clone(),
                    resource_type: ResourceType::Container,
                    parent: Some("containers".into()),
                    expandable: false,
                    depth: 1,
                    status: Some(
                        if is_built {
                            "Built".into()
                        } else {
                            "Not Built".into()
                        },
                    ),
                    secondary_status: run_state,
                });
            }
        }

        // ── Builds ──
        items.push(ResourceItem {
            id: "builds".into(),
            name: "builds".into(),
            resource_type: ResourceType::Build,
            parent: None,
            expandable: true,
            depth: 0,
            status: None,
            secondary_status: None,
        });

        // ── Coming Soon sections ──
        for (id, name) in [
            ("deployments", "deployments"),
            ("templates", "templates"),
            ("nodes", "nodes"),
            ("secrets", "secrets"),
        ] {
            items.push(ResourceItem {
                id: id.into(),
                name: format!("{} (coming soon)", name),
                resource_type: ResourceType::Deployment,
                parent: None,
                expandable: false,
                depth: 0,
                status: None,
                secondary_status: None,
            });
        }

        self.resources.items = items;

        // Restore selection if possible
        if let Some(id) = selected_id {
            if let Some(pos) = self.resources.items.iter().position(|item| item.id == id) {
                self.resources.selected = pos;
            }
        }

        self.check_and_clear_details();
        self.loading.resources = false;
    }

    /// Load details for selected resource
    async fn load_selected_details(&mut self) {
        if let Some(item) = self.resources.items.get(self.resources.selected) {
            if !item.expandable {
                self.loading.details = true;

                if let Some((resource_type, id)) = item.id.split_once(':') {
                    match resource_type {
                        "container" => {
                            if let Some(manifest) = self
                                .bootstrap
                                .available_containers
                                .iter()
                                .find(|m| m.name == id)
                            {
                                self.selected_details =
                                    Some(ResourceDetails::Container(manifest.clone()));
                                self.focused_pane = Pane::Details;
                            }
                        }
                        "deployment" => {
                            if let Ok(details) = self.backend.get_deployment(id).await {
                                self.selected_details =
                                    Some(ResourceDetails::Deployment(details));
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
    }

    /// Trigger a container build
    async fn trigger_build(&mut self, container_name: &str) {
        self.status_message = Some(format!("Starting build: {}", container_name));
        self.logs.push(format!(
            "[{}] Build requested: '{}'",
            chrono::Local::now().format("%H:%M:%S"),
            container_name
        ));

        match self.backend.start_build(container_name).await {
            Ok(build_status) => {
                self.status_message =
                    Some(format!("Build queued: {}", container_name));
                self.selected_details = Some(ResourceDetails::Build(build_status));
                self.focused_pane = Pane::Logs;
            }
            Err(e) => {
                self.status_message = Some(format!("Build failed: {}", e));
                self.logs.push(format!(
                    "[{}] Build error: {}",
                    chrono::Local::now().format("%H:%M:%S"),
                    e
                ));
            }
        }
    }

    async fn refresh_all(&mut self) {
        if let Some(msg) = &self.status_message {
            if msg.starts_with("Expanding") {
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
}

// Helper trait for shared event sender
impl App {
    pub fn events(&self) -> &EventHandler {
        &self.events
    }
}
