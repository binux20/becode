//! Session management commands

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub message_count: usize,
    pub project_path: Option<String>,
    pub provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub name: String,
    pub created_at: String,
    pub updated_at: String,
    pub project_path: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub messages: Vec<SessionMessage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub tool_calls: Vec<serde_json::Value>,
}

fn get_sessions_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".becode")
        .join("sessions")
}

/// List all saved sessions
#[tauri::command]
pub async fn list_sessions() -> Result<Vec<SessionMetadata>, String> {
    let sessions_dir = get_sessions_dir();

    if !sessions_dir.exists() {
        return Ok(vec![]);
    }

    let mut sessions: Vec<SessionMetadata> = vec![];

    let entries = std::fs::read_dir(&sessions_dir)
        .map_err(|e| format!("Failed to read sessions directory: {}", e))?;

    for entry in entries.filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.extension().map_or(false, |ext| ext == "json") {
            if let Ok(content) = std::fs::read_to_string(&path) {
                if let Ok(session) = serde_json::from_str::<Session>(&content) {
                    sessions.push(SessionMetadata {
                        id: session.id,
                        name: session.name,
                        created_at: session.created_at,
                        updated_at: session.updated_at,
                        message_count: session.messages.len(),
                        project_path: session.project_path,
                        provider: session.provider,
                    });
                }
            }
        }
    }

    // Sort by updated_at descending
    sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

    Ok(sessions)
}

/// Load a specific session
#[tauri::command]
pub async fn load_session(id: String) -> Result<Session, String> {
    let session_path = get_sessions_dir().join(format!("{}.json", id));

    if !session_path.exists() {
        return Err(format!("Session not found: {}", id));
    }

    let content = std::fs::read_to_string(&session_path)
        .map_err(|e| format!("Failed to read session: {}", e))?;

    serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse session: {}", e))
}

/// Save current session
#[tauri::command]
pub async fn save_session(
    name: String,
    messages: Vec<SessionMessage>,
    project_path: Option<String>,
    provider: Option<String>,
    model: Option<String>,
) -> Result<String, String> {
    let sessions_dir = get_sessions_dir();

    // Ensure directory exists
    std::fs::create_dir_all(&sessions_dir)
        .map_err(|e| format!("Failed to create sessions directory: {}", e))?;

    let id = uuid::Uuid::new_v4().to_string();
    let now = chrono::Local::now().to_rfc3339();

    let session = Session {
        id: id.clone(),
        name,
        created_at: now.clone(),
        updated_at: now,
        project_path,
        provider,
        model,
        messages,
    };

    let session_path = sessions_dir.join(format!("{}.json", id));
    let content = serde_json::to_string_pretty(&session)
        .map_err(|e| format!("Failed to serialize session: {}", e))?;

    std::fs::write(&session_path, content)
        .map_err(|e| format!("Failed to write session: {}", e))?;

    Ok(id)
}

/// Delete a session
#[tauri::command]
pub async fn delete_session(id: String) -> Result<(), String> {
    let session_path = get_sessions_dir().join(format!("{}.json", id));

    if !session_path.exists() {
        return Err(format!("Session not found: {}", id));
    }

    std::fs::remove_file(&session_path)
        .map_err(|e| format!("Failed to delete session: {}", e))
}

/// Export session to markdown
#[tauri::command]
pub async fn export_session(id: String, format: String) -> Result<String, String> {
    let session = load_session(id).await?;

    match format.as_str() {
        "markdown" | "md" => {
            let mut md = format!("# {}\n\n", session.name);
            md.push_str(&format!("**Created:** {}\n\n", session.created_at));

            if let Some(project) = &session.project_path {
                md.push_str(&format!("**Project:** `{}`\n\n", project));
            }

            if let Some(provider) = &session.provider {
                md.push_str(&format!("**Provider:** {}\n\n", provider));
            }

            md.push_str("---\n\n");

            for msg in &session.messages {
                let role_emoji = match msg.role.as_str() {
                    "user" => "👤 **You**",
                    "assistant" => "🐝 **BeCode**",
                    "system" => "⚙️ **System**",
                    _ => "💬",
                };

                md.push_str(&format!("### {}\n\n", role_emoji));
                md.push_str(&msg.content);
                md.push_str("\n\n");

                if !msg.tool_calls.is_empty() {
                    md.push_str("<details>\n<summary>Tool Calls</summary>\n\n");
                    for tc in &msg.tool_calls {
                        md.push_str(&format!("```json\n{}\n```\n\n",
                            serde_json::to_string_pretty(tc).unwrap_or_default()));
                    }
                    md.push_str("</details>\n\n");
                }

                md.push_str("---\n\n");
            }

            Ok(md)
        }
        "json" => {
            serde_json::to_string_pretty(&session)
                .map_err(|e| format!("Failed to serialize: {}", e))
        }
        _ => Err(format!("Unsupported format: {}", format)),
    }
}
