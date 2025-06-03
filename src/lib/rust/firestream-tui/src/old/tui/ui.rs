//! TUI rendering module
//!
//! This module handles rendering the TUI interface using ratatui.

use crate::config::{ServiceState, ServiceStatus};
use crate::state::{ChangeType, ChangeImpact};
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

/// Draw the state tab
fn draw_state_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // State content
            Constraint::Length(3),  // Status bar
        ])
        .split(area);

    // Draw state content
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" State ");

    let content = if let Some(state) = &app.firestream_state {
        vec![
            Line::from(format!("Version: {}", state.version)),
            Line::from(format!("Serial: {}", state.serial)),
            Line::from(format!("Last Modified: {}", state.last_modified.format("%Y-%m-%d %H:%M:%S"))),
            Line::from(format!("Last Modified By: {}", state.last_modified_by)),
            Line::from(""),
            Line::from(Span::styled("Infrastructure:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(format!("  Stack: {}", state.infrastructure.stack_name.as_deref().unwrap_or("Not configured"))),
            Line::from(format!("  Provider: {}", state.infrastructure.provider.as_ref().map(|p| format!("{:?}", p)).unwrap_or("None".to_string()))),
            Line::from(""),
            Line::from(Span::styled("Resources:", Style::default().add_modifier(Modifier::BOLD))),
            Line::from(format!("  Builds: {}", state.builds.len())),
            Line::from(format!("  Deployments: {}", state.deployments.len())),
            Line::from(format!("  Cluster: {}", state.cluster.name.as_deref().unwrap_or("Not configured"))),
        ]
    } else {
        vec![
            Line::from("No state loaded"),
            Line::from(""),
            Line::from("Press 'r' to refresh state"),
        ]
    };

    let paragraph = Paragraph::new(content)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, chunks[0]);

    // Draw status bar
    let status_block = Block::default()
        .borders(Borders::ALL);

    let status_text = if let Some(op) = &app.state_operation {
        Line::from(op.as_str())
    } else {
        Line::from("[r]efresh [l]ock [u]nlock")
    };

    let status = Paragraph::new(status_text)
        .block(status_block)
        .alignment(Alignment::Center);

    f.render_widget(status, chunks[1]);
}

/// Draw the plan tab
fn draw_plan_tab(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),     // Plan content
            Constraint::Length(3),  // Action bar
        ])
        .split(area);

    // Draw plan content
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Execution Plan ");

    if let Some(plan) = &app.current_plan {
        // Create table for changes
        let headers = Row::new(vec![
            Cell::from("Action").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Resource").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Description").style(Style::default().add_modifier(Modifier::BOLD)),
            Cell::from("Impact").style(Style::default().add_modifier(Modifier::BOLD)),
        ])
        .height(1);

        let rows: Vec<Row> = plan.changes.iter().enumerate().map(|(idx, change)| {
            let is_selected = idx == app.selected_state_item;
            let style = if is_selected {
                Style::default().bg(Color::DarkGray)
            } else {
                Style::default()
            };

            let action_symbol = match change.change_type {
                ChangeType::Create => "+",
                ChangeType::Update => "~",
                ChangeType::Delete => "-",
                ChangeType::Replace => "!",
                ChangeType::NoOp => " ",
            };

            let action_style = match change.change_type {
                ChangeType::Create => Style::default().fg(Color::Green),
                ChangeType::Update => Style::default().fg(Color::Yellow),
                ChangeType::Delete => Style::default().fg(Color::Red),
                ChangeType::Replace => Style::default().fg(Color::Magenta),
                ChangeType::NoOp => Style::default(),
            };

            let impact_style = match change.impact {
                ChangeImpact::None => Style::default(),
                ChangeImpact::Low => Style::default().fg(Color::Green),
                ChangeImpact::Medium => Style::default().fg(Color::Yellow),
                ChangeImpact::High => Style::default().fg(Color::Red),
                ChangeImpact::Critical => Style::default().fg(Color::Magenta),
            };

            Row::new(vec![
                Cell::from(format!("{} {:?}", action_symbol, change.change_type))
                    .style(action_style.add_modifier(style.add_modifier)),
                Cell::from(change.resource_id.clone()).style(style),
                Cell::from(change.description.clone()).style(style),
                Cell::from(format!("{:?}", change.impact))
                    .style(impact_style.add_modifier(style.add_modifier)),
            ])
        }).collect();

        let table = Table::new(
            rows,
            &[
                Constraint::Percentage(15),
                Constraint::Percentage(25),
                Constraint::Percentage(45),
                Constraint::Percentage(15),
            ],
        )
        .header(headers)
        .block(block)
        .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        f.render_widget(table, chunks[0]);
    } else {
        let text = vec![
            Line::from("No plan created"),
            Line::from(""),
            Line::from("Press 'p' to create a new plan"),
        ];

        let paragraph = Paragraph::new(text)
            .block(block)
            .wrap(Wrap { trim: true });

        f.render_widget(paragraph, chunks[0]);
    }

    // Draw action bar
    let action_block = Block::default()
        .borders(Borders::ALL);

    let action_text = if app.current_plan.is_some() {
        Line::from("[p]lan [d]iscard | ↑↓ Navigate")
    } else {
        Line::from("[p]lan")
    };

    let actions = Paragraph::new(action_text)
        .block(action_block)
        .alignment(Alignment::Center);

    f.render_widget(actions, chunks[1]);
}

