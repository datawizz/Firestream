use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

pub struct HelpView;

impl Widget for HelpView {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(" Firestream Help ")
            .title_alignment(Alignment::Center);

        let help_text = vec![
            Line::from(vec![
                Span::styled("Navigation", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  j/k or ↑/↓     Navigate items in current pane"),
            Line::from("  h/l or ←/→     Switch between panes"),
            Line::from("  Space          Expand/collapse tree nodes"),
            Line::from("  Enter          Select/activate item"),
            Line::from("  Tab            Move to next pane"),
            Line::from("  Esc            Back/cancel current operation"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Quick Actions", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  /              Global search overlay"),
            Line::from("  :              Command palette"),
            Line::from("  ?              Show this help"),
            Line::from("  n              New (context-aware)"),
            Line::from("  d              Deploy/Delete (context-aware)"),
            Line::from("  l              View logs"),
            Line::from("  s              Scale/Search (context-aware)"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Resources Pane", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  Space          Expand/collapse sections"),
            Line::from("  n              Create new resource"),
            Line::from("  d              Deploy template or delete resource"),
            Line::from("  b              Build container image"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Details Pane", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  s              Scale deployment"),
            Line::from("  r              Restart deployment"),
            Line::from("  e              Edit resource"),
            Line::from("  l              View logs"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Logs Pane", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  f              Follow logs (tail -f)"),
            Line::from("  Ctrl+C         Stop following logs"),
            Line::from(""),
            Line::from(vec![
                Span::styled("Command Palette", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  deploy <name>              Deploy a template"),
            Line::from("  scale <name> <replicas>    Scale deployment"),
            Line::from("  logs <name>                View logs for resource"),
            Line::from(""),
            Line::from(vec![
                Span::styled("General", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  q              Quit application"),
            Line::from("  Ctrl+C         Force quit"),
            Line::from(""),
            Line::from("Press Esc to close this help"),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        paragraph.render(area, buf);
    }
}
