use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A chat message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// The role of the message sender (e.g., "system", "user", "assistant")
    pub role: String,

    /// The content of the message
    pub content: String,

    /// Optional name of the sender
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Request for a chat completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// The model to use for completion
    pub model: String,

    /// The messages to generate a completion for
    pub messages: Vec<ChatMessage>,

    /// The maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// The sampling temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// The nucleus sampling parameter (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Additional model-specific parameters
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub additional_params: HashMap<String, serde_json::Value>,
}

/// A chat completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// The index of this choice
    pub index: usize,

    /// The generated message
    pub message: ChatMessage,

    /// The reason the generation stopped
    pub finish_reason: Option<String>,
}

/// Response from a chat completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// The ID of the completion
    pub id: String,

    /// The type of object (always "chat.completion")
    pub object: String,

    /// The timestamp of the completion (Unix timestamp in seconds)
    pub created: u64,

    /// The model used for the completion
    pub model: String,

    /// The generated choices
    pub choices: Vec<ChatCompletionChoice>,

    /// Usage statistics for the completion
    pub usage: Option<UsageInfo>,
}

/// Request for a text completion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCompletionRequest {
    /// The model to use for completion
    pub model: String,

    /// The prompt to generate a completion for
    pub prompt: String,

    /// The maximum number of tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// The sampling temperature (0.0 - 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// The nucleus sampling parameter (0.0 - 1.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Whether to stream the response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Additional model-specific parameters
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub additional_params: HashMap<String, serde_json::Value>,
}

/// A text completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCompletionChoice {
    /// The index of this choice
    pub index: usize,

    /// The generated text
    pub text: String,

    /// The reason the generation stopped
    pub finish_reason: Option<String>,
}

/// Response from a text completion request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextCompletionResponse {
    /// The ID of the completion
    pub id: String,

    /// The type of object (always "text_completion")
    pub object: String,

    /// The timestamp of the completion (Unix timestamp in seconds)
    pub created: u64,

    /// The model used for the completion
    pub model: String,

    /// The generated choices
    pub choices: Vec<TextCompletionChoice>,

    /// Usage statistics for the completion
    pub usage: Option<UsageInfo>,
}

/// Request for generating embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingRequest {
    /// The model to use for embeddings
    pub model: String,

    /// The input to generate embeddings for (either a string or array of strings)
    pub input: Vec<String>,

    /// Additional model-specific parameters
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub additional_params: HashMap<String, serde_json::Value>,
}

/// A single embedding result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingData {
    /// The index of this embedding
    pub index: usize,

    /// The embedding vector
    pub embedding: Vec<f32>,
}

/// Response from an embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// The type of object (always "list")
    pub object: String,

    /// The model used for the embeddings
    pub model: String,

    /// The generated embeddings
    pub data: Vec<EmbeddingData>,

    /// Usage statistics for the embeddings
    pub usage: Option<UsageInfo>,
}

/// Usage information for an LLM request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// The number of prompt tokens used
    pub prompt_tokens: u32,

    /// The number of completion tokens used
    pub completion_tokens: u32,

    /// The total number of tokens used
    pub total_tokens: u32,
}

/// A unified request type that can represent any LLM operation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmRequest {
    #[serde(rename = "chat.completion")]
    ChatCompletion(ChatCompletionRequest),

    #[serde(rename = "text.completion")]
    TextCompletion(TextCompletionRequest),

    #[serde(rename = "embedding")]
    Embedding(EmbeddingRequest),
}

/// A unified response type that can represent any LLM operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum LlmResponse {
    #[serde(rename = "chat.completion")]
    ChatCompletion(ChatCompletionResponse),

    #[serde(rename = "text.completion")]
    TextCompletion(TextCompletionResponse),

    #[serde(rename = "embedding")]
    Embedding(EmbeddingResponse),
}
