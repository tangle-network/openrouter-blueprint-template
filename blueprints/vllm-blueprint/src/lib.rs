//! vLLM Blueprint: Implements the generic LLM client for vLLM via REST API.
//! This crate depends on the open-router-blueprint-template-lib for all core traits and types.

use async_trait::async_trait;
use open_router_blueprint_template_lib::llm::{
    ChatCompletionRequest, ChatCompletionResponse, LlmClient, LlmError, ModelInfo, NodeMetrics,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

pub struct VllmLlmClient {
    pub api_url: String,
    pub model: String,
    pub metrics: Arc<RwLock<NodeMetrics>>,
    pub http_client: Client,
}

impl VllmLlmClient {
    pub fn new(api_url: String, model: String) -> Self {
        info!(
            "Creating new VllmLlmClient with API URL: {} and model: {}",
            api_url, model
        );
        Self {
            api_url,
            model,
            metrics: Arc::new(RwLock::new(NodeMetrics {
                cpu_utilization: 0.0,
                memory_utilization: 0.0,
                gpu_utilization: None,
                requests_per_minute: 0,
                average_response_time_ms: 0,
                active_requests: 0,
                last_updated: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
            })),
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl LlmClient for VllmLlmClient {
    fn get_supported_models(&self) -> Vec<ModelInfo> {
        debug!("Checking if model '{}' exists in vLLM", self.model);
        // Check if the model exists in vLLM
        let valid_model = futures::executor::block_on(async {
            let url = format!("{}/v1/models", self.api_url);
            trace!("Sending request to {}", url);
            let res = self.http_client.get(&url).send().await;
            if let Ok(response) = res {
                if response.status().is_success() {
                    #[derive(Deserialize)]
                    struct VllmModelsResponse {
                        data: Vec<VllmModel>,
                    }

                    #[derive(Deserialize)]
                    struct VllmModel {
                        id: String,
                    }

                    if let Ok(models) = response.json::<VllmModelsResponse>().await {
                        let is_valid = models.data.iter().any(|m| m.id == self.model);
                        debug!("Model '{}' validation result: {}", self.model, is_valid);
                        return is_valid;
                    }
                }
            } else if let Err(e) = res {
                warn!("Failed to get vLLM models: {}", e);
            }
            debug!(
                "Model validation failed, assuming model '{}' is invalid",
                self.model
            );
            false
        });

        if !valid_model {
            // Return empty list for unsupported model
            warn!(
                "Model '{}' is not available in vLLM, returning empty model list",
                self.model
            );
            return vec![];
        }

        info!("Model '{}' is available in vLLM", self.model);
        vec![ModelInfo {
            id: self.model.clone(),
            name: self.model.clone(),
            max_context_length: 4096, // Default value, could be model-specific
            supports_chat: true,
            supports_text: true,
            supports_embeddings: false, // vLLM may not support embeddings in all versions
            parameters: Default::default(),
        }]
    }

    fn get_capabilities(&self) -> open_router_blueprint_template_lib::llm::LlmCapabilities {
        open_router_blueprint_template_lib::llm::LlmCapabilities {
            supports_streaming: true, // vLLM supports streaming
            max_concurrent_requests: 4, // vLLM can handle multiple concurrent requests
            supports_batching: true,   // vLLM supports batching
            features: Default::default(),
        }
    }

    fn get_metrics(&self) -> NodeMetrics {
        futures::executor::block_on(async { self.metrics.read().await.clone() })
    }

    async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse, LlmError> {
        info!(
            "Processing chat completion request for model: {}",
            request.model
        );

        // Check if the requested model is supported
        let supported_models = self.get_supported_models();
        let model_supported = supported_models.iter().any(|m| m.id == request.model);

        if !model_supported {
            error!(
                "Model '{}' is not available in vLLM for chat completion",
                request.model
            );
            return Err(LlmError::ModelNotSupported(format!(
                "Model '{}' is not available in vLLM",
                request.model
            )));
        }

        // Build vLLM API request
        #[derive(Serialize)]
        struct VllmChatMessage {
            role: String,
            content: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
        }

        #[derive(Serialize)]
        struct VllmChatRequest {
            model: String,
            messages: Vec<VllmChatMessage>,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stream: Option<bool>,
        }

        let vllm_messages = request
            .messages
            .iter()
            .map(|m| VllmChatMessage {
                role: m.role.clone(),
                content: m.content.clone(),
                name: m.name.clone(),
            })
            .collect();

        let vllm_request = VllmChatRequest {
            model: request.model.clone(),
            messages: vllm_messages,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
        };

        // Send request to vLLM API
        let url = format!("{}/v1/chat/completions", self.api_url);
        debug!("Sending chat completion request to {}", url);

        let res = self
            .http_client
            .post(&url)
            .json(&vllm_request)
            .send()
            .await;

        // Parse response
        let response = match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    #[derive(Deserialize)]
                    struct VllmChatResponseMessage {
                        role: String,
                        content: String,
                        #[serde(default)]
                        name: Option<String>,
                    }

                    #[derive(Deserialize)]
                    struct VllmChatResponseChoice {
                        index: usize,
                        message: VllmChatResponseMessage,
                        finish_reason: Option<String>,
                    }

                    #[derive(Deserialize)]
                    struct VllmUsage {
                        prompt_tokens: u32,
                        completion_tokens: u32,
                        total_tokens: u32,
                    }

                    #[derive(Deserialize)]
                    struct VllmChatResponse {
                        id: String,
                        object: String,
                        created: u64,
                        model: String,
                        choices: Vec<VllmChatResponseChoice>,
                        usage: Option<VllmUsage>,
                    }

                    match resp.json::<VllmChatResponse>().await {
                        Ok(vllm_resp) => {
                            let choices = vllm_resp
                                .choices
                                .into_iter()
                                .map(|c| {
                                    open_router_blueprint_template_lib::llm::ChatCompletionChoice {
                                        index: c.index,
                                        message: open_router_blueprint_template_lib::llm::ChatMessage {
                                            role: c.message.role,
                                            content: c.message.content,
                                            name: c.message.name,
                                        },
                                        finish_reason: c.finish_reason,
                                    }
                                })
                                .collect();

                            let usage = vllm_resp.usage.map(|u| {
                                open_router_blueprint_template_lib::llm::UsageInfo {
                                    prompt_tokens: u.prompt_tokens,
                                    completion_tokens: u.completion_tokens,
                                    total_tokens: u.total_tokens,
                                }
                            });

                            Ok(ChatCompletionResponse {
                                id: vllm_resp.id,
                                object: vllm_resp.object,
                                created: vllm_resp.created,
                                model: vllm_resp.model,
                                choices,
                                usage,
                            })
                        }
                        Err(e) => {
                            error!("Failed to parse vLLM response: {}", e);
                            Err(LlmError::RequestFailed(format!(
                                "Failed to parse vLLM response: {}",
                                e
                            )))
                        }
                    }
                } else {
                    // Try to parse error response
                    #[derive(Deserialize)]
                    struct VllmErrorResponse {
                        error: VllmError,
                    }

                    #[derive(Deserialize)]
                    struct VllmError {
                        message: String,
                        #[serde(rename = "type")]
                        error_type: String,
                    }

                    match resp.json::<VllmErrorResponse>().await {
                        Ok(error_resp) => {
                            error!(
                                "vLLM API error: {} ({})",
                                error_resp.error.message, error_resp.error.error_type
                            );
                            Err(LlmError::RequestFailed(format!(
                                "vLLM API error: {} ({})",
                                error_resp.error.message, error_resp.error.error_type
                            )))
                        }
                        Err(_) => {
                            error!("vLLM API error: {}", resp.status());
                            Err(LlmError::RequestFailed(format!(
                                "vLLM API error: {}",
                                resp.status()
                            )))
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to send request to vLLM API: {}", e);
                Err(LlmError::RequestFailed(format!(
                    "Failed to send request to vLLM API: {}",
                    e
                )))
            }
        };

        info!("Completed chat completion request");
        response
    }

    async fn text_completion(
        &self,
        request: open_router_blueprint_template_lib::llm::TextCompletionRequest,
    ) -> Result<open_router_blueprint_template_lib::llm::TextCompletionResponse, LlmError> {
        info!(
            "Processing text completion request for model: {}",
            request.model
        );

        // Check if the requested model is supported
        let supported_models = self.get_supported_models();
        let model_supported = supported_models.iter().any(|m| m.id == request.model);

        if !model_supported {
            error!(
                "Model '{}' is not available in vLLM for text completion",
                request.model
            );
            return Err(LlmError::ModelNotSupported(format!(
                "Model '{}' is not available in vLLM",
                request.model
            )));
        }

        // Build vLLM API request
        #[derive(Serialize)]
        struct VllmCompletionRequest {
            model: String,
            prompt: String,
            #[serde(skip_serializing_if = "Option::is_none")]
            max_tokens: Option<u32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            temperature: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            top_p: Option<f32>,
            #[serde(skip_serializing_if = "Option::is_none")]
            stream: Option<bool>,
        }

        let vllm_request = VllmCompletionRequest {
            model: request.model.clone(),
            prompt: request.prompt,
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            top_p: request.top_p,
            stream: request.stream,
        };

        // Send request to vLLM API
        let url = format!("{}/v1/completions", self.api_url);
        debug!("Sending text completion request to {}", url);

        let res = self
            .http_client
            .post(&url)
            .json(&vllm_request)
            .send()
            .await;

        // Parse response
        let response = match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    #[derive(Deserialize)]
                    struct VllmCompletionChoice {
                        index: usize,
                        text: String,
                        finish_reason: Option<String>,
                    }

                    #[derive(Deserialize)]
                    struct VllmUsage {
                        prompt_tokens: u32,
                        completion_tokens: u32,
                        total_tokens: u32,
                    }

                    #[derive(Deserialize)]
                    struct VllmCompletionResponse {
                        id: String,
                        object: String,
                        created: u64,
                        model: String,
                        choices: Vec<VllmCompletionChoice>,
                        usage: Option<VllmUsage>,
                    }

                    match resp.json::<VllmCompletionResponse>().await {
                        Ok(vllm_resp) => {
                            let choices = vllm_resp
                                .choices
                                .into_iter()
                                .map(|c| {
                                    open_router_blueprint_template_lib::llm::TextCompletionChoice {
                                        index: c.index,
                                        text: c.text,
                                        finish_reason: c.finish_reason,
                                    }
                                })
                                .collect();

                            let usage = vllm_resp.usage.map(|u| {
                                open_router_blueprint_template_lib::llm::UsageInfo {
                                    prompt_tokens: u.prompt_tokens,
                                    completion_tokens: u.completion_tokens,
                                    total_tokens: u.total_tokens,
                                }
                            });

                            Ok(open_router_blueprint_template_lib::llm::TextCompletionResponse {
                                id: vllm_resp.id,
                                object: vllm_resp.object,
                                created: vllm_resp.created,
                                model: vllm_resp.model,
                                choices,
                                usage,
                            })
                        }
                        Err(e) => {
                            error!("Failed to parse vLLM response: {}", e);
                            Err(LlmError::RequestFailed(format!(
                                "Failed to parse vLLM response: {}",
                                e
                            )))
                        }
                    }
                } else {
                    // Try to parse error response
                    #[derive(Deserialize)]
                    struct VllmErrorResponse {
                        error: VllmError,
                    }

                    #[derive(Deserialize)]
                    struct VllmError {
                        message: String,
                        #[serde(rename = "type")]
                        error_type: String,
                    }

                    match resp.json::<VllmErrorResponse>().await {
                        Ok(error_resp) => {
                            error!(
                                "vLLM API error: {} ({})",
                                error_resp.error.message, error_resp.error.error_type
                            );
                            Err(LlmError::RequestFailed(format!(
                                "vLLM API error: {} ({})",
                                error_resp.error.message, error_resp.error.error_type
                            )))
                        }
                        Err(_) => {
                            error!("vLLM API error: {}", resp.status());
                            Err(LlmError::RequestFailed(format!(
                                "vLLM API error: {}",
                                resp.status()
                            )))
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to send request to vLLM API: {}", e);
                Err(LlmError::RequestFailed(format!(
                    "Failed to send request to vLLM API: {}",
                    e
                )))
            }
        };

        info!("Completed text completion request");
        response
    }

    async fn embeddings(
        &self,
        request: open_router_blueprint_template_lib::llm::EmbeddingRequest,
    ) -> Result<open_router_blueprint_template_lib::llm::EmbeddingResponse, LlmError> {
        info!("Processing embedding request for model: {}", request.model);
        
        // Check if the requested model is supported
        let supported_models = self.get_supported_models();
        let model_supported = supported_models.iter().any(|m| m.id == request.model);

        if !model_supported {
            error!(
                "Model '{}' is not available in vLLM for embeddings",
                request.model
            );
            return Err(LlmError::ModelNotSupported(format!(
                "Model '{}' is not available in vLLM",
                request.model
            )));
        }

        // vLLM may not support embeddings in all versions
        warn!("Embeddings are not implemented in this vLLM blueprint example");

        Err(LlmError::NotImplemented(
            "vLLM embeddings not implemented in this example".to_string(),
        ))
    }
}
