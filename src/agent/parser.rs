//! Response parsing for LLM outputs
//!
//! Handles both native tool calling and JSON block parsing for no-tools providers.

use crate::providers::LLMResponse;
use crate::tools::ToolCall;
use regex::Regex;
use serde_json::Value;

/// Parsed response from LLM
#[derive(Debug)]
pub enum ParsedResponse {
    /// Tool calls to execute
    ToolCalls(Vec<ToolCall>),
    /// Final answer / done
    Done(String),
    /// Plain text response (may need more tool calls)
    Text(String),
    /// Invalid/unparseable response
    Invalid(String),
}

/// Parser for LLM responses
pub struct ResponseParser {
    /// Whether the provider supports native tool calling
    native_tools: bool,
}

impl ResponseParser {
    pub fn new(native_tools: bool) -> Self {
        Self { native_tools }
    }

    /// Parse an LLM response
    pub fn parse(&self, response: &LLMResponse) -> ParsedResponse {
        if self.native_tools {
            self.parse_native(response)
        } else {
            self.parse_json_blocks(&response.content)
        }
    }

    /// Parse response with native tool calling
    fn parse_native(&self, response: &LLMResponse) -> ParsedResponse {
        // Check for tool calls
        if !response.tool_calls.is_empty() {
            let calls: Vec<ToolCall> = response
                .tool_calls
                .iter()
                .map(|tc| ToolCall {
                    tool: tc.name.clone(),
                    args: tc.arguments.clone(),
                    id: Some(tc.id.clone()),
                })
                .collect();
            return ParsedResponse::ToolCalls(calls);
        }

        // Check finish reason
        if response.finish_reason.as_deref() == Some("stop") {
            return ParsedResponse::Done(response.content.clone());
        }

        // Plain text
        if !response.content.is_empty() {
            ParsedResponse::Text(response.content.clone())
        } else {
            ParsedResponse::Invalid("Empty response".to_string())
        }
    }

    /// Parse response with JSON blocks (for no-tools providers)
    fn parse_json_blocks(&self, content: &str) -> ParsedResponse {
        let mut calls = Vec::new();

        // Pattern 1: ```json blocks
        let json_block_re = Regex::new(r"```json\s*\n([\s\S]*?)\n```").unwrap();
        for cap in json_block_re.captures_iter(content) {
            if let Some(json_str) = cap.get(1) {
                if let Some(parsed) = self.parse_json_object(json_str.as_str()) {
                    match parsed {
                        JsonParsed::ToolCall(call) => calls.push(call),
                        JsonParsed::Done(msg) => {
                            return ParsedResponse::Done(msg);
                        }
                    }
                }
            }
        }

        // Pattern 2: ``` blocks without language tag
        if calls.is_empty() {
            let code_block_re = Regex::new(r"```\s*\n([\s\S]*?)\n```").unwrap();
            for cap in code_block_re.captures_iter(content) {
                if let Some(json_str) = cap.get(1) {
                    if let Some(parsed) = self.parse_json_object(json_str.as_str()) {
                        match parsed {
                            JsonParsed::ToolCall(call) => calls.push(call),
                            JsonParsed::Done(msg) => {
                                return ParsedResponse::Done(msg);
                            }
                        }
                    }
                }
            }
        }

        // Pattern 3: Inline JSON (fallback)
        if calls.is_empty() {
            // Look for {"tool": ...} patterns
            let inline_re = Regex::new(r#"\{\s*"tool"\s*:\s*"[^"]+"[^}]*\}"#).unwrap();
            for mat in inline_re.find_iter(content) {
                if let Some(parsed) = self.parse_json_object(mat.as_str()) {
                    if let JsonParsed::ToolCall(call) = parsed {
                        calls.push(call);
                    }
                }
            }
        }

        if !calls.is_empty() {
            ParsedResponse::ToolCalls(calls)
        } else if content.trim().is_empty() {
            ParsedResponse::Invalid("Empty response".to_string())
        } else {
            // No JSON blocks found - treat as text
            // This could be a final answer or just commentary
            ParsedResponse::Text(content.to_string())
        }
    }

    /// Parse a single JSON object
    fn parse_json_object(&self, json_str: &str) -> Option<JsonParsed> {
        let json_str = json_str.trim();
        let value: Value = serde_json::from_str(json_str).ok()?;

        // Check for "done" signal
        if value.get("done").and_then(|v| v.as_bool()).unwrap_or(false) {
            let message = value
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Task completed")
                .to_string();
            return Some(JsonParsed::Done(message));
        }

        // Check for tool call
        let tool_name = value.get("tool").and_then(|v| v.as_str())?;

        // Handle __done__ special tool
        if tool_name == "__done__" {
            let message = value
                .get("args")
                .and_then(|a| a.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Task completed")
                .to_string();
            return Some(JsonParsed::Done(message));
        }

        let args = value.get("args").cloned().unwrap_or(serde_json::json!({}));

        Some(JsonParsed::ToolCall(ToolCall {
            tool: tool_name.to_string(),
            args,
            id: Some(uuid::Uuid::new_v4().to_string()),
        }))
    }
}

enum JsonParsed {
    ToolCall(ToolCall),
    Done(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::LLMToolCall;

    #[test]
    fn test_parse_native_tool_calls() {
        let parser = ResponseParser::new(true);

        let response = LLMResponse {
            content: "Let me read that file.".to_string(),
            tool_calls: vec![LLMToolCall {
                id: "call_123".to_string(),
                name: "read_file".to_string(),
                arguments: serde_json::json!({"path": "src/main.rs"}),
            }],
            finish_reason: Some("tool_calls".to_string()),
            usage: None,
        };

        match parser.parse(&response) {
            ParsedResponse::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].tool, "read_file");
            }
            _ => panic!("Expected ToolCalls"),
        }
    }

