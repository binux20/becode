//! Input bar widget

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::themes::Theme;

/// Render the input bar
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    input: &str,
    cursor_pos: usize,
) {
    let block = Block::default()
        .title(" Input ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    // Build input line with cursor
    let before_cursor = &input[..cursor_pos.min(input.len())];
    let cursor_char = input.chars().nth(cursor_pos).unwrap_or(' ');
    let after_cursor = if cursor_pos < input.len() {
        &input[cursor_pos + cursor_char.len_utf8()..]
    } else {
        ""
    };

    let line = Line::from(vec![
        Span::styled("> ", theme.primary_style().add_modifier(Modifier::BOLD)),
        Span::raw(before_cursor),
        Span::styled(
            cursor_char.to_string(),
            Style::default()
                .fg(Theme::parse_color(&theme.colors.background))
                .bg(Theme::parse_color(&theme.colors.primary)),
        ),
        Span::raw(after_cursor),
    ]);

    let paragraph = Paragraph::new(line).block(block);

    frame.render_widget(paragraph, area);
}
