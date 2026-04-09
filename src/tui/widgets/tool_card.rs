//! Tool call card widget

use ratatui::{
    layout::Rect,
    style::Modifier,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui::themes::Theme;

/// Tool call status
#[derive(Debug, Clone)]
pub enum ToolStatus {
    Running,
    Success { duration_ms: u32 },
    Error { message: String },
}

/// Render a tool call card
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    tool_name: &str,
    status: &ToolStatus,
    diff: Option<&str>,
) {
    let (status_icon, status_style) = match status {
        ToolStatus::Running => ("🔄", theme.warning_style()),
        ToolStatus::Success { .. } => ("✅", theme.success_style()),
        ToolStatus::Error { .. } => ("❌", theme.error_style()),
    };

    let title = format!(" {} {} ", status_icon, tool_name);

    let block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let mut lines: Vec<Line> = Vec::new();

    // Status line
    match status {
        ToolStatus::Running => {
            lines.push(Line::from(Span::styled("Running...", status_style)));
        }
        ToolStatus::Success { duration_ms } => {
            lines.push(Line::from(Span::styled(
                format!("Completed in {}ms", duration_ms),
                status_style,
            )));
        }
        ToolStatus::Error { message } => {
            lines.push(Line::from(Span::styled(
                format!("Error: {}", message),
                status_style,
            )));
        }
    }

    // Diff preview
    if let Some(diff_text) = diff {
        lines.push(Line::from(""));
        for line in diff_text.lines().take(6) {
            let style = if line.starts_with('+') && !line.starts_with("+++") {
                theme.success_style()
            } else if line.starts_with('-') && !line.starts_with("---") {
                theme.error_style()
            } else if line.starts_with("@@") {
                theme.secondary_style()
            } else {
                theme.muted_style()
            };
            lines.push(Line::from(Span::styled(line, style)));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
