use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap, List, ListItem},
};
use crate::app::App;
use crate::models::IcebergCatalogType;

pub struct IcebergConfigView<'a> {
    _app: &'a App,
}

impl<'a> IcebergConfigView<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { _app: app }
    }
}

impl<'a> Widget for IcebergConfigView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let chunks = Layout::default()
            .direction(ratatui::layout::Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ])
            .split(area);
        
        // Left pane - Catalog list
        self.render_catalog_list(chunks[0], buf);
        
        // Right pane - Configuration
        self.render_configuration(chunks[1], buf);
    }
}

impl<'a> IcebergConfigView<'a> {
    fn render_catalog_list(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Iceberg Catalogs ")
            .border_style(Style::default());
        
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        // Mock catalog list for now
        let catalogs = vec![
            ("local", IcebergCatalogType::Memory, true),
            ("s3_prod", IcebergCatalogType::Rest, false),
            ("gcs_analytics", IcebergCatalogType::Rest, false),
        ];
        
        let items: Vec<ListItem> = catalogs.iter().enumerate().map(|(idx, (name, catalog_type, is_default))| {
            let spans = vec![
                Span::raw(if *is_default { "⭐ " } else { "   " }),
                Span::raw(*name),
                Span::raw(" ("),
                Span::raw(match catalog_type {
                    IcebergCatalogType::Memory => "Local",
                    IcebergCatalogType::Rest => "REST",
                    IcebergCatalogType::FileSystem => "FileSystem",
                }),
                Span::raw(")"),
            ];
            
            let style = if idx == 0 { // Mock selection
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            ListItem::new(Line::from(spans)).style(style)
        }).collect();
        
        let list = List::new(items);
        list.render(inner_area, buf);
        
        // Add new catalog hint
        let hint = Line::from(vec![
            Span::styled("[N]", Style::default().fg(Color::Yellow)),
            Span::raw("ew  "),
            Span::styled("[D]", Style::default().fg(Color::Red)),
            Span::raw("elete  "),
            Span::styled("[T]", Style::default().fg(Color::Green)),
            Span::raw("est"),
        ]);
        
        let hint_area = Rect {
            x: area.x + 1,
            y: area.y + area.height - 2,
            width: area.width - 2,
            height: 1,
        };
        buf.set_line(hint_area.x, hint_area.y, &hint, hint_area.width);
    }
    
    fn render_configuration(&self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Catalog Configuration ")
            .border_style(Style::default());
        
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        // Mock configuration for local catalog
        let lines = vec![
            Line::from(vec![
                Span::styled("local", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("─".repeat(inner_area.width as usize)),
            Line::from(""),
            Line::from(vec![
                Span::styled("Storage Backend", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("Type:        Local Filesystem"),
            Line::from("Path:        /tmp/iceberg-warehouse"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Configuration", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("Auto-create: ✓ Enabled"),
            Line::from("Compression: Snappy"),
            Line::from("Format:      Parquet"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Statistics", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("Namespaces:  2"),
            Line::from("Tables:      5"),
            Line::from("Total Size:  1.2 GB"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Connection Status", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(vec![
                Span::raw("Status:      "),
                Span::styled("● Connected", Style::default().fg(Color::Green)),
            ]),
            Line::from("Last Check:  2 seconds ago"),
        ];
        
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true });
        
        paragraph.render(inner_area, buf);
        
        // Storage type selector for new catalogs
        if false { // When creating new catalog
            self.render_storage_selector(inner_area, buf);
        }
    }
    
    fn render_storage_selector(&self, area: Rect, buf: &mut Buffer) {
        let lines = vec![
            Line::from(vec![
                Span::styled("New Catalog", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from("─".repeat(area.width as usize)),
            Line::from(""),
            Line::from("Name: my_catalog_"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Storage Backend", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("  [•] Local Filesystem"),
            Line::from("  [ ] Amazon S3"),
            Line::from("  [ ] Google Cloud Storage"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Local Configuration", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("Path: /tmp/iceberg-warehouse/my_catalog"),
            Line::from(""),
            Line::from(vec![
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Create  "),
                Span::styled("[Esc]", Style::default().fg(Color::Red)),
                Span::raw(" Cancel"),
            ]),
        ];
        
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: true });
        
        paragraph.render(area, buf);
    }
}

// S3 Configuration Form
pub struct S3ConfigForm {
    pub bucket: String,
    pub prefix: Option<String>,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: Option<String>,
    pub selected_field: usize,
}

impl S3ConfigForm {
    pub fn new() -> Self {
        Self {
            bucket: String::new(),
            prefix: None,
            region: "us-east-1".to_string(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            endpoint: None,
            selected_field: 0,
        }
    }
    
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        let empty_string = String::new();
        let prefix_value = self.prefix.as_ref().unwrap_or(&empty_string);
        let endpoint_value = self.endpoint.as_ref().unwrap_or(&empty_string);
        
        let fields = vec![
            ("Bucket", &self.bucket, false),
            ("Prefix", prefix_value, false),
            ("Region", &self.region, false),
            ("Access Key ID", &self.access_key_id, false),
            ("Secret Access Key", &self.secret_access_key, true), // Password field
            ("Endpoint (optional)", endpoint_value, false),
        ];
        
        let mut y = area.y;
        
        for (idx, (label, value, is_password)) in fields.iter().enumerate() {
            let is_selected = idx == self.selected_field;
            let label_style = if is_selected {
                Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            
            // Label
            let label_line = Line::from(vec![
                Span::styled(format!("{}:", label), label_style),
            ]);
            buf.set_line(area.x, y, &label_line, area.width);
            y += 1;
            
            // Input box
            let input_area = Rect {
                x: area.x,
                y,
                width: area.width.min(50),
                height: 3,
            };
            
            let input_style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            };
            
            let input_block = Block::default()
                .borders(Borders::ALL)
                .border_style(input_style);
            
            let inner = input_block.inner(input_area);
            input_block.render(input_area, buf);
            
            // Value
            let display_value = if *is_password && !value.is_empty() {
                "•".repeat(value.len())
            } else {
                value.to_string()
            };
            
            let value_line = Line::from(display_value);
            buf.set_line(inner.x, inner.y, &value_line, inner.width);
            
            y += 3; // Space for input box
        }
    }
}
