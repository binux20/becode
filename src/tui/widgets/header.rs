//! Header widget with logo and status

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::themes::Theme;

/// Render the header
pub fn render(frame: &mut Frame, area: Rect, theme: &Theme, provider: &str, model: Option<&str>) {
    let title = vec![
        Span::styled("🐝 ", Style::default()),
        Span::styled(
            "BeCode",
            Style::default()
                .fg(Theme::parse_color(&theme.colors.primary))
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" v2.0.0", theme.muted_style()),
    ];

    let model_display = model.unwrap_or("(default)");
    let status = vec![
        Span::styled(provider, theme.secondary_style()),
        Span::styled("/", theme.muted_style()),
        Span::styled(model_display, theme.secondary_style()),
    ];

    // Calculate spacing
    let title_len: usize = 15; // approximate
    let status_len = provider.len() + 1 + model_display.len();
    let spacing = area.width as usize - title_len - status_len - 4;
    let spaces = " ".repeat(spacing.max(1));

    let header_line = Line::from(vec![
        title.into_iter().collect::<Vec<_>>(),
        vec![Span::raw(spaces)],
        status,
    ].concat());

    let block = Block::default()
        .borders(Borders::BOTTOM)
        .border_style(theme.border_style());

    let paragraph = Paragraph::new(header_line).block(block);

    frame.render_widget(paragraph, area);
}
