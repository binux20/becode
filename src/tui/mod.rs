//! TUI (Terminal User Interface) for BeCode
//!
//! Beautiful terminal interface with bee theme.

mod app;
mod events;
mod themes;
mod mascot;
mod widgets;

pub use app::{App, ChatMessage, MessageRole, ToolCallDisplay, ToolStatus, PanelFocus};

use crate::config::Config;
use crate::permissions::Permission;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::time::Duration;

/// TUI settings passed from CLI
pub struct TuiSettings {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub permission: String,
}

// Bee theme colors
const BEE_YELLOW: Color = Color::Rgb(255, 200, 0);
const BEE_ORANGE: Color = Color::Rgb(255, 140, 0);
const BEE_DARK: Color = Color::Rgb(30, 30, 30);
const HONEY_GOLD: Color = Color::Rgb(218, 165, 32);

/// Run the TUI application
pub async fn run_tui(settings: &TuiSettings, config: &Config) -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Parse permission
    let permission = match settings.permission.as_str() {
        "read-only" => Permission::ReadOnly,
        "danger" => Permission::DangerFullAccess,
        _ => Permission::WorkspaceWrite,
    };

    // Create app
    let mut app = App::new(config, settings.provider.clone(), settings.model.clone(), permission);

    // Main loop
    let result = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

async fn run_app<B: ratatui::backend::Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        // Poll for events with timeout
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match (key.modifiers, key.code) {
                    // Quit
                    (KeyModifiers::NONE, KeyCode::Esc) => {
                        app.running = false;
                        break;
                    }
                    (KeyModifiers::CONTROL, KeyCode::Char('c')) => {
                        app.running = false;
                        break;
                    }
                    // Submit message
                    (KeyModifiers::NONE, KeyCode::Enter) => {
                        if !app.input.is_empty() && !app.thinking {
                            let input = app.input.clone();
                            app.input.clear();
                            app.cursor_pos = 0;
                            app.add_user_message(input.clone());
                            app.thinking = true;
                            app.status = "Thinking...".to_string();

                            // TODO: Actually run the agent here
                            // For now, just echo
                            app.add_assistant_message(format!("You said: {}", input));
                            app.thinking = false;
                            app.status = "Ready".to_string();
                            app.scroll_to_bottom();
                        }
                    }
                    // Input editing
                    (KeyModifiers::NONE, KeyCode::Backspace) => {
                        if app.cursor_pos > 0 {
                            app.cursor_pos -= 1;
                            app.input.remove(app.cursor_pos);
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Delete) => {
                        if app.cursor_pos < app.input.len() {
                            app.input.remove(app.cursor_pos);
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Left) => {
                        if app.cursor_pos > 0 {
                            app.cursor_pos -= 1;
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Right) => {
                        if app.cursor_pos < app.input.len() {
                            app.cursor_pos += 1;
                        }
                    }
                    (KeyModifiers::NONE, KeyCode::Home) => {
                        app.cursor_pos = 0;
                    }
                    (KeyModifiers::NONE, KeyCode::End) => {
                        app.cursor_pos = app.input.len();
                    }
                    // Scroll
                    (KeyModifiers::NONE, KeyCode::PageUp) => {
                        app.scroll_up();
                    }
                    (KeyModifiers::NONE, KeyCode::PageDown) => {
                        app.scroll_down();
                    }
                    // Clear
                    (KeyModifiers::CONTROL, KeyCode::Char('l')) => {
                        app.messages.clear();
                        app.messages.push(ChatMessage {
                            role: MessageRole::System,
                            content: "Chat cleared. Ready for new task.".to_string(),
                            tool_calls: vec![],
                            timestamp: chrono::Local::now(),
                        });
                    }
                    // Character input
                    (KeyModifiers::NONE | KeyModifiers::SHIFT, KeyCode::Char(c)) => {
                        app.input.insert(app.cursor_pos, c);
                        app.cursor_pos += 1;
                    }
                    _ => {}
                }
            }
        }

        if !app.running {
            break;
        }
    }

    Ok(())
}

fn draw_ui(f: &mut Frame, app: &App) {
    let size = f.area();

    // Main layout: header, content, input, status
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(10),    // Chat
            Constraint::Length(3),  // Input
            Constraint::Length(1),  // Status
        ])
        .split(size);

    draw_header(f, chunks[0], app);
    draw_chat(f, chunks[1], app);
    draw_input(f, chunks[2], app);
    draw_status(f, chunks[3], app);
}

fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let title = vec![
        Span::styled(" B", Style::default().fg(BEE_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("ee", Style::default().fg(BEE_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled("C", Style::default().fg(BEE_YELLOW).add_modifier(Modifier::BOLD)),
        Span::styled("ode ", Style::default().fg(BEE_ORANGE).add_modifier(Modifier::BOLD)),
        Span::styled("| ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.provider_name, Style::default().fg(Color::Cyan)),
        Span::styled(" / ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            app.model.as_deref().unwrap_or("default"),
            Style::default().fg(Color::Green),
        ),
    ];

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(HONEY_GOLD))
        .title(Line::from(title));

    let project_info = format!(" {} ", app.project.display());
    let inner = Paragraph::new(project_info)
        .style(Style::default().fg(Color::Gray))
        .block(block);

    f.render_widget(inner, area);
}

fn draw_chat(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.thinking { BEE_YELLOW } else { Color::DarkGray }))
        .title(if app.thinking {
            Span::styled(" Thinking... ", Style::default().fg(BEE_YELLOW).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(" Chat ", Style::default().fg(Color::White))
        });

    let inner_area = block.inner(area);
    f.render_widget(block, area);

    // Build chat lines
    let mut lines: Vec<Line> = vec![];

    for msg in &app.messages {
        let (prefix, style) = match msg.role {
            MessageRole::User => ("You: ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            MessageRole::Assistant => ("Bee: ", Style::default().fg(BEE_YELLOW).add_modifier(Modifier::BOLD)),
            MessageRole::System => (">> ", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC)),
        };

        lines.push(Line::from(vec![
            Span::styled(prefix, style),
            Span::raw(&msg.content),
        ]));

        // Show tool calls
        for tc in &msg.tool_calls {
            let status_icon = match &tc.status {
                ToolStatus::Pending => "...",
                ToolStatus::Running => ">>>",
                ToolStatus::Success => "OK ",
                ToolStatus::Error(_) => "ERR",
            };
            let status_color = match &tc.status {
                ToolStatus::Pending => Color::Gray,
                ToolStatus::Running => BEE_YELLOW,
                ToolStatus::Success => Color::Green,
                ToolStatus::Error(_) => Color::Red,
            };

            lines.push(Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("[{}] ", status_icon), Style::default().fg(status_color)),
                Span::styled(&tc.tool, Style::default().fg(Color::Magenta)),
                Span::raw(": "),
                Span::styled(&tc.args_preview, Style::default().fg(Color::DarkGray)),
            ]));

            if let Some(ref output) = tc.output_preview {
                let preview = if output.len() > 60 {
                    format!("{}...", &output[..60])
                } else {
                    output.clone()
                };
                lines.push(Line::from(vec![
                    Span::raw("        "),
                    Span::styled(preview, Style::default().fg(Color::DarkGray)),
                ]));
            }
        }

        lines.push(Line::from(""));
    }

    let paragraph = Paragraph::new(lines)
        .wrap(Wrap { trim: false })
        .scroll((app.scroll_offset as u16, 0));

    f.render_widget(paragraph, inner_area);
}

fn draw_input(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.focus == PanelFocus::Input { BEE_YELLOW } else { Color::DarkGray }))
        .title(Span::styled(" Input ", Style::default().fg(Color::White)));

    let input_text = if app.input.is_empty() {
        Span::styled("Type your message...", Style::default().fg(Color::DarkGray))
    } else {
        Span::raw(&app.input)
    };

    let input = Paragraph::new(input_text).block(block);
    f.render_widget(input, area);

    // Show cursor
    if app.focus == PanelFocus::Input {
        f.set_cursor_position((
            area.x + 1 + app.cursor_pos as u16,
            area.y + 1,
        ));
    }
}

fn draw_status(f: &mut Frame, area: Rect, app: &App) {
    let status = Line::from(vec![
        Span::styled(" [Esc] ", Style::default().fg(Color::DarkGray)),
        Span::styled("Quit", Style::default().fg(Color::Gray)),
        Span::styled(" | [Enter] ", Style::default().fg(Color::DarkGray)),
        Span::styled("Send", Style::default().fg(Color::Gray)),
        Span::styled(" | [Ctrl+L] ", Style::default().fg(Color::DarkGray)),
        Span::styled("Clear", Style::default().fg(Color::Gray)),
        Span::styled(" | ", Style::default().fg(Color::DarkGray)),
        Span::styled(&app.status, Style::default().fg(if app.thinking { BEE_YELLOW } else { Color::Green })),
    ]);

    let paragraph = Paragraph::new(status);
    f.render_widget(paragraph, area);
}
