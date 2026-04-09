//! web_fetch tool - Fetch URL contents

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

/// Default timeout in seconds
const DEFAULT_TIMEOUT_SECS: u64 = 30;

/// Max response size (5MB)
const MAX_RESPONSE_SIZE: usize = 5 * 1024 * 1024;

/// Tool for fetching URL contents
pub struct WebFetchTool;

#[async_trait]
impl Tool for WebFetchTool {
    fn name(&self) -> &'static str {
        "web_fetch"
    }

    fn description(&self) -> &'static str {
        "Fetch content from a URL. Use for downloading documentation, API responses, etc."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "url": {
                    "type": "string",
                    "description": "URL to fetch"
                },
                "method": {
                    "type": "string",
                    "enum": ["GET", "POST", "PUT", "DELETE"],
                    "description": "HTTP method",
                    "default": "GET"
                },
                "headers": {
                    "type": "object",
                    "description": "Additional HTTP headers",
                    "additionalProperties": { "type": "string" }
                },
                "body": {
                    "type": "string",
                    "description": "Request body (for POST/PUT)"
                },
                "timeout_secs": {
                    "type": "integer",
                    "description": "Timeout in seconds",
                    "default": 30
                }
            },
            "required": ["url"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::ReadOnly
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let url = input["url"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "url is required".to_string(),
            })?;

        let method = input["method"].as_str().unwrap_or("GET");
        let timeout_secs = input["timeout_secs"]
            .as_u64()
            .unwrap_or(DEFAULT_TIMEOUT_SECS);

        // Parse headers
        let headers: HashMap<String, String> = input["headers"]
            .as_object()
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let body = input["body"].as_str().map(|s| s.to_string());

        // Validate URL
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(ToolError::InvalidInput {
                reason: "URL must start with http:// or https://".to_string(),
            });
        }

        // Build request
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .build()
            .map_err(|e| ToolError::Http(e.to_string()))?;

        let mut request = match method.to_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            _ => {
                return Err(ToolError::InvalidInput {
                    reason: format!("Unsupported HTTP method: {}", method),
                })
            }
        };

        // Add headers
        for (key, value) in &headers {
            request = request.header(key.as_str(), value.as_str());
        }

        // Add body if present
        if let Some(body_content) = body {
            request = request.body(body_content);
        }

        // Execute request
        let start = std::time::Instant::now();
        let response = request.send().await.map_err(|e| ToolError::Http(e.to_string()))?;

        let status = response.status().as_u16();
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.to_string(), s.to_string())))
            .collect();

        // Read body with size limit
        let content_length = response.content_length().unwrap_or(0) as usize;
        if content_length > MAX_RESPONSE_SIZE {
            return Err(ToolError::ExecutionFailed {
                reason: format!(
                    "Response too large: {} bytes (max: {} bytes)",
                    content_length, MAX_RESPONSE_SIZE
                ),
            });
        }

        let body_bytes = response
            .bytes()
            .await
            .map_err(|e| ToolError::Http(e.to_string()))?;

        if body_bytes.len() > MAX_RESPONSE_SIZE {
            return Err(ToolError::ExecutionFailed {
                reason: format!(
                    "Response too large: {} bytes (max: {} bytes)",
                    body_bytes.len(),
                    MAX_RESPONSE_SIZE
                ),
            });
        }

        // Try to decode as text
        let body_text = String::from_utf8_lossy(&body_bytes).to_string();
        let fetch_time_ms = start.elapsed().as_millis() as u32;

        let result = json!({
            "url": url,
            "status": status,
            "headers": response_headers,
            "body": body_text,
            "bytes": body_bytes.len(),
            "fetch_time_ms": fetch_time_ms
        });

        Ok(ToolOutput::success(self.name(), result, fetch_time_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests require network access
    // In CI, they might be skipped or use mocks

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_web_fetch_get() {
        let tool = WebFetchTool;
        let ctx = create_test_context();

        let input = json!({
            "url": "https://httpbin.org/get"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
        assert_eq!(output.result["status"], 200);
    }

    fn create_test_context() -> ToolContext {
        use crate::permissions::PermissionEnforcer;
        use std::env;
        use std::sync::Arc;

        let workspace = env::current_dir().unwrap();
        let enforcer = Arc::new(PermissionEnforcer::new(
            Permission::ReadOnly,
            workspace.clone(),
        ));

        ToolContext {
            workspace_root: workspace,
            permission: Permission::ReadOnly,
            enforcer,
        }
    }
}
