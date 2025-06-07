use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};
use crate::app::App;

pub struct CommandPalette<'a> {
    app: &'a App,
}

impl<'a> CommandPalette<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for CommandPalette<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a centered popup
        let popup_width = 60.min(area.width - 4);
        let popup_height = 20.min(area.height - 4);
        
        let popup_area = Rect {
            x: area.x + (area.width - popup_width) / 2,
            y: area.y + (area.height - popup_height) / 2,
            width: popup_width,
            height: popup_height,
        };
        
        // Clear the background
        Clear.render(popup_area, buf);
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Command Palette ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan));
        
        let inner_area = block.inner(popup_area);
        block.render(popup_area, buf);
        
        // Split into input and suggestions
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner_area);
        
        // Render input field
        let input_block = Block::default()
            .borders(Borders::ALL);
        
        let input_text = vec![
            Line::from(vec![
                Span::raw("> "),
                Span::raw(&self.app.command_input),
                Span::styled("█", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]),
        ];
        
        let input = Paragraph::new(input_text)
            .block(input_block);
        
        input.render(chunks[0], buf);
        
        // Render suggestions based on input
        let suggestions = get_command_suggestions(&self.app.command_input);
        
        let items: Vec<ListItem> = suggestions.into_iter().map(|(cmd, desc)| {
            ListItem::new(Line::from(vec![
                Span::styled(cmd, Style::default().fg(Color::Yellow)),
                Span::raw("  "),
                Span::styled(desc, Style::default().fg(Color::DarkGray)),
            ]))
        }).collect();
        
        let list = List::new(items)
            .highlight_style(Style::default().bg(Color::DarkGray));
        
        list.render(chunks[1], buf);
        
        // Show help at bottom
        let help = Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" navigate  "),
            Span::styled("[enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" execute  "),
            Span::styled("[esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" cancel"),
        ]);
        
        buf.set_line(
            popup_area.x + 2,
            popup_area.y + popup_area.height - 1,
            &help,
            popup_area.width - 4,
        );
    }
}

fn get_command_suggestions(input: &str) -> Vec<(&'static str, &'static str)> {
    let all_commands = vec![
        ("deploy template", "Deploy new application"),
        ("deploy etl-pipeline", "Deploy specific template"),
        ("deployments list", "Show all deployments"),
        ("deployment logs", "View deployment logs"),
        ("deployment scale", "Scale deployment replicas"),
        ("build start", "Start a new build"),
        ("build status", "Check build status"),
        ("node provision", "Provision new node"),
        ("node list", "List all nodes"),
        ("secret create", "Create new secret"),
        ("secret list", "List all secrets"),
        ("data tables", "List Delta tables"),
        ("data branches", "List LakeFS branches"),
    ];
    
    if input.is_empty() {
        all_commands
    } else {
        all_commands.into_iter()
            .filter(|(cmd, _)| cmd.starts_with(input))
            .collect()
    }
}
