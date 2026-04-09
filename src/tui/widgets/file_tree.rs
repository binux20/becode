//! File tree panel widget

use ratatui::{
    layout::Rect,
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use std::path::Path;

use crate::tui::themes::Theme;

/// Render the file tree panel
pub fn render(
    frame: &mut Frame,
    area: Rect,
    theme: &Theme,
    project_path: &Path,
    expanded: bool,
) {
    if !expanded {
        return;
    }

    let block = Block::default()
        .title(" Project ")
        .borders(Borders::ALL)
        .border_style(theme.border_style());

    let mut lines = vec![
        Line::from(vec![
            Span::styled("📁 ", theme.primary_style()),
            Span::styled(
                project_path.display().to_string(),
                theme.secondary_style(),
            ),
        ]),
    ];

    // Try to list directory contents
    if let Ok(entries) = std::fs::read_dir(project_path) {
        let mut entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        entries.sort_by_key(|e| {
            let is_dir = e.path().is_dir();
            let name = e.file_name().to_string_lossy().to_lowercase();
            (!is_dir, name) // Directories first, then alphabetical
        });

        for (i, entry) in entries.iter().take(15).enumerate() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            // Skip hidden files and common ignored dirs
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }

            let is_last = i == entries.len().min(15) - 1;
            let prefix = if is_last { "└── " } else { "├── " };

            let (icon, style) = if path.is_dir() {
                ("📁", theme.secondary_style())
            } else {
                let icon = file_icon(&name);
                (icon, theme.muted_style())
            };

            lines.push(Line::from(vec![
                Span::raw(format!("   {}", prefix)),
                Span::raw(format!("{} ", icon)),
                Span::styled(name, style),
            ]));
        }

        if entries.len() > 15 {
            lines.push(Line::from(Span::styled(
                format!("   ... and {} more", entries.len() - 15),
                theme.muted_style(),
            )));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}

/// Get icon for file type
fn file_icon(name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("");
    match ext.to_lowercase().as_str() {
        "rs" => "🦀",
        "py" => "🐍",
        "js" | "jsx" => "📜",
        "ts" | "tsx" => "📘",
        "go" => "🐹",
        "rb" => "💎",
        "java" => "☕",
        "c" | "h" => "⚙️",
        "cpp" | "hpp" | "cc" => "⚙️",
        "md" => "📝",
        "json" => "📋",
        "toml" | "yaml" | "yml" => "⚙️",
        "html" => "🌐",
        "css" | "scss" | "sass" => "🎨",
        "sql" => "🗃️",
        "sh" | "bash" | "zsh" => "🐚",
        "dockerfile" => "🐳",
        "lock" => "🔒",
        "gitignore" => "🙈",
        _ => "📄",
    }
}
