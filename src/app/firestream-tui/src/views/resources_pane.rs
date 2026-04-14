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
    
    /// Get the ancestry path for proper tree line drawing
    fn get_ancestry_continues(&self, idx: usize) -> Vec<bool> {
        let items = &self.app.resources.items;
        if idx >= items.len() {
            return vec![];
        }
        
        let item = &items[idx];
        let mut continues = vec![false; item.depth];
        
        // For each depth level, check if there's a continuation
        for depth in 0..item.depth {
            // Find the last item at this depth before our index
            let mut last_at_depth_idx = None;
            for i in (0..idx).rev() {
                if items[i].depth == depth {
                    last_at_depth_idx = Some(i);
                    break;
                }
            }
            
            if let Some(last_idx) = last_at_depth_idx {
                // Check if this item has siblings after it
                for i in (last_idx + 1)..items.len() {
                    if items[i].depth < depth {
                        break;
                    }
                    if items[i].depth == depth {
                        continues[depth] = true;
                        break;
                    }
                }
            }
        }
        
        continues
    }
    
    /// Check if this is the last item at its depth level with the same parent
    fn is_last_sibling(&self, idx: usize) -> bool {
        let items = &self.app.resources.items;
        if idx >= items.len() {
            return true;
        }
        
        let item = &items[idx];
        
        // Look for any sibling after this item
        for i in (idx + 1)..items.len() {
            let next = &items[i];
            
            // Stop if we've gone up to a parent level
            if next.depth < item.depth {
                return true;
            }
            
            // Found a sibling
            if next.depth == item.depth && next.parent == item.parent {
                return false;
            }
        }
        
        true
    }
    
    /// Get count of direct children for an item
    fn get_child_count(&self, item_id: &str) -> usize {
        self.app.resources.items.iter()
            .filter(|item| item.parent.as_ref() == Some(&item_id.to_string()))
            .count()
    }
}

impl<'a> Widget for ResourcesPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.app.focused_pane == Pane::Resources;
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title("╼ Resources ╾")
            .border_style(if is_focused {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            });

        // Build list items
        let items: Vec<ListItem> = self.app.resources.items.iter().enumerate().map(|(idx, item)| {
            let is_selected = idx == self.app.resources.selected;
            let is_expanded = self.app.resources.expanded.contains(&item.id);
            
            let mut spans = vec![];
            
            // Build tree structure
            if item.depth > 0 {
                let continues = self.get_ancestry_continues(idx);
                let is_last = self.is_last_sibling(idx);
                
                // Draw vertical lines for parent levels
                for (depth, &has_continuation) in continues.iter().enumerate() {
                    if depth < item.depth - 1 {
                        if has_continuation {
                            spans.push(Span::styled("│   ", Style::default().fg(Color::DarkGray)));
                        } else {
                            spans.push(Span::raw("    "));
                        }
                    }
                }
                
                // Draw the branch connector
                if is_last {
                    spans.push(Span::styled("└── ", Style::default().fg(Color::DarkGray)));
                } else {
                    spans.push(Span::styled("├── ", Style::default().fg(Color::DarkGray)));
                }
            }
            
            // Add expand/collapse indicator for expandable items
            if item.expandable {
                let indicator = if is_expanded { "▾ " } else { "▸ " };
                let color = if is_selected && is_focused {
                    Color::Cyan
                } else {
                    Color::DarkGray
                };
                spans.push(Span::styled(indicator, Style::default().fg(color)));
            }
            
            // Add catalog type prefix for data catalogs
            if item.id.starts_with("data.catalog:") {
                let prefix = if item.name.contains("local") {
                    "[local] "
                } else if item.name.contains("s3") {
                    "[s3] "
                } else if item.name.contains("gcp") || item.name.contains("gcs") {
                    "[gcp] "
                } else {
                    ""
                };
                if !prefix.is_empty() {
                    spans.push(Span::styled(prefix, Style::default().fg(Color::DarkGray)));
                }
            }
            
            // Resource name - use consistent coloring (white/gray instead of rainbow)
            let name_color = if is_selected && is_focused {
                Color::White
            } else {
                Color::Gray
            };
            
            // Special formatting for root items and coming soon
            let name_style = if item.name.contains("(coming soon)") {
                Style::default().fg(Color::DarkGray)
            } else if item.depth == 0 {
                Style::default()
                    .fg(name_color)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED)
            } else {
                Style::default().fg(name_color)
            };
            
            spans.push(Span::styled(&item.name, name_style));
            
            // Add count for expandable items
            if item.expandable && is_expanded {
                let child_count = self.get_child_count(&item.id);
                if child_count > 0 {
                    spans.push(Span::styled(
                        format!(" ({})", child_count),
                        Style::default().fg(Color::DarkGray)
                    ));
                }
            }
            
            // Status indicators
            if item.resource_type == crate::models::ResourceType::Container && item.depth > 0 {
                // Dual status for containers: build + running
                if let Some(status) = &item.status {
                    let (symbol, color) = match status.as_str() {
                        "Built" => ("●", Color::Green),
                        "Not Built" => ("○", Color::DarkGray),
                        "Building" => ("◐", Color::Cyan),
                        _ => ("?", Color::Gray),
                    };
                    spans.push(Span::raw(" "));
                    spans.push(Span::styled(symbol, Style::default().fg(color)));
                }
                if let Some(run_status) = &item.secondary_status {
                    let (symbol, color) = match run_status.as_str() {
                        "running" => ("▶", Color::Green),
                        "exited" => ("■", Color::DarkGray),
                        "paused" => ("⏸", Color::Yellow),
                        _ => ("■", Color::DarkGray),
                    };
                    spans.push(Span::styled(symbol, Style::default().fg(color)));
                }
            } else if let Some(status) = &item.status {
                let (symbol, color) = match status.as_str() {
                    "Running" | "Ready" => ("●", Color::Green),
                    "Pending" | "Provisioning" => ("◐", Color::Yellow),
                    "Failed" | "Error" => ("✖", Color::Red),
                    "Stopped" => ("■", Color::DarkGray),
                    _ => ("?", Color::Gray),
                };
                spans.push(Span::raw(" "));
                spans.push(Span::styled(symbol, Style::default().fg(color)));
            }
            
            // Apply selection and focus styling
            let line_style = if is_selected && is_focused {
                Style::default()
                    .bg(Color::Rgb(40, 40, 40))
                    .add_modifier(Modifier::BOLD)
            } else if is_selected {
                Style::default()
                    .add_modifier(Modifier::DIM)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(spans)).style(line_style)
        }).collect();

        let list = List::new(items)
            .block(block);

        list.render(area, buf);
    }
}
