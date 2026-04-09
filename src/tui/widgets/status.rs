//! Status bar widget

use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::tui::themes::Theme;

/// Render the status bar
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    thinking: bool,
    tokens: Option<u32>,
) {
    let mut spans = vec![
        Span::styled("[F1]", theme.muted_style()),
        Span::raw(" Help  "),
        Span::styled("[F2]", theme.muted_style()),
        Span::raw(" Provider  "),
        Span::styled("[F3]", theme.muted_style()),
        Span::raw(" Model  "),
        Span::styled("[F4]", theme.muted_style()),
        Span::raw(" Project  "),
        Span::styled("[Ctrl+L]", theme.muted_style()),
        Span::raw(" Clear  "),
        Span::styled("[Esc]", theme.muted_style()),
        Span::raw(" Quit"),
    ];

    // Add thinking indicator
    if thinking {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            "🐝 Thinking...",
            theme.primary_style().add_modifier(Modifier::BOLD),
        ));
    }

    // Add token count if available
    if let Some(count) = tokens {
        // Calculate position for right alignment
        let token_text = format!("  Tokens: {}", count);
        spans.push(Span::styled(token_text, theme.muted_style()));
    }

    let line = Line::from(spans);
    let paragraph = Paragraph::new(line);

    frame.render_widget(paragraph, area);
}
