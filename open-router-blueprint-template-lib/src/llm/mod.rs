//! LLM interface abstraction layer for the OpenRouter Blueprint
//!
//! This module defines the core interfaces and types for interacting with
//! locally hosted LLMs through the Tangle network.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod models;
pub use models::*;

mod local_llm;
pub use local_llm::*;

mod streaming;
pub use streaming::*;

/// Errors that can occur when interacting with an LLM
#[derive(Debug, Error)]
pub enum LlmError {
    #[error("LLM request failed: {0}")]
    RequestFailed(String),
    
    #[error("Model not supported: {0}")]
    ModelNotSupported(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    
    #[error("LLM client not initialized")]
    ClientNotInitialized,
    
    #[error("Operation timed out after {0:?}")]
    Timeout(Duration),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Result type for LLM operations
pub type Result<T> = std::result::Result<T, LlmError>;

/// Trait for LLM clients
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Get information about the supported models
    fn get_supported_models(&self) -> Vec<ModelInfo>;
    
    /// Get the capabilities of this LLM client
    fn get_capabilities(&self) -> LlmCapabilities;
    
    /// Get current metrics for this LLM client
    fn get_metrics(&self) -> NodeMetrics;
    
    /// Process a chat completion request
    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse>;
    
    /// Process a text completion request
    async fn text_completion(&self, request: TextCompletionRequest) -> Result<TextCompletionResponse>;
    
    /// Process an embedding request
    async fn embeddings(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse>;
}

/// Information about a specific LLM model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Unique identifier for the model
    pub id: String,
    
    /// Human-readable name of the model
    pub name: String,
    
    /// Maximum context length supported by the model
    pub max_context_length: usize,
    
    /// Whether the model supports chat completions
    pub supports_chat: bool,
    
    /// Whether the model supports text completions
    pub supports_text: bool,
    
    /// Whether the model supports embeddings
    pub supports_embeddings: bool,
    
    /// Additional model-specific parameters
    pub parameters: HashMap<String, String>,
}

/// Capabilities of an LLM client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmCapabilities {
    /// Whether the client supports streaming responses
    pub supports_streaming: bool,
    
    /// Maximum number of concurrent requests supported
    pub max_concurrent_requests: usize,
    
    /// Whether the client supports batching requests
    pub supports_batching: bool,
    
    /// Additional capability flags
    pub features: HashMap<String, bool>,
}

/// Metrics for an LLM node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    /// Current CPU utilization (0.0 - 1.0)
    pub cpu_utilization: f32,
    
    /// Current memory utilization (0.0 - 1.0)
    pub memory_utilization: f32,
    
    /// Current GPU utilization if available (0.0 - 1.0)
    pub gpu_utilization: Option<f32>,
    
    /// Number of requests processed in the last minute
    pub requests_per_minute: u32,
    
    /// Average response time in milliseconds
    pub average_response_time_ms: u64,
    
    /// Number of requests currently being processed
    pub active_requests: u32,
    
    /// Timestamp of the last update (Unix timestamp in seconds)
    pub last_updated: u64,
}

/// Trait for LLM clients that support streaming responses
#[async_trait::async_trait]
pub trait StreamingLlmClient: LlmClient {
    /// Process a streaming chat completion request
    async fn streaming_chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionStream>;
    
    /// Process a streaming text completion request
    async fn streaming_text_completion(&self, request: TextCompletionRequest) -> Result<TextCompletionStream>;
}

/// Extension trait for checking if an LlmClient also implements StreamingLlmClient
pub trait LlmClientExt {
    /// Check if this client supports streaming
    fn supports_streaming(&self) -> bool;
    
    /// Try to get this client as a StreamingLlmClient
    fn as_streaming(&self) -> Option<&dyn StreamingLlmClient>;
    
    /// Process a chat completion request (extension method)
    async fn chat_completion_ext(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse>;
    
    /// Process a text completion request (extension method)
    async fn text_completion_ext(&self, request: TextCompletionRequest) -> Result<TextCompletionResponse>;
    
    /// Process an embedding request (extension method)
    async fn embeddings_ext(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse>;
}

impl<T: LlmClient + 'static> LlmClientExt for T {
    fn supports_streaming(&self) -> bool {
        // Check if the client's capabilities indicate streaming support
        self.get_capabilities().supports_streaming
    }
    
    fn as_streaming(&self) -> Option<&dyn StreamingLlmClient> {
        // If the client doesn't claim to support streaming, don't even try
        if !self.supports_streaming() {
            return None;
        }
        
        // We can't directly downcast to dyn StreamingLlmClient because it doesn't have a known size
        // Instead, we'll return None for now and implement a different approach
        None // Placeholder - proper downcasting would require a different approach
    }
    
    async fn chat_completion_ext(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        self.chat_completion(request).await
    }
    
    async fn text_completion_ext(&self, request: TextCompletionRequest) -> Result<TextCompletionResponse> {
        self.text_completion(request).await
    }
    
    async fn embeddings_ext(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.embeddings(request).await
    }
}

impl LlmClientExt for std::sync::Arc<dyn LlmClient> {
    fn supports_streaming(&self) -> bool {
        // Check if the client's capabilities indicate streaming support
        self.get_capabilities().supports_streaming
    }
    
    fn as_streaming(&self) -> Option<&dyn StreamingLlmClient> {
        // If the client doesn't claim to support streaming, don't even try
        if !self.supports_streaming() {
            return None;
        }
        
        // For Arc<dyn LlmClient>, we need a different approach
        // This is a simplification that assumes if it supports streaming,
        // we can try to use it as a streaming client
        None // This is a placeholder - we'll need to implement proper downcasting
    }
    
    async fn chat_completion_ext(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        self.chat_completion(request).await
    }
    
    async fn text_completion_ext(&self, request: TextCompletionRequest) -> Result<TextCompletionResponse> {
        self.text_completion(request).await
    }
    
    async fn embeddings_ext(&self, request: EmbeddingRequest) -> Result<EmbeddingResponse> {
        self.embeddings(request).await
    }
}
