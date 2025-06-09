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
    
    fn render_secret_details(&self, secret: &crate::models::SecretInfo, area: Rect, buf: &mut Buffer, block: Block) {
        let inner_area = block.inner(area);
        block.render(area, buf);
        
        let chunks = Layout::default()
            .constraints([
                Constraint::Length(1),   // Name
                Constraint::Length(1),   // Separator
                Constraint::Length(1),   // Namespace
                Constraint::Length(1),   // Type
                Constraint::Length(1),   // Created
                Constraint::Length(1),   // Updated
                Constraint::Length(1),   // Spacing
                Constraint::Length(1),   // Secret Values header
                Constraint::Length(1),   // Separator
                Constraint::Min(0),      // Fields
                Constraint::Length(2),   // Instructions
            ])
            .split(inner_area);

        // Header info
        let name = Line::from(vec![
            Span::styled(&secret.name, Style::default().add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(chunks[0].x, chunks[0].y, &name, chunks[0].width);
        
        let separator = Line::from("─".repeat(chunks[1].width as usize));
        buf.set_line(chunks[1].x, chunks[1].y, &separator, chunks[1].width);
        
        let namespace = Line::from(format!("Namespace:   {}", secret.namespace));
        buf.set_line(chunks[2].x, chunks[2].y, &namespace, chunks[2].width);
        
        let secret_type = Line::from(format!("Type:        {:?}", secret.secret_type));
        buf.set_line(chunks[3].x, chunks[3].y, &secret_type, chunks[3].width);
        
        let created = Line::from(format!("Created:     {} ago", "10 days")); // TODO: format timestamp
        buf.set_line(chunks[4].x, chunks[4].y, &created, chunks[4].width);
        
        let updated = Line::from(format!("Updated:     {} ago", "2 days")); // TODO: format timestamp
        buf.set_line(chunks[5].x, chunks[5].y, &updated, chunks[5].width);
        
        let values_header = Line::from(vec![
            Span::styled("Secret Values", Style::default().add_modifier(Modifier::BOLD)),
        ]);
        buf.set_line(chunks[7].x, chunks[7].y, &values_header, chunks[7].width);
        
        let separator2 = Line::from("─".repeat(chunks[8].width as usize));
        buf.set_line(chunks[8].x, chunks[8].y, &separator2, chunks[8].width);
        
        // Render the secret fields
        if let Some(ref secret_state) = self.app.secret_editing {
            let fields_area = chunks[9];
            let field_height = 3; // Each field takes 3 lines (label, input box, spacing)
            
            for (idx, field) in secret_state.fields.iter().enumerate() {
                let field_y = fields_area.y + (idx as u16 * field_height);
                
                if field_y + field_height > fields_area.y + fields_area.height {
                    break; // Don't render fields that would overflow
                }
                
                let is_selected = idx == secret_state.selected_field;
                let is_editing = is_selected && secret_state.is_editing;
                
                // Label
                let label_style = if is_selected && !is_editing {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                let label = Line::from(vec![
                    Span::styled(format!("{}:", field.key), label_style),
                ]);
                buf.set_line(fields_area.x, field_y, &label, fields_area.width);
                
                // Input box
                let input_y = field_y + 1;
                let input_style = if is_editing {
                    Style::default().bg(Color::DarkGray)
                } else if is_selected {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                
                let display_value = if field.is_password && !is_editing {
                    "•".repeat(field.value.len().min(20))
                } else {
                    field.value.clone()
                };
                
                // Draw input box border
                let input_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(input_style);
                let input_area = Rect {
                    x: fields_area.x,
                    y: input_y,
                    width: fields_area.width.min(50), // Max width of 50
                    height: 3,
                };
                
                // Get inner area before rendering the block
                let value_area = input_block.inner(input_area);
                input_block.render(input_area, buf);
                
                // Draw value inside box
                let value_line = if is_editing {
                    Line::from(vec![
                        Span::raw(&display_value),
                        Span::styled("█", Style::default().add_modifier(Modifier::SLOW_BLINK)), // Cursor
                    ])
                } else {
                    Line::from(display_value)
                };
                buf.set_line(value_area.x, value_area.y, &value_line, value_area.width);
            }
        }
        
        // Instructions
        let instructions = if let Some(ref secret_state) = self.app.secret_editing {
            if secret_state.is_editing {
                Line::from(vec![
                    Span::styled("Editing: ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Enter", Style::default().fg(Color::Yellow)),
                    Span::styled(" to save, ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Esc", Style::default().fg(Color::Yellow)),
                    Span::styled(" to cancel", Style::default().fg(Color::DarkGray)),
                ])
            } else {
                Line::from(vec![
                    Span::styled("↑↓ Navigate  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Enter", Style::default().fg(Color::Yellow)),
                    Span::styled(" to edit field", Style::default().fg(Color::DarkGray)),
                ])
            }
        } else {
            Line::from(vec![
                Span::styled("Loading secret data...", Style::default().fg(Color::DarkGray)),
            ])
        };
        
        let instructions_y = area.y + area.height - 2;
        buf.set_line(chunks[10].x, instructions_y, &instructions, chunks[10].width);
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
                    self.render_secret_details(secret, area, buf, block);
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


