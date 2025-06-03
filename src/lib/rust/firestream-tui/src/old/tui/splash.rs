//! Splash screen module for Firestream TUI
//!
//! Displays the Firestream banner and deployment mode selection

use crate::core::Result;
use crate::deploy::DeploymentMode;
use crossterm::{
    event::{self, Event, KeyCode},
};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::time::Duration;

/// Splash screen state
pub struct SplashScreen {
    selected_mode: usize,
    show_mode_selection: bool,
}

impl SplashScreen {
    /// Create new splash screen
    pub fn new() -> Self {
        Self {
            selected_mode: 0,
            show_mode_selection: false,
        }
    }

    /// Show splash screen and optionally get deployment mode
    pub async fn show<B: Backend>(
        &mut self,
        terminal: &mut Terminal<B>,
        auto_continue: bool,
    ) -> Result<Option<DeploymentMode>> {
        let mut frames = 0;
        let fade_duration = 30; // frames for fade effect

        loop {
            terminal.draw(|f| self.draw(f, frames, fade_duration))?;

            // Handle events
            if event::poll(Duration::from_millis(50))? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') => return Ok(None),
                        KeyCode::Enter | KeyCode::Char(' ') => {
                            if self.show_mode_selection {
                                let modes = DeploymentMode::all();
                                return Ok(Some(modes[self.selected_mode]));
                            } else if !auto_continue {
                                self.show_mode_selection = true;
                            } else {
                                return Ok(None);
                            }
                        }
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.show_mode_selection && self.selected_mode > 0 {
                                self.selected_mode -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            if self.show_mode_selection {
                                let max = DeploymentMode::all().len() - 1;
                                if self.selected_mode < max {
                                    self.selected_mode += 1;
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            frames += 1;

            // Auto-continue after animation
            if auto_continue && frames > fade_duration + 20 {
                return Ok(None);
            }
        }
    }

    /// Draw splash screen
    fn draw(&self, f: &mut Frame, frames: u32, fade_duration: u32) {
        let area = f.area();

        // Calculate fade-in alpha (0-255)
        let alpha = if frames < fade_duration {
            (frames as f32 / fade_duration as f32 * 255.0) as u8
        } else {
            255
        };

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(11), // Banner height
                Constraint::Percentage(10),
                Constraint::Min(0),
                Constraint::Percentage(10),
            ])
            .split(area);

        // Draw banner
        self.draw_banner(f, chunks[1], alpha);

        // Draw welcome message
        self.draw_welcome(f, chunks[2], alpha);

        // Draw mode selection or continue prompt
        if self.show_mode_selection {
            self.draw_mode_selection(f, chunks[3]);
        } else {
            self.draw_continue_prompt(f, chunks[3], alpha);
        }
    }

    /// Draw the Firestream banner
    fn draw_banner(&self, f: &mut Frame, area: Rect, alpha: u8) {
        let banner_lines = vec![
            "  ███████ ██ ██████  ███████ ███████ ████████ ██████  ███████  █████  ███    ███",
            "  ██      ██ ██   ██ ██      ██         ██    ██   ██ ██      ██   ██ ████  ████",
            "  █████   ██ ██████  █████   ███████    ██    ██████  █████   ███████ ██ ████ ██",
            "  ██      ██ ██   ██ ██           ██    ██    ██   ██ ██      ██   ██ ██  ██  ██",
            "  ██      ██ ██   ██ ███████ ███████    ██    ██   ██ ███████ ██   ██ ██      ██",
        ];

        let color = if alpha < 255 {
            Color::Rgb(
                (0x00 as f32 * alpha as f32 / 255.0) as u8,
                (0xFF as f32 * alpha as f32 / 255.0) as u8,
                (0xFF as f32 * alpha as f32 / 255.0) as u8,
            )
        } else {
            Color::Cyan
        };

        let banner_text: Vec<Line> = banner_lines
            .into_iter()
            .map(|line| {
                Line::from(Span::styled(
                    line,
                    Style::default().fg(color).add_modifier(Modifier::BOLD),
                ))
            })
            .collect();

        let banner = Paragraph::new(banner_text)
            .alignment(Alignment::Center)
            .block(Block::default());

        f.render_widget(banner, area);
    }

    /// Draw welcome message
    fn draw_welcome(&self, f: &mut Frame, area: Rect, alpha: u8) {
        let color = if alpha < 255 {
            Color::Rgb(
                (0xFF as f32 * alpha as f32 / 255.0) as u8,
                (0xFF as f32 * alpha as f32 / 255.0) as u8,
                (0xFF as f32 * alpha as f32 / 255.0) as u8,
            )
        } else {
            Color::White
        };

        let welcome = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Welcome to Firestream",
                Style::default().fg(color).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(
                "Data Infrastructure Management Tool",
                Style::default().fg(color),
            )),
        ])
        .alignment(Alignment::Center);

        f.render_widget(welcome, area);
    }

    /// Draw continue prompt
    fn draw_continue_prompt(&self, f: &mut Frame, area: Rect, alpha: u8) {
        let color = if alpha < 255 {
            Color::Rgb(
                (0x80 as f32 * alpha as f32 / 255.0) as u8,
                (0x80 as f32 * alpha as f32 / 255.0) as u8,
                (0x80 as f32 * alpha as f32 / 255.0) as u8,
            )
        } else {
            Color::Gray
        };

        let prompt = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                "Press ENTER to select deployment mode",
                Style::default().fg(color),
            )),
            Line::from(Span::styled(
                "Press SPACE to continue to management interface",
                Style::default().fg(color),
            )),
            Line::from(Span::styled(
                "Press ESC to exit",
                Style::default().fg(color),
            )),
        ])
        .alignment(Alignment::Center);

        f.render_widget(prompt, area);
    }

    /// Draw deployment mode selection
    fn draw_mode_selection(&self, f: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(" Select Deployment Mode ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));

        let inner = block.inner(area);

        // Create layout for list and description
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(40), Constraint::Percentage(60)])
            .split(inner);

        // Create mode list
        let modes = DeploymentMode::all();
        let items: Vec<ListItem> = modes
            .iter()
            .enumerate()
            .map(|(i, mode)| {
                let style = if i == self.selected_mode {
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(format!("  {}", mode.display_name())).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::RIGHT))
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        // Create description
        let selected_mode = &modes[self.selected_mode];
        let description = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(
                selected_mode.display_name(),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
            )),
            Line::from(""),
            Line::from(selected_mode.description()),
            Line::from(""),
            Line::from(Span::styled(
                "Press ENTER to start in this mode",
                Style::default().fg(Color::Green),
            )),
            Line::from(Span::styled(
                "Press ESC to cancel",
                Style::default().fg(Color::Gray),
            )),
        ])
        .block(Block::default().borders(Borders::NONE))
        .wrap(ratatui::widgets::Wrap { trim: true });

        f.render_widget(block, area);
        f.render_widget(list, chunks[0]);
        f.render_widget(description, chunks[1]);
    }
}
