//! Chat panel widget

use ratatui::{
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::{ChatMessage, MessageRole};
use crate::tui::themes::Theme;

/// Render the chat panel
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    messages: &[ChatMessage],
    scroll: u16,
) {
    let block = Block::default()
        .title(" Chat ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let inner = block.inner(area);

    // Build lines from messages
    let mut lines: Vec<Line> = Vec::new();

    for msg in messages {
        // Role indicator
        let (prefix, style) = match msg.role {
            MessageRole::User => (
                "👤 You: ",
                Style::default()
                    .fg(Theme::parse_color(&theme.colors.secondary))
                    .add_modifier(Modifier::BOLD),
            ),
            MessageRole::Assistant => (
                "🐝 BeCode: ",
                Style::default()
                    .fg(Theme::parse_color(&theme.colors.primary))
                    .add_modifier(Modifier::BOLD),
            ),
            MessageRole::System => (
                "⚙️ System: ",
                theme.muted_style(),
            ),
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
        ]));

        // Message content (wrap long lines)
        for line in msg.content.lines() {
            lines.push(Line::from(Span::raw(format!("   {}", line))));
        }

        // Tool calls
        for tc in &msg.tool_calls {
            let status_icon = match &tc.status {
                crate::tui::app::ToolStatus::Pending => "⏳",
                crate::tui::app::ToolStatus::Running => "🔄",
                crate::tui::app::ToolStatus::Success => "✅",
                crate::tui::app::ToolStatus::Error(_) => "❌",
            };

            lines.push(Line::from(vec![
                Span::raw("   "),
                Span::styled(
                    format!("┌─ {} {} ", status_icon, tc.tool),
                    theme.secondary_style(),
                ),
            ]));

            if let Some(ref preview) = tc.output_preview {
                for line in preview.lines().take(3) {
                    lines.push(Line::from(vec![
                        Span::raw("   │ "),
                        Span::styled(line, theme.muted_style()),
                    ]));
                }
            }

            lines.push(Line::from(Span::styled("   └───────────", theme.muted_style())));
        }

        // Empty line between messages
        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    frame.render_widget(paragraph, area);
}
