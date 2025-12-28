use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Paragraph, Widget},
};

pub struct SplashView {
    pub version: String,
    pub working_dir: String,
    pub config_dir: String,
}

impl SplashView {
    pub fn new() -> Self {
        let working_dir = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "~/firestream".to_string());
        
        let config_dir = dirs::config_dir()
            .map(|p| p.join("firestream").display().to_string())
            .unwrap_or_else(|| "~/.config/firestream".to_string());

        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            working_dir,
            config_dir,
        }
    }

    fn render_banner(&self, area: Rect, buf: &mut Buffer) {
        let banner_lines = vec![
            "███████ ██ ██████  ███████ ███████ ████████ ██████  ███████  █████  ███    ███",
            "██      ██ ██   ██ ██      ██         ██    ██   ██ ██      ██   ██ ████  ████",
            "█████   ██ ██████  █████   ███████    ██    ██████  █████   ███████ ██ ████ ██",
            "██      ██ ██   ██ ██           ██    ██    ██   ██ ██      ██   ██ ██  ██  ██",
            "██      ██ ██   ██ ███████ ███████    ██    ██   ██ ███████ ██   ██ ██      ██",
        ];

        // Define fire colors (warm golden) and stream colors (light cyan)
        let fire_color = Color::Rgb(239, 200, 131);    // Warm golden
        let fire_dim = Color::Rgb(180, 150, 100);      // Darker golden
        let stream_color = Color::Rgb(157, 230, 242);  // Light cyan
        let stream_dim = Color::Rgb(100, 170, 180);    // Darker cyan
        
        let styled_lines: Vec<Line> = banner_lines
            .into_iter()
            .map(|line| {
                let chars: Vec<Span> = line
                    .chars()
                    .enumerate()
                    .map(|(j, ch)| {
                        // "FIRE" ends at position 25, "STREAM" starts at position 27
                        let is_fire_section = j < 27;
                        
                        let color = if ch == '█' {
                            if is_fire_section {
                                fire_color
                            } else {
                                stream_color
                            }
                        } else if ch != ' ' {
                            if is_fire_section {
                                fire_dim
                            } else {
                                stream_dim
                            }
                        } else {
                            Color::Reset
                        };
                        
                        Span::styled(ch.to_string(), Style::default().fg(color).add_modifier(Modifier::BOLD))
                    })
                    .collect();
                Line::from(chars)
            })
            .collect();

        let banner = Paragraph::new(styled_lines)
            .alignment(Alignment::Center);

        banner.render(area, buf);
    }

    fn render_info(&self, area: Rect, buf: &mut Buffer) {
        let info_lines = vec![
            Line::from(vec![
                Span::styled("v", Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled(&self.version, Style::default().fg(Color::Rgb(239, 200, 131))),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("cwd ", Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled(&self.working_dir, Style::default().fg(Color::Rgb(150, 150, 170))),
            ]),
            Line::from(vec![
                Span::styled("config ", Style::default().fg(Color::Rgb(100, 100, 120))),
                Span::styled(&self.config_dir, Style::default().fg(Color::Rgb(150, 150, 170))),
            ]),
        ];

        let info = Paragraph::new(info_lines)
            .alignment(Alignment::Center);

        info.render(area, buf);
    }

    fn render_controls(&self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),  // Controls header
                Constraint::Length(1),  // Spacing
                Constraint::Length(1),  // Navigation section
                Constraint::Length(1),  // Up/Down
                Constraint::Length(1),  // Left/Right resources
                Constraint::Length(1),  // Left/Right other
                Constraint::Length(1),  // Enter
                Constraint::Length(1),  // Tab
                Constraint::Length(1),  // Spacing
                Constraint::Length(1),  // General section
                Constraint::Length(1),  // Ctrl+C
                Constraint::Length(2),  // Spacing
                Constraint::Length(1),  // Press any key
            ])
            .split(area);

        // Controls header
        let controls_header = Line::from(vec![
            Span::styled("Controls", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
        ]);
        Paragraph::new(vec![controls_header])
            .alignment(Alignment::Center)
            .render(chunks[0], buf);

        // Navigation section
        let nav_header = Line::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Rgb(239, 200, 131))),
        ]);
        Paragraph::new(vec![nav_header])
            .alignment(Alignment::Center)
            .render(chunks[2], buf);

        // Up/Down
        let up_down = Line::from(vec![
            Span::styled("↑↓", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
            Span::styled("  Navigate items", Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        Paragraph::new(vec![up_down])
            .alignment(Alignment::Center)
            .render(chunks[3], buf);

        // Left/Right resources
        let left_right_resources = Line::from(vec![
            Span::styled("←→", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
            Span::styled("  Expand/collapse (Resources pane)", Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        Paragraph::new(vec![left_right_resources])
            .alignment(Alignment::Center)
            .render(chunks[4], buf);

        // Left/Right other
        let left_right_other = Line::from(vec![
            Span::styled("←→", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
            Span::styled("  Switch panes (Details/Logs)", Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        Paragraph::new(vec![left_right_other])
            .alignment(Alignment::Center)
            .render(chunks[5], buf);

        // Enter
        let enter = Line::from(vec![
            Span::styled("Enter", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
            Span::styled("  Select/activate item", Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        Paragraph::new(vec![enter])
            .alignment(Alignment::Center)
            .render(chunks[6], buf);

        // Tab
        let tab = Line::from(vec![
            Span::styled("Tab/Shift+Tab", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
            Span::styled("  Navigate between panes", Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        Paragraph::new(vec![tab])
            .alignment(Alignment::Center)
            .render(chunks[7], buf);

        // General section
        let general_header = Line::from(vec![
            Span::styled("General", Style::default().fg(Color::Rgb(239, 200, 131))),
        ]);
        Paragraph::new(vec![general_header])
            .alignment(Alignment::Center)
            .render(chunks[9], buf);

        // Ctrl+C
        let ctrl_c = Line::from(vec![
            Span::styled("Ctrl+C", Style::default().fg(Color::Rgb(157, 230, 242)).add_modifier(Modifier::BOLD)),
            Span::styled("  Quit application", Style::default().fg(Color::Rgb(150, 150, 170))),
        ]);
        Paragraph::new(vec![ctrl_c])
            .alignment(Alignment::Center)
            .render(chunks[10], buf);

        // Press any key
        let continue_hint = Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Rgb(100, 100, 120))),
            Span::styled("any key", Style::default().fg(Color::Rgb(239, 200, 131)).add_modifier(Modifier::BOLD)),
            Span::styled(" to continue...", Style::default().fg(Color::Rgb(100, 100, 120))),
        ]);
        Paragraph::new(vec![continue_hint])
            .alignment(Alignment::Center)
            .render(chunks[12], buf);
    }


}

impl Widget for SplashView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Clear the area with a dark background matching the banner
        let bg_block = Block::default()
            .style(Style::default().bg(Color::Rgb(10, 10, 20)));
        bg_block.render(area, buf);

        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(15),  // Top spacing
                Constraint::Length(5),       // Banner
                Constraint::Length(2),       // Spacing
                Constraint::Length(4),       // Version and paths
                Constraint::Length(3),       // Spacing
                Constraint::Length(14),      // Controls section
                Constraint::Min(0),          // Flexible space
                Constraint::Percentage(5),   // Bottom spacing
            ])
            .margin(2)
            .split(area);

        // Render banner
        self.render_banner(chunks[1], buf);

        // Render info
        self.render_info(chunks[3], buf);

        // Render controls section
        self.render_controls(chunks[5], buf);
    }
}
