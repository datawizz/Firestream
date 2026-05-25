use ratatui::{
    buffer::Buffer,
    layout::Rect,
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
                ResourceDetails::Container(manifest) => {
                    render_container_details(manifest, &self.app.bootstrap, area, buf, block);
                }
                ResourceDetails::Deployment(deployment) => {
                    render_deployment_details(deployment, area, buf, block);
                }
                ResourceDetails::Template(template) => {
                    render_template_details(template, area, buf, block);
                }
                ResourceDetails::Node(node) => {
                    render_node_details(node, area, buf, block);
                }
                ResourceDetails::Build(build) => {
                    render_build_details(build, area, buf, block);
                }
                ResourceDetails::Table(_) | ResourceDetails::Secret(_) => {
                    let text = vec![
                        Line::from("Details not available"),
                        Line::from(""),
                        Line::from("This feature is coming in a future release."),
                    ];
                    Paragraph::new(text)
                        .block(block)
                        .wrap(Wrap { trim: true })
                        .render(area, buf);
                }
            }
        } else {
            let text = vec![
                Line::from("No resource selected"),
                Line::from(""),
                Line::from("Select a resource from the left pane to view details"),
            ];

            Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: true })
                .render(area, buf);
        }
    }
}

fn render_container_details(
    manifest: &crate::backend::ContainerManifest,
    bootstrap: &crate::backend::BootstrapState,
    area: Rect, buf: &mut Buffer, block: Block,
) {
    let is_built = bootstrap.built_images.contains_key(&manifest.name);
    let (status_text, status_color) = if is_built {
        ("Built", Color::Green)
    } else {
        ("Not Built", Color::Yellow)
    };

    let mut lines = vec![
        Line::from(Span::styled(&manifest.name, Style::default().add_modifier(Modifier::BOLD))),
        Line::from("─".repeat(area.width.saturating_sub(4) as usize)),
        Line::from(vec![
            Span::raw("Image:       "),
            Span::styled(format!("● {}", status_text), Style::default().fg(status_color)),
        ]),
    ];

    // Running state
    if let Some(run_state) = bootstrap.running_containers.get(&manifest.name) {
        let (run_text, run_color) = match run_state.as_str() {
            "running" => ("Running", Color::Green),
            "exited" => ("Stopped", Color::DarkGray),
            _ => (run_state.as_str(), Color::Yellow),
        };
        lines.push(Line::from(vec![
            Span::raw("Container:   "),
            Span::styled(format!("▶ {}", run_text), Style::default().fg(run_color)),
        ]));
    } else {
        lines.push(Line::from(vec![
            Span::raw("Container:   "),
            Span::styled("■ Not deployed", Style::default().fg(Color::DarkGray)),
        ]));
    }

    lines.extend([
        Line::from(format!("Tag:         {}:{}", manifest.image_name, manifest.image_tag)),
        Line::from(format!("Nix Package: {}", manifest.nix_package)),
    ]);

    if !manifest.ports.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Ports", Style::default().add_modifier(Modifier::BOLD))));
        for (host, container) in &manifest.ports {
            lines.push(Line::from(format!("  {}:{}", host, container)));
        }
    }

    if !manifest.dependencies.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Dependencies", Style::default().add_modifier(Modifier::BOLD))));
        for dep in &manifest.dependencies {
            lines.push(Line::from(format!("  - {}", dep)));
        }
    }

    if let Some(ref creds) = manifest.credentials {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Credentials", Style::default().add_modifier(Modifier::BOLD))));
        if !creds.url.is_empty() {
            lines.push(Line::from(format!("  URL:      {}", creds.url)));
        }
        lines.push(Line::from(format!("  User:     {}", creds.username)));
        lines.push(Line::from(format!("  Password: {}", "●".repeat(creds.password.len().min(12)))));
    }

    lines.push(Line::from(""));
    lines.push(Line::from("─".repeat(area.width.saturating_sub(4) as usize)));
    lines.push(Line::from(vec![
        Span::styled("[B]", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        Span::raw("uild container"),
    ]));

    Paragraph::new(lines).block(block).wrap(Wrap { trim: true }).render(area, buf);
}

