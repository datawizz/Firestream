use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};
use crate::app::{App, Pane, ResourceDetails};
use crate::models::{DeploymentStatus, PodStatus, NodeStatus};

pub struct DetailsPane<'a> {
    app: &'a App,
}

impl<'a> DetailsPane<'a> {
    pub fn new(app: &'a App) -> Self {
        Self { app }
    }
}

impl<'a> Widget for DetailsPane<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let is_focused = self.app.focused_pane == Pane::Details;
        
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Details ")
            .border_style(if is_focused {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        if let Some(details) = &self.app.selected_details {
            match details {
                ResourceDetails::Deployment(deployment) => {
                    render_deployment_details(deployment, area, buf, block);
                }
                ResourceDetails::Template(template) => {
                    render_template_details(template, area, buf, block);
                }
                ResourceDetails::Node(node) => {
                    render_node_details(node, area, buf, block);
                }
                ResourceDetails::Table(table) => {
                    render_table_details(table, area, buf, block);
                }
                ResourceDetails::Build(build) => {
                    render_build_details(build, area, buf, block);
                }
                ResourceDetails::Secret(secret) => {
                    render_secret_details(secret, area, buf, block);
                }
            }
        } else {
            let text = vec![
                Line::from("No resource selected"),
                Line::from(""),
                Line::from("Select a resource from the left pane to view details"),
            ];
            
            let paragraph = Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true });
            
            paragraph.render(area, buf);
        }
    }
}

fn render_deployment_details(deployment: &crate::models::DeploymentDetail, area: Rect, buf: &mut Buffer, block: Block) {
    let chunks = Layout::default()
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(area);

    let status_color = match deployment.deployment.status {
        DeploymentStatus::Running => Color::Green,
        DeploymentStatus::Pending => Color::Yellow,
        DeploymentStatus::Failed => Color::Red,
        DeploymentStatus::Unknown => Color::Gray,
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(&deployment.deployment.name, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(vec![
            Span::raw("Status:     "),
            Span::styled(
                format!("● {:?}", deployment.deployment.status),
                Style::default().fg(status_color)
            ),
        ]),
        Line::from(format!("Type:       pyspark")), // TODO: get from template
        Line::from(format!("Replicas:   {}/{}", 
            deployment.deployment.replicas.ready,
            deployment.deployment.replicas.desired
        )),
        Line::from(format!("CPU:        45% ████████░░░░░░░░")), // TODO: actual metrics
        Line::from(format!("Memory:     2.1G / 4.0G")), // TODO: actual metrics
        Line::from(format!("Created:    2 hours ago")), // TODO: format timestamp
        Line::from(""),
    ];

    // Add Kafka topics if available
    lines.push(Line::from("Kafka Topics:"));
    lines.push(Line::from("  input:    raw-events (5.2k msg/s)"));
    lines.push(Line::from("  output:   processed-events (4.8k msg/s)"));
    
    // Add pods section
    if !deployment.pods.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled("Pods", Style::default().add_modifier(Modifier::BOLD)),
        ]));
        lines.push(Line::from("─".repeat(area.width as usize - 2)));
        
        for pod in &deployment.pods {
            let pod_status_color = match pod.status {
                PodStatus::Running => Color::Green,
                PodStatus::Pending => Color::Yellow,
                PodStatus::Failed => Color::Red,
                PodStatus::Succeeded => Color::Blue,
            };
            
            lines.push(Line::from(vec![
                Span::raw("▼ "),
                Span::raw(&pod.name),
                Span::raw(" ("),
                Span::raw(&pod.node),
                Span::raw(")"),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  Status:    "),
                Span::styled(format!("{:?}", pod.status), Style::default().fg(pod_status_color)),
            ]));
            
            for container in &pod.containers {
                lines.push(Line::from(format!("  CPU:       {} cores", container.cpu)));
                lines.push(Line::from(format!("  Memory:    {}", container.memory)));
            }
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    paragraph.render(chunks[0], buf);
}

