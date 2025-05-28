//! TUI rendering module
//!
//! This module handles rendering the TUI interface using ratatui.

use crate::config::{ServiceState, ServiceStatus};
use crate::tui::app::{App, Tab};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Cell, Paragraph, Row, Table, Tabs, Wrap,
    },
    Frame,
};

/// Main drawing function
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),     // Content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);
    draw_content(f, chunks[1], app);
    draw_footer(f, chunks[2]);
}

/// Draw the header with tabs
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let titles = vec!["Services", "Logs", "Config", "Help"];
    let selected = match app.current_tab {
        Tab::Services => 0,
        Tab::Logs => 1,
        Tab::Config => 2,
        Tab::Help => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default()
            .title(" Firestream v0.1.0 ")
            .borders(Borders::ALL))
        .highlight_style(Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD))
        .select(selected);

    f.render_widget(tabs, area);
}

/// Draw the main content area
fn draw_content(f: &mut Frame, area: Rect, app: &App) {
    match app.current_tab {
        Tab::Services => draw_services_tab(f, area, app),
        Tab::Logs => draw_logs_tab(f, area, app),
        Tab::Config => draw_config_tab(f, area, app),
        Tab::Help => draw_help_tab(f, area),
    }
}

/// Draw the services tab
fn draw_services_tab(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Services ");

    // Create table headers
    let headers = Row::new(vec![
        Cell::from("Service").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Status").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Version").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("CPU").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Memory").style(Style::default().add_modifier(Modifier::BOLD)),
        Cell::from("Actions").style(Style::default().add_modifier(Modifier::BOLD)),
    ])
    .height(1);

    // Create table rows
    let mut rows = Vec::new();
    let mut current_idx = 0;

    // Add installed services
    for (name, state) in &app.service_states {
        let is_selected = current_idx == app.selected_service;
        rows.push(create_service_row(name, Some(state), is_selected));
        current_idx += 1;
    }

    // Add available but not installed services
    for service in &app.available_services {
        if !app.service_states.contains_key(service) {
            let is_selected = current_idx == app.selected_service;
            rows.push(create_service_row(service, None, is_selected));
            current_idx += 1;
        }
    }

    let table = Table::new(
        rows,
        &[
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
        ],
    )
    .header(headers)
    .block(block)
    .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    f.render_widget(table, area);
}

/// Create a service row for the table
fn create_service_row<'a>(name: &'a str, state: Option<&'a ServiceState>, is_selected: bool) -> Row<'a> {
    let style = if is_selected {
        Style::default().bg(Color::DarkGray)
    } else {
        Style::default()
    };

    if let Some(state) = state {
        let status_style = match &state.status {
            ServiceStatus::Running => Style::default().fg(Color::Green),
            ServiceStatus::Stopped => Style::default().fg(Color::Yellow),
            ServiceStatus::Error(_) => Style::default().fg(Color::Red),
            ServiceStatus::Installing => Style::default().fg(Color::Blue),
            ServiceStatus::NotInstalled => Style::default().fg(Color::Gray),
        };

        let status_text = match &state.status {
            ServiceStatus::Running => "Running",
            ServiceStatus::Stopped => "Stopped",
            ServiceStatus::Error(_) => "Error",
            ServiceStatus::Installing => "Installing",
            ServiceStatus::NotInstalled => "Not Installed",
        };

        let version = state.installed_version.as_deref().unwrap_or("-");
        let (cpu, memory) = if let Some(usage) = &state.resource_usage {
            (format!("{:.1}", usage.cpu_cores), format!("{}Mi", usage.memory_mb))
        } else {
            ("-".to_string(), "-".to_string())
        };

        let actions = match state.status {
            ServiceStatus::Running => "[S]top [R]estart",
            ServiceStatus::Stopped => "[S]tart [D]elete",
            _ => "[L]ogs",
        };

        Row::new(vec![
            Cell::from(name).style(style),
            Cell::from(status_text).style(status_style.add_modifier(style.add_modifier)),
            Cell::from(version).style(style),
            Cell::from(cpu).style(style),
            Cell::from(memory).style(style),
            Cell::from(actions).style(style),
        ])
    } else {
        // Not installed service
        Row::new(vec![
            Cell::from(name).style(style),
            Cell::from("Not Installed").style(Style::default().fg(Color::Gray).add_modifier(style.add_modifier)),
            Cell::from("-").style(style),
            Cell::from("-").style(style),
            Cell::from("-").style(style),
            Cell::from("[I]nstall").style(style),
        ])
    }
}

/// Draw the logs tab
fn draw_logs_tab(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Logs ");

    let text = if let Some(service_name) = app.selected_service_name() {
        vec![
            Line::from(format!("Logs for service: {}", service_name)),
            Line::from(""),
            Line::from("[2024-01-01 12:00:00] Service started"),
            Line::from("[2024-01-01 12:00:01] Health check passed"),
            Line::from("[2024-01-01 12:00:02] Ready to accept connections"),
        ]
    } else {
        vec![Line::from("Select a service to view logs")]
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw the config tab
fn draw_config_tab(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Configuration ");

    let text = if let Some(service_name) = app.selected_service_name() {
        vec![
            Line::from(format!("Configuration for service: {}", service_name)),
            Line::from(""),
            Line::from("Press 'e' to edit configuration"),
            Line::from(""),
            Line::from("# Example configuration:"),
            Line::from("[metadata]"),
            Line::from(format!("name = \"{}\"", service_name)),
            Line::from("version = \"1.0.0\""),
        ]
    } else {
        vec![Line::from("Select a service to view configuration")]
    };

    let paragraph = Paragraph::new(text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw the help tab
fn draw_help_tab(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Help ");

    let help_text = vec![
        Line::from("Firestream - Data Infrastructure Management Tool"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ↑/k        Move up"),
        Line::from("  ↓/j        Move down"),
        Line::from("  Tab        Next tab"),
        Line::from("  Shift+Tab  Previous tab"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  i          Install service"),
        Line::from("  s          Start/Stop service"),
        Line::from("  r          Restart service"),
        Line::from("  l          View logs"),
        Line::from("  e          Edit configuration"),
        Line::from(""),
        Line::from(vec![
            Span::styled("General:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  ?          Show this help"),
        Line::from("  q/Ctrl+C   Quit"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

/// Draw the footer with key hints
fn draw_footer(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL);

    let hints = Paragraph::new(Line::from(vec![
        Span::from("[i]nstall "),
        Span::from("[s]tart/stop "),
        Span::from("[r]estart "),
        Span::from("[q]uit "),
        Span::from("[?]help"),
    ]))
    .block(block)
    .alignment(Alignment::Center);

    f.render_widget(hints, area);
}
