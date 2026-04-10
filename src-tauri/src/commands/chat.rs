//! Chat commands for AI interaction

use serde::{Deserialize, Serialize};
use tauri::{Emitter, Window};
use std::sync::atomic::{AtomicBool, Ordering};

/// Flag to cancel ongoing execution
static CANCEL_FLAG: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
    #[serde(default)]
    pub tool_calls: Vec<ToolCallInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub tool: String,
    pub args: serde_json::Value,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StreamChunk {
    pub chunk_type: String,
    pub content: Option<String>,
    pub tool_call: Option<ToolCallInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatResponse {
    pub success: bool,
    pub message: Option<ChatMessage>,
    pub error: Option<String>,
}

/// Send a message to the AI and stream the response
#[tauri::command]
pub async fn send_message(
    window: Window,
    message: String,
    provider: String,
    model: Option<String>,
    project_path: String,
) -> Result<ChatResponse, String> {
    CANCEL_FLAG.store(false, Ordering::SeqCst);

    // Emit that we're starting
    window.emit("chat-status", "thinking").map_err(|e| e.to_string())?;

    // TODO: Integrate with actual agent runtime
    // For now, simulate streaming response

    let response_id = uuid::Uuid::new_v4().to_string();

    // Simulate streaming chunks
    let demo_response = format!(
        "I received your message: \"{}\"\n\n\
        I'm using **{}** provider with model **{}**.\n\n\
        Project: `{}`\n\n\
        This is a demo response. The full agent integration is coming soon!",
        message,
        provider,
        model.as_deref().unwrap_or("default"),
        project_path
    );

    // Stream the response character by character (with some grouping for performance)
    let chars: Vec<char> = demo_response.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if CANCEL_FLAG.load(Ordering::SeqCst) {
            window.emit("chat-status", "cancelled").map_err(|e| e.to_string())?;
            return Ok(ChatResponse {
                success: false,
                message: None,
                error: Some("Cancelled by user".to_string()),
            });
        }

        // Send chunks of ~5-10 characters
        let chunk_size = std::cmp::min(8, chars.len() - i);
        let chunk: String = chars[i..i + chunk_size].iter().collect();

        window.emit("message-chunk", StreamChunk {
            chunk_type: "text".to_string(),
            content: Some(chunk),
            tool_call: None,
        }).map_err(|e| e.to_string())?;

        i += chunk_size;

        // Small delay for streaming effect
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }

    // Emit completion
    window.emit("message-done", ()).map_err(|e| e.to_string())?;
    window.emit("chat-status", "ready").map_err(|e| e.to_string())?;

    Ok(ChatResponse {
        success: true,
        message: Some(ChatMessage {
            id: response_id,
            role: "assistant".to_string(),
            content: demo_response,
            timestamp: chrono::Local::now().to_rfc3339(),
            tool_calls: vec![],
        }),
        error: None,
    })
}

/// Cancel the current execution
#[tauri::command]
pub async fn cancel_execution() -> Result<(), String> {
    CANCEL_FLAG.store(true, Ordering::SeqCst);
    Ok(())
}

/// Compact the chat context using a sub-agent
#[tauri::command]
pub async fn compact_context(
    window: Window,
    messages: Vec<ChatMessage>,
    keep_last: usize,
) -> Result<Vec<ChatMessage>, String> {
    window.emit("chat-status", "compacting").map_err(|e| e.to_string())?;

    if messages.len() <= keep_last {
        window.emit("chat-status", "ready").map_err(|e| e.to_string())?;
        return Ok(messages);
    }

    // Split messages: to compact vs to keep
    let split_point = messages.len().saturating_sub(keep_last);
    let to_compact = &messages[..split_point];
    let to_keep = &messages[split_point..];

    // TODO: Use actual sub-agent for summarization
    // For now, create a simple summary
    let summary = format!(
        "[Summary of {} previous messages: Discussion about coding tasks and file operations]",
        to_compact.len()
    );

    let mut result = vec![ChatMessage {
        id: uuid::Uuid::new_v4().to_string(),
        role: "system".to_string(),
        content: summary,
        timestamp: chrono::Local::now().to_rfc3339(),
        tool_calls: vec![],
    }];

    result.extend(to_keep.iter().cloned());

    window.emit("chat-status", "ready").map_err(|e| e.to_string())?;

    Ok(result)
}
