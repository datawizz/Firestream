//! Deploy tab for running deployment operations
//!
//! This module provides UI for deployment operations

use crate::deploy::{DeploymentMode, DeploymentManager, DeploymentProgress};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

/// Deploy tab state
pub struct DeployTab {
    pub selected_mode: usize,
    pub is_running: bool,
    pub progress: f64,
    pub log_buffer: Arc<Mutex<VecDeque<String>>>,
    pub current_step: String,
    pub deployment_handle: Option<tokio::task::JoinHandle<()>>,
}

impl Default for DeployTab {
    fn default() -> Self {
        Self::new()
    }
}

impl DeployTab {
    /// Create new deploy tab
    pub fn new() -> Self {
        Self {
            selected_mode: 0,
            is_running: false,
            progress: 0.0,
            log_buffer: Arc::new(Mutex::new(VecDeque::with_capacity(100))),
            current_step: String::new(),
            deployment_handle: None,
        }
    }

    /// Add log line
    pub fn add_log(&self, line: String) {
        let mut buffer = self.log_buffer.lock().unwrap();
        if buffer.len() >= 100 {
            buffer.pop_front();
        }
        buffer.push_back(line);
    }

    /// Start deployment
    pub async fn start_deployment(&mut self, mode: DeploymentMode) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_running {
            return Ok(());
        }

        self.is_running = true;
        self.progress = 0.0;
        self.current_step = "Initializing...".to_string();
        
        // Clear logs
        {
            let mut buffer = self.log_buffer.lock().unwrap();
            buffer.clear();
        }

        // Create progress channel
        let (tx, mut rx) = mpsc::channel::<DeploymentProgress>(100);
        
        // Clone for the closure
        let log_buffer = self.log_buffer.clone();
        
        // Spawn progress handler
        let _progress_handle = tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                let mut buffer = log_buffer.lock().unwrap();
                if buffer.len() >= 100 {
                    buffer.pop_front();
                }
                buffer.push_back(progress.message.clone());
            }
        });

        // Create deployment manager
        let mut manager = DeploymentManager::new(mode).await?;
        
        // Set progress callback
        let tx_clone = tx.clone();
        manager.set_progress_callback(move |progress| {
            let _ = tx_clone.blocking_send(progress);
        });

        // Spawn deployment task
        let deployment_handle = tokio::spawn(async move {
            match manager.deploy().await {
                Ok(_) => {
                    let _ = tx.send(DeploymentProgress {
                        step: "Complete".to_string(),
                        message: "Deployment completed successfully!".to_string(),
                        progress: 1.0,
                        is_error: false,
                    }).await;
                }
                Err(e) => {
                    let _ = tx.send(DeploymentProgress {
                        step: "Error".to_string(),
                        message: format!("Deployment failed: {}", e),
                        progress: 0.0,
                        is_error: true,
                    }).await;
                }
            }
        });

        self.deployment_handle = Some(deployment_handle);
        
        Ok(())
    }

    /// Check if deployment is complete
    pub fn check_deployment_status(&mut self) {
        if let Some(handle) = &self.deployment_handle {
            if handle.is_finished() {
                self.is_running = false;
                self.deployment_handle = None;
            }
        }
    }

    /// Draw the deploy tab
    pub fn draw(&self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(10), // Mode selection
                Constraint::Length(3),  // Progress
                Constraint::Min(0),     // Logs
            ])
            .split(area);

        self.draw_mode_selection(f, chunks[0]);
        self.draw_progress(f, chunks[1]);
        self.draw_logs(f, chunks[2]);
    }

    /// Draw mode selection area
    fn draw_mode_selection(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Deployment Modes ");

        let inner = block.inner(area);

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(inner);

        // Mode list
        let modes = DeploymentMode::all();
        let items: Vec<ListItem> = modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                let style = if i == self.selected_mode {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else if self.is_running {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(format!("  {}", mode.display_name())).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::RIGHT));

        // Description
        let selected_mode = &modes[self.selected_mode];
        let desc_text = if self.is_running {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Deployment in progress...",
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(format!("Running: {}", selected_mode.display_name())),
            ]
        } else {
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    selected_mode.display_name(),
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(selected_mode.description()),
                Line::from(""),
                Line::from(Span::styled(
                    "Press 'd' to deploy in this mode",
                    Style::default().fg(Color::Green),
                )),
            ]
        };

        let description = Paragraph::new(desc_text)
            .wrap(Wrap { trim: true });

        f.render_widget(block, area);
        f.render_widget(list, chunks[0]);
        f.render_widget(description, chunks[1]);
    }

    /// Draw progress bar
    fn draw_progress(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Progress ");

        let gauge = Gauge::default()
            .block(block)
            .gauge_style(Style::default().fg(Color::Cyan))
            .percent((self.progress * 100.0) as u16)
            .label(if self.is_running {
                self.current_step.clone()
            } else {
                "Ready".to_string()
            });

        f.render_widget(gauge, area);
    }

    /// Draw logs
    fn draw_logs(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Deployment Logs ");

        let buffer = self.log_buffer.lock().unwrap();
        let logs: Vec<Line> = buffer
            .iter()
            .map(|line| {
                // Color code based on content
                let style = if line.contains("ERROR") || line.contains("Failed") {
                    Style::default().fg(Color::Red)
                } else if line.contains("WARNING") {
                    Style::default().fg(Color::Yellow)
                } else if line.contains("SUCCESS") || line.contains("successfully") {
                    Style::default().fg(Color::Green)
                } else if line.starts_with("###") {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                Line::from(Span::styled(line, style))
            })
            .collect();

        let paragraph = Paragraph::new(logs)
            .block(block)
            .wrap(Wrap { trim: false });

        f.render_widget(paragraph, area);
    }
}
