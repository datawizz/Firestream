use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

use crate::app::App;
use crate::views::{
    ResourcesPane, DetailsPane, LogsPane, HelpView, CommandPalette, SearchView, SplashView, View
};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Status bar at top
                Constraint::Min(0),     // Main content
            ])
            .split(area);

        // Draw main content based on current view
        match &self.current_view {
            View::Splash => {
                // Splash view takes full area, no status bar
                SplashView::new().render(area, buf);
                return; // Skip status bar for splash
            }
            View::Main => self.render_main_view(chunks[1], buf),
            View::Help => HelpView.render(chunks[1], buf),
            View::NewDeployment => self.render_new_deployment_view(chunks[1], buf),
            View::Search(_) => SearchView::new(self).render(chunks[1], buf),
            View::CommandPalette => {
                // Render main view underneath
                self.render_main_view(chunks[1], buf);
                // Then render command palette on top
                CommandPalette::new(self).render(chunks[1], buf);
            }
        }

        // Always draw status bar at top
        self.render_status_bar(chunks[0], buf);

        // Draw overlays
        if self.command_mode {
            CommandPalette::new(self).render(chunks[1], buf);
        } else if self.search_mode {
            SearchView::new(self).render(chunks[1], buf);
        }
    }
}

impl App {
    fn render_main_view(&self, area: Rect, buf: &mut Buffer) {
        // Three-pane layout
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30),  // Resources pane
                Constraint::Min(0),      // Details pane
            ])
            .split(area);

        // Resources pane (left)
        ResourcesPane::new(self).render(main_chunks[0], buf);

        // Split right side into details and logs
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70),  // Details pane
                Constraint::Percentage(30),  // Logs pane
            ])
            .split(main_chunks[1]);

        // Details pane (top right)
        DetailsPane::new(self).render(right_chunks[0], buf);

        // Logs pane (bottom right)
        LogsPane::new(self).render(right_chunks[1], buf);
    }

    fn render_new_deployment_view(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" New Deployment ")
            .title_alignment(Alignment::Center);

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),   // Template selection
                Constraint::Length(3),   // Name input
                Constraint::Length(1),   // Spacer
                Constraint::Length(3),   // Configuration header
                Constraint::Length(3),   // Replicas
                Constraint::Length(3),   // CPU
                Constraint::Length(3),   // Memory
                Constraint::Length(3),   // GPU
                Constraint::Length(1),   // Spacer
                Constraint::Length(3),   // Input configuration header
                Constraint::Length(3),   // Input topic
                Constraint::Length(3),   // Output table
                Constraint::Length(3),   // Batch size
                Constraint::Length(3),   // Window
                Constraint::Min(0),      // Spacer
                Constraint::Length(3),   // Actions
            ])
            .split(block.inner(area));

        block.render(area, buf);

        // Template field
        let template_block = Block::default()
            .borders(Borders::ALL)
            .title("Template");
        let template_text = vec![Line::from("pyspark/stream-processor")];
        Paragraph::new(template_text).block(template_block).render(chunks[0], buf);

        // Name field
        let name_block = Block::default()
            .borders(Borders::ALL)
            .title("Name");
        let name_text = vec![Line::from("stream-proc-prod")];
        Paragraph::new(name_text).block(name_block).render(chunks[1], buf);

        // Configuration section
        let config_text = vec![Line::from(Span::styled(
            "Configuration",
            Style::default().add_modifier(Modifier::BOLD),
        ))];
        Paragraph::new(config_text).render(chunks[3], buf);

        // Replicas
        self.render_field("Replicas", "3", chunks[4], buf);
        
        // CPU
        self.render_field("CPU", "4 cores", chunks[5], buf);
        
        // Memory
        self.render_field("Memory", "8 GB", chunks[6], buf);
        
        // GPU
        self.render_field("GPU", "[ ] enabled", chunks[7], buf);

        // Input Configuration section
        let input_config_text = vec![Line::from(Span::styled(
            "Input Configuration",
            Style::default().add_modifier(Modifier::BOLD),
        ))];
        Paragraph::new(input_config_text).render(chunks[9], buf);

        // Input fields
        self.render_field("Input Topic", "raw-events", chunks[10], buf);
        self.render_field("Output Table", "processed", chunks[11], buf);
        self.render_field("Batch Size", "1000 messages", chunks[12], buf);
        self.render_field("Window", "5 minutes", chunks[13], buf);

        // Actions
        let actions = vec![Line::from(vec![
            Span::styled("[Validate]", Style::default().fg(Color::Yellow)),
            Span::raw("  "),
            Span::styled("[Deploy]", Style::default().fg(Color::Green)),
            Span::raw("  "),
            Span::styled("[Cancel]", Style::default().fg(Color::Red)),
        ])];
        Paragraph::new(actions)
            .alignment(Alignment::Center)
            .render(chunks[15], buf);
    }

    fn render_field(&self, label: &str, value: &str, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(20),
                Constraint::Min(0),
            ])
            .split(area);

        let label_text = vec![Line::from(format!("{}:", label))];
        Paragraph::new(label_text).render(chunks[0], buf);

        let value_block = Block::default()
            .borders(Borders::ALL);
        let value_text = vec![Line::from(value)];
        Paragraph::new(value_text).block(value_block).render(chunks[1], buf);
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        // Create block with borders
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 80)))
            .title(" Firestream ")
            .title_alignment(Alignment::Left)
            .title_style(Style::default()
                .fg(Color::Rgb(239, 200, 131))
                .add_modifier(Modifier::BOLD));
            
        // Calculate inner area before rendering
        let inner = block.inner(area);
        
        // Render the block
        block.render(area, buf);
        
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(10),  // Version
                Constraint::Length(15),  // Environment
                Constraint::Length(15),  // Status
                Constraint::Length(15),  // Uptime
                Constraint::Min(0),      // Resource usage
                Constraint::Length(30),  // Key hints
            ])
            .split(inner);

        // Version info
        let version_info = Line::from(vec![
            Span::styled(format!("v{}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &version_info, chunks[0].width);

        // Environment
        let env_info = Line::from(vec![
            Span::raw("| "),
            Span::raw("local-k3d"),
        ]);
        buf.set_line(chunks[1].x, chunks[1].y, &env_info, chunks[1].width);

        // Connection status
        let status_info = Line::from(vec![
            Span::raw("| "),
            Span::styled("● connected", Style::default().fg(Color::Green)),
        ]);
        buf.set_line(chunks[2].x, chunks[2].y, &status_info, chunks[2].width);

        // Uptime
        let uptime_info = Line::from(vec![
            Span::raw("| ↑ "),
            Span::raw("15d 3h"),
        ]);
        buf.set_line(chunks[3].x, chunks[3].y, &uptime_info, chunks[3].width);

        // Resource usage
        let resource_info = Line::from(vec![
            Span::raw("| cpu: "),
            Span::styled("42%", Style::default().fg(Color::Yellow)),
            Span::raw(" mem: "),
            Span::styled("71%", Style::default().fg(Color::Yellow)),
        ]);
        buf.set_line(chunks[4].x, chunks[4].y, &resource_info, chunks[4].width);

        // Status message (if any)
        if let Some(msg) = &self.status_message {
            let status = Line::from(vec![
                Span::raw("| "),
                Span::styled(msg, Style::default().fg(Color::Rgb(239, 200, 131))),
            ]);
            buf.set_line(chunks[5].x, chunks[5].y, &status, chunks[5].width);
        }
    }
}