fn render_template_details(template: &crate::models::Template, area: Rect, buf: &mut Buffer, block: Block) {
    let lines = vec![
        Line::from(vec![
            Span::styled(&template.name, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(format!("Type:        {:?}", template.template_type)),
        Line::from(format!("Version:     {}", template.version)),
        Line::from(format!("Description: {}", template.description.as_deref().unwrap_or("N/A"))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Configuration", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(format!("CPU:         {}", template.config.resources.cpu)),
        Line::from(format!("Memory:      {}", template.config.resources.memory)),
        Line::from(format!("GPU:         {}", if template.config.resources.gpu.unwrap_or(false) { "enabled" } else { "disabled" })),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    paragraph.render(area, buf);
}

fn render_node_details(node: &crate::models::NodeDetail, area: Rect, buf: &mut Buffer, block: Block) {
    let status_color = match node.node.status {
        NodeStatus::Ready => Color::Green,
        NodeStatus::NotReady => Color::Red,
        NodeStatus::Provisioning => Color::Yellow,
        NodeStatus::Terminating => Color::Red,
    };

    let mut lines = vec![
        Line::from(vec![
            Span::styled(&node.node.name, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(vec![
            Span::raw("Status:      "),
            Span::styled(format!("{:?}", node.node.status), Style::default().fg(status_color)),
        ]),
        Line::from(format!("Provider:    {:?}", node.node.provider)),
        Line::from(format!("Type:        {}", node.node.instance_type)),
        Line::from(format!("Spot:        {}", if node.node.spot { "Yes" } else { "No" })),
        Line::from(""),
        Line::from(vec![
            Span::styled("Resources", Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(format!("CPU:         {} cores ({:.1} used)", node.node.resources.cpu, node.utilization.cpu)),
        Line::from(format!("Memory:      {} ({:.1} GB used)", node.node.resources.memory, node.utilization.memory)),
    ];

    if let Some(gpu) = &node.node.resources.gpu {
        lines.push(Line::from(format!("GPU:         {}x {} ({:.0}% utilized)", 
            gpu.count, gpu.gpu_type, node.utilization.gpu.unwrap_or(0.0)
        )));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    paragraph.render(area, buf);
}

fn render_table_details(table: &crate::models::DeltaTable, area: Rect, buf: &mut Buffer, block: Block) {
    let lines = vec![
        Line::from(vec![
            Span::styled(&table.name, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(format!("Format:      {}", table.format)),
        Line::from(format!("Location:    {}", table.location)),
        Line::from(format!("Partitions:  {}", table.partition_columns.join(", "))),
        Line::from(format!("Size:        {:.1} GB", table.size_in_bytes as f64 / 1_000_000_000.0)),
        Line::from(format!("Row Count:   {:.1}M", table.num_files as f64 / 1_000_000.0)),
        Line::from(format!("Updated:     {} ago", "2 minutes")), // TODO: format timestamp
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    paragraph.render(area, buf);
}

fn render_build_details(build: &crate::models::BuildStatus, area: Rect, buf: &mut Buffer, block: Block) {
    let status_color = match build.status {
        crate::models::BuildState::Pending => Color::Gray,
        crate::models::BuildState::Building => Color::Yellow,
        crate::models::BuildState::Success => Color::Green,
        crate::models::BuildState::Failed => Color::Red,
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(&build.build_id, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(vec![
            Span::raw("Status:      "),
            Span::styled(format!("{:?}", build.status), Style::default().fg(status_color)),
        ]),
        Line::from(format!("Progress:    {:.0}%", build.progress)),
        Line::from(format!("Step:        {} / {}", 
            build.current_step.as_deref().unwrap_or("N/A"),
            build.total_steps.map(|s| s.to_string()).unwrap_or("?".to_string())
        )),
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    paragraph.render(area, buf);
}

fn render_secret_details(secret: &crate::models::SecretInfo, area: Rect, buf: &mut Buffer, block: Block) {
    let lines = vec![
        Line::from(vec![
            Span::styled(&secret.name, Style::default().add_modifier(Modifier::BOLD)),
        ]),
        Line::from("─".repeat(area.width as usize - 2)),
        Line::from(format!("Namespace:   {}", secret.namespace)),
        Line::from(format!("Type:        {:?}", secret.secret_type)),
        Line::from(format!("Keys:        {}", secret.keys.join(", "))),
        Line::from(format!("Created:     {} ago", "10 days")), // TODO: format timestamp
        Line::from(format!("Updated:     {} ago", "2 days")), // TODO: format timestamp
    ];

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true });

    paragraph.render(area, buf);
}
