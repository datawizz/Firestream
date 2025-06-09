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
            Line::from("  ↑/↓            Navigate items in current pane"),
            Line::from("  ←/→            Expand/collapse in Resources pane"),
            Line::from("  ←/→            Switch between panes (Details/Logs)"),
            Line::from("  Enter          Select/activate item"),
            Line::from("  Tab            Move to next pane"),
            Line::from("  Shift+Tab      Move to previous pane"),
            Line::from(""),
            Line::from(vec![
                Span::styled("General", Style::default().add_modifier(Modifier::BOLD).fg(Color::Cyan)),
            ]),
            Line::from("  Ctrl+C         Quit application"),
        ];

        let paragraph = Paragraph::new(help_text)
            .block(block)
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Left);

        paragraph.render(area, buf);
    }
}
