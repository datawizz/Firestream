use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Widget},
};

use crate::app::App;
use crate::views::{
    DetailsPane, HelpView, LogsPane, ResourcesPane, SplashView, View,
};

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Draw main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Status bar at top
                Constraint::Min(0),   // Main content
            ])
            .split(area);

        // Draw main content based on current view
        match &self.current_view {
            View::Splash => {
                SplashView::new().render(area, buf);
                return; // Skip status bar for splash
            }
            View::Main => self.render_main_view(chunks[1], buf),
            View::Help => HelpView.render(chunks[1], buf),
            _ => self.render_main_view(chunks[1], buf),
        }

        // Always draw status bar at top
        self.render_status_bar(chunks[0], buf);
    }
}

impl App {
    fn render_main_view(&self, area: Rect, buf: &mut Buffer) {
        // Three-pane layout
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(30), // Resources pane
                Constraint::Min(0),    // Details pane
            ])
            .split(area);

        // Resources pane (left)
        ResourcesPane::new(self).render(main_chunks[0], buf);

        // Split right side into details and logs
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(70), // Details pane
                Constraint::Percentage(30), // Logs pane
            ])
            .split(main_chunks[1]);

        // Details pane (top right)
        DetailsPane::new(self).render(right_chunks[0], buf);

        // Logs pane (bottom right)
        LogsPane::new(self).render(right_chunks[1], buf);
    }

    fn render_status_bar(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Rgb(60, 60, 80)))
            .title(" Firestream ")
            .title_alignment(Alignment::Left)
            .title_style(
                Style::default()
                    .fg(Color::Rgb(239, 200, 131))
                    .add_modifier(Modifier::BOLD),
            );

        let inner = block.inner(area);
        block.render(area, buf);

        // Two sections: left for system info, right for context hints
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60), // System info
                Constraint::Percentage(40), // Context hints
            ])
            .split(inner);

        // Left section - real system info from bootstrap
        let mut sys_spans: Vec<Span> = vec![
            Span::styled(
                format!("v{}", env!("CARGO_PKG_VERSION")),
                Style::default().fg(Color::Rgb(150, 150, 170)),
            ),
        ];

        if self.bootstrap.docker_available {
            let docker_ver = self
                .bootstrap
                .docker_version
                .as_deref()
                .unwrap_or("unknown");
            let health_color = if self.bootstrap.docker_healthy {
                Color::Green
            } else {
                Color::Yellow
            };
            sys_spans.push(Span::raw(" | Docker "));
            sys_spans.push(Span::raw(docker_ver));
            sys_spans.push(Span::raw(" "));
            sys_spans.push(Span::styled("●", Style::default().fg(health_color)));

            // Image count
            let built = self.bootstrap.built_images.len();
            let total = self.bootstrap.available_containers.len();
            sys_spans.push(Span::raw(format!(" | {}/{} built", built, total)));

            // Running count
            let running = self.bootstrap.running_containers.values()
                .filter(|s| s.as_str() == "running")
                .count();
            if running > 0 {
                sys_spans.push(Span::raw(" | "));
                sys_spans.push(Span::styled(
                    format!("{} running", running),
                    Style::default().fg(Color::Green),
                ));
            }

            // Nix cache
            let cache_status = if self.bootstrap.nix_volume_exists {
                "ready"
            } else {
                "cold"
            };
            sys_spans.push(Span::raw(format!(" | Nix: {}", cache_status)));
        } else {
            sys_spans.push(Span::raw(" | "));
            sys_spans.push(Span::styled(
                "Demo Mode (no Docker)",
                Style::default().fg(Color::Yellow),
            ));
        }

        let sys_line = Line::from(sys_spans);
        buf.set_line(
            main_chunks[0].x,
            main_chunks[0].y,
            &sys_line,
            main_chunks[0].width,
        );

        // Right section - context hints or status message
        if let Some(msg) = &self.status_message {
            let status = Line::from(vec![
                Span::raw("│ "),
                Span::styled(msg, Style::default().fg(Color::Rgb(239, 200, 131))),
            ]);
            buf.set_line(
                main_chunks[1].x,
                main_chunks[1].y,
                &status,
                main_chunks[1].width,
            );
        } else {
            let hints = match self.focused_pane {
                crate::app::Pane::Resources => {
                    vec![
                        Span::raw("│ "),
                        Span::styled("●", Style::default().fg(Color::Green)),
                        Span::raw("built "),
                        Span::styled("○", Style::default().fg(Color::DarkGray)),
                        Span::raw("not "),
                        Span::styled("▶", Style::default().fg(Color::Green)),
                        Span::raw("up "),
                        Span::styled("■", Style::default().fg(Color::DarkGray)),
                        Span::raw("down │ "),
                        Span::styled("B", Style::default().fg(Color::Cyan)),
                        Span::raw(": build │ "),
                        Span::styled("^C", Style::default().fg(Color::Cyan)),
                        Span::raw(": quit"),
                    ]
                }
                crate::app::Pane::Details => {
                    vec![
                        Span::raw("│ "),
                        Span::styled("B", Style::default().fg(Color::Cyan)),
                        Span::raw(": build │ "),
                        Span::styled("Tab", Style::default().fg(Color::Cyan)),
                        Span::raw(": next │ "),
                        Span::styled("^C", Style::default().fg(Color::Cyan)),
                        Span::raw(": quit"),
                    ]
                }
                crate::app::Pane::Logs => {
                    vec![
                        Span::raw("│ "),
                        Span::styled("Tab", Style::default().fg(Color::Cyan)),
                        Span::raw(": next │ "),
                        Span::styled("^C", Style::default().fg(Color::Cyan)),
                        Span::raw(": quit"),
                    ]
                }
            };
            let hint_line = Line::from(hints);
            buf.set_line(
                main_chunks[1].x,
                main_chunks[1].y,
                &hint_line,
                main_chunks[1].width,
            );
        }
    }
}
