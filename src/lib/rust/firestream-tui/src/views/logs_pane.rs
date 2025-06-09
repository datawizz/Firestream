use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Widget},
};
use crate::app::{App, Pane};

pub struct LogsPane<'a> {
    app: &'a App,
}

impl<'a> LogsPane<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for LogsPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.app.focused_pane == Pane::Logs;
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Logs ")
            .border_style(if is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        if self.app.logs.is_empty() {
            let items = vec![
                ListItem::new(Line::from("No logs available")),
                ListItem::new(Line::from("")),
                ListItem::new(Line::from("Select a deployment and press 'l' to view logs")),
            ];
            
            let list = List::new(items).block(block);
            list.render(area, buf);
        } else {
            // Show logs with timestamps
            let items: Vec<ListItem> = self.app.logs.iter().map(|log| {
                ListItem::new(Line::from(log.as_str()))
            }).collect();
            
            // Always show the most recent logs (scroll to bottom)
            let visible_height = area.height.saturating_sub(2) as usize; // Account for borders
            let skip = if items.len() > visible_height {
                items.len() - visible_height
            } else {
                0
            };
            
            let visible_items: Vec<ListItem> = items.into_iter().skip(skip).collect();
            let list = List::new(visible_items).block(block);
            list.render(area, buf);
        }
        
        // Show control hints when focused
        if is_focused && area.height > 2 {
            let hint = if self.app.logs.is_empty() {
                "[l] load logs"
            } else {
                "[f] follow  [ctrl+c] stop"
            };
            
            let hint_line = Line::from(hint).style(Style::default().fg(Color::DarkGray));
            buf.set_line(area.x + 2, area.y + area.height - 2, &hint_line, area.width - 4);
        }
    }
}