    #[test]
    fn test_parse_json_blocks() {
        let parser = ResponseParser::new(false);

        let content = r#"
Let me read the file.

```json
{"tool": "read_file", "args": {"path": "src/main.rs"}}
```
"#;

        let response = LLMResponse {
            content: content.to_string(),
            tool_calls: vec![],
            finish_reason: None,
            usage: None,
        };

        match parser.parse(&response) {
            ParsedResponse::ToolCalls(calls) => {
                assert_eq!(calls.len(), 1);
                assert_eq!(calls[0].tool, "read_file");
            }
            _ => panic!("Expected ToolCalls"),
        }
    }

    #[test]
    fn test_parse_multiple_json_blocks() {
        let parser = ResponseParser::new(false);

        let content = r#"
Reading multiple files.

```json
{"tool": "read_file", "args": {"path": "src/main.rs"}}
```

```json
{"tool": "read_file", "args": {"path": "Cargo.toml"}}
```
"#;

        let response = LLMResponse {
            content: content.to_string(),
            tool_calls: vec![],
            finish_reason: None,
            usage: None,
        };

        match parser.parse(&response) {
            ParsedResponse::ToolCalls(calls) => {
                assert_eq!(calls.len(), 2);
            }
            _ => panic!("Expected ToolCalls"),
        }
    }

    #[test]
    fn test_parse_done_signal() {
        let parser = ResponseParser::new(false);

        let content = r#"
Task completed!

```json
{"done": true, "message": "Successfully fixed the bug"}
```
"#;

        let response = LLMResponse {
            content: content.to_string(),
            tool_calls: vec![],
            finish_reason: None,
            usage: None,
        };

        match parser.parse(&response) {
            ParsedResponse::Done(msg) => {
                assert_eq!(msg, "Successfully fixed the bug");
            }
            _ => panic!("Expected Done"),
        }
    }

    #[test]
    fn test_parse_plain_text() {
        let parser = ResponseParser::new(false);

        let content = "This is just a plain text response without any JSON blocks.";

        let response = LLMResponse {
            content: content.to_string(),
            tool_calls: vec![],
            finish_reason: None,
            usage: None,
        };

        match parser.parse(&response) {
            ParsedResponse::Text(text) => {
                assert!(text.contains("plain text"));
            }
            _ => panic!("Expected Text"),
        }
    }
}
