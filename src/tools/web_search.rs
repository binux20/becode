//! web_search tool - Search the web
//!
//! Uses a search API to find relevant information.

use super::traits::{Tool, ToolContext, ToolError, ToolOutput};
use crate::permissions::Permission;
use async_trait::async_trait;
use serde_json::{json, Value};

/// Default number of results
const DEFAULT_NUM_RESULTS: u32 = 5;

/// Tool for web search
pub struct WebSearchTool;

#[async_trait]
impl Tool for WebSearchTool {
    fn name(&self) -> &'static str {
        "web_search"
    }

    fn description(&self) -> &'static str {
        "Search the web for information. Returns titles, URLs, and snippets."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query"
                },
                "num_results": {
                    "type": "integer",
                    "description": "Number of results to return",
                    "default": 5
                }
            },
            "required": ["query"]
        })
    }

    fn required_permission(&self) -> Permission {
        Permission::ReadOnly
    }

    async fn execute(&self, input: Value, _ctx: &ToolContext) -> Result<ToolOutput, ToolError> {
        let query = input["query"]
            .as_str()
            .ok_or_else(|| ToolError::InvalidInput {
                reason: "query is required".to_string(),
            })?;

        let num_results = input["num_results"]
            .as_u64()
            .unwrap_or(DEFAULT_NUM_RESULTS as u64) as u32;

        // Try different search backends
        let results = search_duckduckgo(query, num_results).await?;

        let result = json!({
            "query": query,
            "results": results,
            "total_results": results.len()
        });

        Ok(ToolOutput::success(self.name(), result, 0))
    }
}

/// Search using DuckDuckGo Instant Answer API
async fn search_duckduckgo(query: &str, _num_results: u32) -> Result<Vec<Value>, ToolError> {
    let client = reqwest::Client::new();

    // DuckDuckGo Instant Answer API
    let url = format!(
        "https://api.duckduckgo.com/?q={}&format=json&no_html=1&skip_disambig=1",
        urlencoding::encode(query)
    );

    let response = client
        .get(&url)
        .header("User-Agent", "BeCode/2.0")
        .send()
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    let data: Value = response
        .json()
        .await
        .map_err(|e| ToolError::Http(e.to_string()))?;

    let mut results = Vec::new();

    // Extract abstract
    if let Some(abstract_text) = data["Abstract"].as_str() {
        if !abstract_text.is_empty() {
            results.push(json!({
                "title": data["Heading"].as_str().unwrap_or("Result"),
                "url": data["AbstractURL"].as_str().unwrap_or(""),
                "snippet": abstract_text,
                "source": data["AbstractSource"].as_str().unwrap_or("")
            }));
        }
    }

    // Extract related topics
    if let Some(topics) = data["RelatedTopics"].as_array() {
        for topic in topics.iter().take(5) {
            if let Some(text) = topic["Text"].as_str() {
                results.push(json!({
                    "title": topic["FirstURL"].as_str()
                        .map(|u| u.split('/').last().unwrap_or("").replace('_', " "))
                        .unwrap_or_default(),
                    "url": topic["FirstURL"].as_str().unwrap_or(""),
                    "snippet": text,
                    "source": "DuckDuckGo"
                }));
            }
        }
    }

    // If no results from DDG, return a message
    if results.is_empty() {
        results.push(json!({
            "title": "No direct results",
            "url": format!("https://duckduckgo.com/?q={}", urlencoding::encode(query)),
            "snippet": "Try searching directly or rephrase your query.",
            "source": "DuckDuckGo"
        }));
    }

    Ok(results)
}

// URL encoding helper
mod urlencoding {
    pub fn encode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push('+'),
                _ => {
                    for b in c.to_string().as_bytes() {
                        result.push_str(&format!("%{:02X}", b));
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_encoding() {
        assert_eq!(urlencoding::encode("hello world"), "hello+world");
        assert_eq!(urlencoding::encode("rust lang"), "rust+lang");
    }

    #[tokio::test]
    #[ignore] // Requires network
    async fn test_web_search() {
        let tool = WebSearchTool;
        let ctx = create_test_context();

        let input = json!({
            "query": "Rust programming language"
        });

        let result = tool.execute(input, &ctx).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert!(output.success);
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