fn render_deployment_details(deployment: &crate::models::DeploymentDetail, area: Rect, buf: &mut Buffer, block: Block) {
    let status_color = match deployment.deployment.status {
        DeploymentStatus::Running => Color::Green,
        DeploymentStatus::Pending => Color::Yellow,
        DeploymentStatus::Failed => Color::Red,
        DeploymentStatus::Unknown => Color::Gray,
    };

    let mut lines = vec![
        Line::from(Span::styled(&deployment.deployment.name, Style::default().add_modifier(Modifier::BOLD))),
        Line::from("─".repeat(area.width.saturating_sub(4) as usize)),
        Line::from(vec![
            Span::raw("Status:     "),
            Span::styled(
                format!("● {:?}", deployment.deployment.status),
                Style::default().fg(status_color),
            ),
        ]),
        Line::from(format!(
            "Replicas:   {}/{}",
            deployment.deployment.replicas.ready,
            deployment.deployment.replicas.desired
        )),
    ];

    if !deployment.pods.is_empty() {
        lines.push(Line::from(""));
        lines.push(Line::from(Span::styled("Pods", Style::default().add_modifier(Modifier::BOLD))));
        lines.push(Line::from("─".repeat(area.width.saturating_sub(4) as usize)));

        for pod in &deployment.pods {
            let pod_status_color = match pod.status {
                PodStatus::Running => Color::Green,
                PodStatus::Pending => Color::Yellow,
                PodStatus::Failed => Color::Red,
                PodStatus::Succeeded => Color::Blue,
            };

            lines.push(Line::from(vec![
                Span::raw("  "),
                Span::raw(&pod.name),
                Span::raw(" "),
                Span::styled(format!("{:?}", pod.status), Style::default().fg(pod_status_color)),
            ]));
        }
    }

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .render(area, buf);
}

fn render_template_details(template: &crate::models::Template, area: Rect, buf: &mut Buffer, block: Block) {
    let lines = vec![
        Line::from(Span::styled(&template.name, Style::default().add_modifier(Modifier::BOLD))),
        Line::from("─".repeat(area.width.saturating_sub(4) as usize)),
        Line::from(format!("Type:        {:?}", template.template_type)),
        Line::from(format!("Version:     {}", template.version)),
        Line::from(format!(
            "Description: {}",
            template.description.as_deref().unwrap_or("N/A")
        )),
        Line::from(""),
        Line::from(format!("CPU:         {}", template.config.resources.cpu)),
        Line::from(format!("Memory:      {}", template.config.resources.memory)),
    ];

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .render(area, buf);
}

fn render_node_details(node: &crate::models::NodeDetail, area: Rect, buf: &mut Buffer, block: Block) {
    let status_color = match node.node.status {
        NodeStatus::Ready => Color::Green,
        NodeStatus::NotReady => Color::Red,
        NodeStatus::Provisioning => Color::Yellow,
        NodeStatus::Terminating => Color::Red,
    };

    let mut lines = vec![
        Line::from(Span::styled(&node.node.name, Style::default().add_modifier(Modifier::BOLD))),
        Line::from("─".repeat(area.width.saturating_sub(4) as usize)),
        Line::from(vec![
            Span::raw("Status:      "),
            Span::styled(format!("{:?}", node.node.status), Style::default().fg(status_color)),
        ]),
        Line::from(format!("Provider:    {:?}", node.node.provider)),
        Line::from(format!("Type:        {}", node.node.instance_type)),
        Line::from(format!("CPU:         {} cores", node.node.resources.cpu)),
        Line::from(format!("Memory:      {}", node.node.resources.memory)),
    ];

    if let Some(gpu) = &node.node.resources.gpu {
        lines.push(Line::from(format!("GPU:         {}x {}", gpu.count, gpu.gpu_type)));
    }

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .render(area, buf);
}

fn render_build_details(build: &crate::models::BuildStatus, area: Rect, buf: &mut Buffer, block: Block) {
    let status_color = match build.status {
        crate::models::BuildState::Pending => Color::Gray,
        crate::models::BuildState::Building => Color::Yellow,
        crate::models::BuildState::Success => Color::Green,
        crate::models::BuildState::Failed => Color::Red,
    };

    let lines = vec![
        Line::from(Span::styled(&build.build_id, Style::default().add_modifier(Modifier::BOLD))),
        Line::from("─".repeat(area.width.saturating_sub(4) as usize)),
        Line::from(vec![
            Span::raw("Status:      "),
            Span::styled(format!("{:?}", build.status), Style::default().fg(status_color)),
        ]),
        Line::from(format!("Progress:    {:.0}%", build.progress)),
        Line::from(format!(
            "Step:        {}",
            build.current_step.as_deref().unwrap_or("N/A"),
        )),
    ];

    Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: true })
        .render(area, buf);
}
