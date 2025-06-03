use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Widget},
};
use crate::app::App;

pub struct SearchView<'a> {
    app: &'a App,
}

impl<'a> SearchView<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for SearchView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a centered popup
        let popup_width = 70.min(area.width - 4);
        let popup_height = 25.min(area.height - 4);
        
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
            .title(" Search ")
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Cyan));
        
        let inner_area = block.inner(popup_area);
        block.render(popup_area, buf);
        
        // Split into input and results
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
            ])
            .split(inner_area);
        
        // Render search input
        let input_block = Block::default()
            .borders(Borders::ALL);
        
        let input_text = vec![
            Line::from(vec![
                Span::raw("> "),
                Span::raw(&self.app.search_input),
                Span::styled("█", Style::default().add_modifier(Modifier::SLOW_BLINK)),
            ]),
        ];
        
        let input = Paragraph::new(input_text)
            .block(input_block);
        
        input.render(chunks[0], buf);
        
        // Render search results
        let results = if self.app.search_input.is_empty() {
            vec![]
        } else {
            get_mock_search_results(&self.app.search_input)
        };
        
        if results.is_empty() && !self.app.search_input.is_empty() {
            let no_results = vec![
                Line::from(""),
                Line::from("No results found"),
                Line::from(""),
                Line::from("Try searching for:"),
                Line::from("  - Service names (kafka, etl-pipeline)"),
                Line::from("  - Template types (pyspark, python)"),
                Line::from("  - Resource types (deployment, node)"),
            ];
            
            let paragraph = Paragraph::new(no_results)
                .alignment(Alignment::Center);
            
            paragraph.render(chunks[1], buf);
        } else {
            let mut items = vec![];
            
            // Group results by category
            let mut services = vec![];
            let mut templates = vec![];
            let mut topics = vec![];
            let mut docs = vec![];
            
            for (category, name, description) in results {
                match category {
                    "Services" => services.push((name, description)),
                    "Templates" => templates.push((name, description)),
                    "Topics" => topics.push((name, description)),
                    "Documentation" => docs.push((name, description)),
                    _ => {}
                }
            }
            
            // Add categorized results
            if !services.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Services", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
                ])));
                for (name, desc) in services {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(name, Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled(desc, Style::default().fg(Color::DarkGray)),
                    ])));
                }
                items.push(ListItem::new(Line::from("")));
            }
            
            if !templates.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Templates", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
                ])));
                for (name, desc) in templates {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(name, Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled(desc, Style::default().fg(Color::DarkGray)),
                    ])));
                }
                items.push(ListItem::new(Line::from("")));
            }
            
            if !topics.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Topics", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
                ])));
                for (name, desc) in topics {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(name, Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled(desc, Style::default().fg(Color::DarkGray)),
                    ])));
                }
                items.push(ListItem::new(Line::from("")));
            }
            
            if !docs.is_empty() {
                items.push(ListItem::new(Line::from(vec![
                    Span::styled("Documentation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow)),
                ])));
                for (name, desc) in docs {
                    items.push(ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(name, Style::default().fg(Color::Cyan)),
                        Span::raw("  "),
                        Span::styled(desc, Style::default().fg(Color::DarkGray)),
                    ])));
                }
            }
            
            let list = List::new(items);
            list.render(chunks[1], buf);
        }
        
        // Show help at bottom
        let help = Line::from(vec![
            Span::styled("[↑↓]", Style::default().fg(Color::Yellow)),
            Span::raw(" navigate  "),
            Span::styled("[enter]", Style::default().fg(Color::Yellow)),
            Span::raw(" open  "),
            Span::styled("[tab]", Style::default().fg(Color::Yellow)),
            Span::raw(" next category  "),
            Span::styled("[esc]", Style::default().fg(Color::Yellow)),
            Span::raw(" close"),
        ]);
        
        buf.set_line(
            popup_area.x + 2,
            popup_area.y + popup_area.height - 1,
            &help,
            popup_area.width - 4,
        );
    }
}

fn get_mock_search_results(query: &str) -> Vec<(&'static str, &'static str, &'static str)> {
    let all_items = vec![
        ("Services", "kafka", "Kafka message broker (3/3 running)"),
        ("Templates", "kafka-consumer", "Basic Kafka consumer in Python"),
        ("Templates", "kafka-streams", "Stream processing with Kafka"),
        ("Topics", "raw-events", "15.2k messages/sec"),
        ("Topics", "processed-events", "8.7k messages/sec"),
        ("Documentation", "Kafka configuration", "How to configure Kafka topics"),
        ("Services", "etl-pipeline", "ETL pipeline deployment (running)"),
        ("Services", "api-service", "API service deployment (running)"),
        ("Templates", "stream-processor", "PySpark streaming template"),
        ("Templates", "batch-analytics", "Batch processing template"),
    ];
    
    all_items.into_iter()
        .filter(|(_, name, _)| name.to_lowercase().contains(&query.to_lowercase()))
        .collect()
}