/// Draw the deploy tab
fn draw_deploy_tab(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Deploy ");

    let content = if let Some(plan) = &app.current_plan {
        let status_color = match plan.status {
            crate::state::PlanStatus::Pending => Color::Yellow,
            crate::state::PlanStatus::Approved => Color::Green,
            crate::state::PlanStatus::Applying => Color::Blue,
            crate::state::PlanStatus::Applied => Color::Green,
            crate::state::PlanStatus::Failed => Color::Red,
            crate::state::PlanStatus::Cancelled => Color::DarkGray,
        };

        vec![
            Line::from(format!("Plan ID: {}", plan.id)),
            Line::from(format!("Created: {}", plan.created_at.format("%Y-%m-%d %H:%M:%S"))),
            Line::from(format!("Created By: {}", plan.created_by)),
            Line::from(""),
            Line::from(vec![
                Span::from("Status: "),
                Span::styled(format!("{:?}", plan.status), Style::default().fg(status_color).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(format!("Changes: {}", plan.changes.len())),
            Line::from(""),
        ]
    } else {
        vec![
            Line::from("No plan available for deployment"),
            Line::from(""),
            Line::from("Create a plan in the Plan tab first"),
        ]
    };

    let mut paragraph_lines = content;

    // Add action hints based on plan status
    if let Some(plan) = &app.current_plan {
        match plan.status {
            crate::state::PlanStatus::Pending => {
                paragraph_lines.push(Line::from(""));
                paragraph_lines.push(Line::from(Span::styled("This plan requires approval before applying.", Style::default().fg(Color::Yellow))));
                paragraph_lines.push(Line::from(""));
                paragraph_lines.push(Line::from("Press 'y' to approve or 'n' to reject"));
            }
            crate::state::PlanStatus::Approved => {
                paragraph_lines.push(Line::from(""));
                paragraph_lines.push(Line::from(Span::styled("Plan is approved and ready to apply.", Style::default().fg(Color::Green))));
                paragraph_lines.push(Line::from(""));
                paragraph_lines.push(Line::from("Press 'a' to apply the plan"));
            }
            _ => {}
        }
    }

    // Add operation status if any
    if let Some(op) = &app.state_operation {
        paragraph_lines.push(Line::from(""));
        paragraph_lines.push(Line::from(Span::styled(op, Style::default().fg(Color::Cyan))));
    }

    let paragraph = Paragraph::new(paragraph_lines)
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, area);
}

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
    draw_footer(f, chunks[2], app);
}

/// Draw the header with tabs
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let titles = vec!["Services", "State", "Plan", "Deploy", "Logs", "Config", "Help"];
    let selected = match app.current_tab {
        Tab::Services => 0,
        Tab::State => 1,
        Tab::Plan => 2,
        Tab::Deploy => 3,
        Tab::Logs => 4,
        Tab::Config => 5,
        Tab::Help => 6,
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
        Tab::State => draw_state_tab(f, area, app),
        Tab::Plan => draw_plan_tab(f, area, app),
        Tab::Deploy => draw_deploy_tab(f, area, app),
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
        Line::from("Firestream - Kubernetes Deployment Platform"),
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
            Span::styled("Services Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  i          Install service"),
        Line::from("  s          Start/Stop service"),
        Line::from("  r          Restart service"),
        Line::from(""),
        Line::from(vec![
            Span::styled("State Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  r          Refresh state"),
        Line::from("  l          Lock state"),
        Line::from("  u          Unlock state"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Plan Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  p          Create plan"),
        Line::from("  d          Discard plan"),
        Line::from(""),
        Line::from(vec![
            Span::styled("Deploy Tab:", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("  a          Apply plan"),
        Line::from("  y          Approve plan"),
        Line::from("  n          Reject plan"),
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
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL);

    let hints = match app.current_tab {
        Tab::Services => Line::from(vec![
            Span::from("[i]nstall | [s]tart/stop | [r]estart | [q]uit"),
        ]),
        Tab::State => Line::from(vec![
            Span::from("[r]efresh | [l]ock | [u]nlock | [q]uit"),
        ]),
        Tab::Plan => Line::from(vec![
            Span::from("[p]lan | [d]iscard | ↑↓: Navigate | [q]uit"),
        ]),
        Tab::Deploy => Line::from(vec![
            Span::from("[a]pply | [y]approve | [n]reject | [q]uit"),
        ]),
        _ => Line::from(vec![
            Span::from("Tab/Shift+Tab: Navigate | ?: Help | q: Quit"),
        ]),
    };

    let paragraph = Paragraph::new(hints)
        .block(block)
        .alignment(Alignment::Center);

    f.render_widget(paragraph, area);
}
