use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Widget},
};
use crate::app::{App, Pane};

pub struct ResourcesPane<'a> {
    app: &'a App,
}

impl<'a> ResourcesPane<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for ResourcesPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.app.focused_pane == Pane::Resources;
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Resources ")
            .border_style(if is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        // Build list items
        let items: Vec<ListItem> = self.app.resources.items.iter().enumerate().map(|(idx, item)| {
            let is_selected = idx == self.app.resources.selected;
            
            // Build indentation based on depth
            let indent = "  ".repeat(item.depth);
            
            // Build prefix based on expandable state
            let prefix = if item.expandable {
                if self.app.resources.expanded.contains(&item.id) {
                    "▼ "
                } else {
                    "▶ "
                }
            } else if item.depth > 0 {
                if is_selected && is_focused {
                    "› "
                } else {
                    "  "
                }
            } else {
                ""
            };
            
            // Build status indicator
            let status_indicator = if let Some(status) = &item.status {
                match status.as_str() {
                    "Running" => " ●",
                    "Pending" => " ○",
                    "Failed" => " ■",
                    "Ready" => " ●",
                    _ => "",
                }
            } else {
                ""
            };
            
            let status_color = if let Some(status) = &item.status {
                match status.as_str() {
                    "Running" | "Ready" => Color::Green,
                    "Pending" | "Provisioning" => Color::Yellow,
                    "Failed" | "Error" => Color::Red,
                    _ => Color::White,
                }
            } else {
                Color::White
            };
            
            let mut spans = vec![
                Span::raw(indent),
                Span::raw(prefix),
                Span::raw(&item.name),
            ];
            
            if !status_indicator.is_empty() {
                spans.push(Span::styled(status_indicator, Style::default().fg(status_color)));
            }
            
            let style = if is_selected && is_focused {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default().add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(spans)).style(style)
        }).collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(Style::default());

        list.render(area, buf);

        // Render action hints at the bottom
        if is_focused && area.height > 4 {
            let actions_y = area.y + area.height - 2;
            let actions = Line::from(vec![
                Span::styled("[n]", Style::default().fg(Color::Yellow)),
                Span::raw(" new "),
                Span::styled("[d]", Style::default().fg(Color::Yellow)),
                Span::raw(" deploy "),
                Span::styled("[b]", Style::default().fg(Color::Yellow)),
                Span::raw(" build "),
                Span::styled("[/]", Style::default().fg(Color::Yellow)),
                Span::raw(" search"),
            ]);
            buf.set_line(area.x + 1, actions_y, &actions, area.width - 2);
        }
    }
}
