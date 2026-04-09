//! Google Gemini provider

use super::traits::{
    Attachment, ContentPart, LLMProvider, LLMResponse, LLMToolCall,
    Message, MessageContent, MessageRole, ProviderError, StreamChunk, Usage,
};
use crate::tools::ToolSpec;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::pin::Pin;
use tokio_stream::Stream;

pub struct GeminiProvider {
    api_key: String,
    client: Client,
}

impl GeminiProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            client: Client::new(),
        }
    }

    fn api_url(&self, model: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            model, self.api_key
        )
    }

    fn convert_messages(&self, messages: &[Message]) -> Vec<ApiContent> {
        messages.iter().filter_map(|msg| {
            if msg.role == MessageRole::System {
                return None;
            }

            let role = match msg.role {
                MessageRole::User => "user",
                MessageRole::Assistant => "model",
                MessageRole::Tool => "function",
                MessageRole::System => return None,
            };

            let parts: Vec<Part> = match &msg.content {
                MessageContent::Text(text) => vec![Part::Text { text: text.clone() }],
                MessageContent::Parts(parts) => {
                    parts.iter().map(|p| match p {
                        ContentPart::Text { text } => Part::Text { text: text.clone() },
                        ContentPart::ImageUrl { image_url } => Part::InlineData {
                            inline_data: InlineData {
                                mime_type: "image/png".to_string(),
                                data: image_url.url.clone(),
                            },
                        },
                    }).collect()
                }
            };

            Some(ApiContent {
                role: role.to_string(),
                parts,
            })
        }).collect()
    }

    fn convert_tools(&self, tools: &[ToolSpec]) -> Vec<ApiTool> {
        vec![ApiTool {
            function_declarations: tools.iter().map(|t| FunctionDeclaration {
                name: t.name.clone(),
                description: t.description.clone(),
                parameters: t.input_schema.clone(),
            }).collect(),
        }]
    }

    fn extract_system(&self, messages: &[Message]) -> Option<SystemInstruction> {
        messages.iter()
            .find(|m| m.role == MessageRole::System)
            .and_then(|m| m.content.as_text())
            .map(|text| SystemInstruction {
                parts: vec![Part::Text { text: text.to_string() }],
            })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiRequest {
    contents: Vec<ApiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<SystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<ApiTool>>,
    generation_config: GenerationConfig,
}

#[derive(Debug, Serialize)]
struct SystemInstruction {
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
struct ApiContent {
    role: String,
    parts: Vec<Part>,
}

#[derive(Debug, Serialize)]
#[serde(untagged)]
enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
    FunctionCall { function_call: FunctionCall },
    FunctionResponse { function_response: FunctionResponse },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct InlineData {
    mime_type: String,
    data: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionCall {
    name: String,
    args: Value,
}

#[derive(Debug, Serialize)]
struct FunctionResponse {
    name: String,
    response: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiTool {
    function_declarations: Vec<FunctionDeclaration>,
}

#[derive(Debug, Serialize)]
struct FunctionDeclaration {
    name: String,
    description: String,
    parameters: Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GenerationConfig {
    max_output_tokens: u32,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ApiResponse {
    candidates: Option<Vec<Candidate>>,
    usage_metadata: Option<UsageMetadata>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Candidate {
    content: CandidateContent,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CandidateContent {
    parts: Vec<ResponsePart>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ResponsePart {
    Text { text: String },
    FunctionCall { function_call: FunctionCall },
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

#[async_trait]
impl LLMProvider for GeminiProvider {
    fn name(&self) -> &str {
        "gemini"
    }

    fn default_model(&self) -> &str {
        "gemini-2.0-flash"
    }

    fn supports_tools(&self) -> bool {
        true
    }

    fn supports_streaming(&self) -> bool {
        true
    }

    fn supports_vision(&self) -> bool {
        true
    }

    async fn chat(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        _attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<LLMResponse, ProviderError> {
        let model = model.unwrap_or(self.default_model());
        let contents = self.convert_messages(messages);
        let system_instruction = self.extract_system(messages);
        let api_tools = tools.filter(|t| !t.is_empty()).map(|t| self.convert_tools(t));

        let request = ApiRequest {
            contents,
            system_instruction,
            tools: api_tools,
            generation_config: GenerationConfig {
                max_output_tokens: 4096,
            },
        };

        let response = self.client
            .post(&self.api_url(model))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| ProviderError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(ProviderError::ApiError { status, message: text });
        }

        let api_response: ApiResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::ParseError(e.to_string()))?;

        let candidate = api_response.candidates
            .and_then(|c| c.into_iter().next())
            .ok_or_else(|| ProviderError::InvalidResponse("No candidates".to_string()))?;

        let mut content = String::new();
        let mut tool_calls = Vec::new();

        for part in candidate.content.parts {
            match part {
                ResponsePart::Text { text } => content.push_str(&text),
                ResponsePart::FunctionCall { function_call } => {
                    tool_calls.push(LLMToolCall {
                        id: format!("call_{}", uuid::Uuid::new_v4()),
                        name: function_call.name,
                        arguments: function_call.args,
                    });
                }
            }
        }

        let usage = api_response.usage_metadata.map(|u| Usage {
            prompt_tokens: u.prompt_token_count,
            completion_tokens: u.candidates_token_count,
            total_tokens: u.total_token_count,
        });

        Ok(LLMResponse {
            content,
            tool_calls,
            finish_reason: candidate.finish_reason,
            usage,
        })
    }

    async fn chat_stream(
        &self,
        messages: &[Message],
        tools: Option<&[ToolSpec]>,
        attachments: Option<&[Attachment]>,
        model: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = StreamChunk> + Send>>, ProviderError> {
        let response = self.chat(messages, tools, attachments, model).await?;
        let chunks = vec![
            StreamChunk::Text(response.content),
            StreamChunk::Done,
        ];
        Ok(Box::pin(tokio_stream::iter(chunks)))
    }
}
