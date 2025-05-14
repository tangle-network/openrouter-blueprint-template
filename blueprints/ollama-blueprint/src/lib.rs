use async_trait::async_trait;
use open_router_blueprint_template_lib::llm::{
    ChatCompletionRequest, ChatCompletionResponse, LlmClient, LlmError, ModelInfo, NodeMetrics,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, trace, warn};

pub struct OllamaLlmClient {
    pub api_url: String,
    pub model: String,
    pub metrics: Arc<RwLock<NodeMetrics>>,
    pub http_client: Client,
}

impl OllamaLlmClient {
    pub fn new(api_url: String, model: String) -> Self {
        info!(
            "Creating new OllamaLlmClient with API URL: {} and model: {}",
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
impl LlmClient for OllamaLlmClient {
    fn get_supported_models(&self) -> Vec<ModelInfo> {
        debug!("Checking if model '{}' exists in Ollama", self.model);
        // Check if the model exists in Ollama
        let valid_model = futures::executor::block_on(async {
            let url = format!("{}/api/tags", self.api_url);
            trace!("Sending request to {}", url);
            let res = self.http_client.get(&url).send().await;
            if let Ok(response) = res {
                if response.status().is_success() {
                    #[derive(Deserialize)]
                    struct OllamaModels {
                        models: Vec<OllamaModel>,
                    }

                    #[derive(Deserialize)]
                    struct OllamaModel {
                        name: String,
                    }

                    if let Ok(models) = response.json::<OllamaModels>().await {
                        let is_valid = models.models.iter().any(|m| m.name == self.model);
                        debug!("Model '{}' validation result: {}", self.model, is_valid);
                        return is_valid;
                    }
                }
            } else if let Err(e) = res {
                warn!("Failed to get Ollama models: {}", e);
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
                "Model '{}' is not available in Ollama, returning empty model list",
                self.model
            );
            return vec![];
        }

        info!("Model '{}' is available in Ollama", self.model);
        vec![ModelInfo {
            id: self.model.clone(),
            name: self.model.clone(),
            max_context_length: 4096,
            supports_chat: true,
            supports_text: true,
            supports_embeddings: false,
            parameters: Default::default(),
        }]
    }

    fn get_capabilities(&self) -> open_router_blueprint_template_lib::llm::LlmCapabilities {
        open_router_blueprint_template_lib::llm::LlmCapabilities {
            supports_streaming: false,
            max_concurrent_requests: 1,
            supports_batching: false,
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
            error!("Model '{}' is not available in Ollama", request.model);
            return Err(LlmError::ModelNotSupported(format!(
                "Model '{}' is not available in Ollama",
                request.model
            )));
        }

        debug!("Building Ollama API request for model: {}", self.model);
        // Build Ollama API request
        #[derive(Serialize)]
        struct OllamaRequest {
            model: String,
            prompt: String,
            stream: bool,
        }

        #[derive(Deserialize, Debug)]
        struct OllamaResponse {
            model: String,
            created_at: String,
            response: String,
            done: bool,
        }

        // Convert chat messages to a prompt string
        let prompt = request
            .messages
            .iter()
            .map(|m| format!("{}:\n{}", m.role, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        trace!(
            "Converted {} chat messages to prompt format",
            request.messages.len()
        );

        let ollama_req = OllamaRequest {
            model: self.model.clone(),
            prompt,
            stream: false,
        };

        // Ollama API endpoint
        let url = format!("{}/api/generate", self.api_url);
        debug!("Sending request to Ollama API: {}", url);

        // Send request to Ollama API
        let res = self.http_client.post(&url).json(&ollama_req).send().await;

        let res = match res {
            Ok(response) => response,
            Err(e) => {
                // Check if error message indicates model not found
                let err_msg = e.to_string();
                error!("Failed to send request to Ollama: {}", err_msg);

                if err_msg.contains("model not found") || err_msg.contains("failed to load model") {
                    return Err(LlmError::ModelNotSupported(format!(
                        "Model '{}' not found in Ollama",
                        &self.model
                    )));
                } else {
                    return Err(LlmError::RequestFailed(err_msg));
                }
            }
        };

        // Check response status code
        if !res.status().is_success() {
            // Try to parse error response
            #[derive(Deserialize, Debug)]
            struct OllamaErrorResponse {
                error: String,
            }

            let status = res.status();
            warn!("Ollama API returned non-success status: {}", status);

            let err_text = match res.text().await {
                Ok(text) => {
                    trace!("Error response body: {}", text);
                    text
                }
                Err(e) => {
                    warn!("Failed to read error response body: {}", e);
                    String::default()
                }
            };

            // Check for model not found errors
            if status.as_u16() == 404
                || status.as_u16() == 400
                || err_text.contains("model not found")
                || err_text.contains("failed to load")
            {
                error!("Model not supported error: {}", err_text);
                return Err(LlmError::ModelNotSupported(format!(
                    "Model '{}' not supported: {}",
                    &self.model, err_text
                )));
            }

            error!("Ollama API error ({}): {}", status, err_text);
            return Err(LlmError::RequestFailed(format!(
                "Ollama API error ({}): {}",
                status, err_text
            )));
        }

        debug!("Successfully received response from Ollama, parsing JSON");

        // Parse successful response
        let ollama_resp = match res.json::<OllamaResponse>().await {
            Ok(response) => response,
            Err(e) => {
                error!("Failed to parse Ollama response: {}", e);
                return Err(LlmError::RequestFailed(format!(
                    "Failed to parse Ollama response: {}",
                    e
                )));
            }
        };

        let response_id = uuid::Uuid::new_v4().to_string();
        info!(
            "Successfully completed chat request, returning response with ID: {}",
            response_id
        );

        Ok(ChatCompletionResponse {
            id: response_id,
            object: "chat.completion".to_string(),
            created: chrono::Utc::now().timestamp() as u64,
            model: ollama_resp.model,
            choices: vec![
                open_router_blueprint_template_lib::llm::ChatCompletionChoice {
                    index: 0,
                    message: open_router_blueprint_template_lib::llm::ChatMessage {
                        role: "assistant".to_string(),
                        name: None,
                        content: ollama_resp.response,
                    },
                    finish_reason: Some("stop".to_string()),
                },
            ],
            usage: None,
        })
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
                "Model '{}' is not available in Ollama for text completion",
                request.model
            );
            return Err(LlmError::ModelNotSupported(format!(
                "Model '{}' is not available in Ollama",
                request.model
            )));
        }

        debug!("Converting text completion request to chat completion format");
        // For Ollama, text and chat are equivalent.
        let chat_req = ChatCompletionRequest {
            model: request.model.clone(),
            messages: vec![open_router_blueprint_template_lib::llm::ChatMessage {
                role: "user".to_string(),
                name: None,
                content: request.prompt,
            }],
            max_tokens: None,
            temperature: None,
            top_p: None,
            stream: None,
            additional_params: std::collections::HashMap::new(),
        };

        trace!("Delegating to chat_completion method");
        let chat_resp = self.chat_completion(chat_req).await?;

        debug!("Converting chat completion response to text completion format");
        let response = open_router_blueprint_template_lib::llm::TextCompletionResponse {
            id: chat_resp.id,
            object: chat_resp.object,
            created: chat_resp.created,
            model: chat_resp.model,
            choices: vec![
                open_router_blueprint_template_lib::llm::TextCompletionChoice {
                    index: 0,
                    text: chat_resp.choices[0].message.content.clone(),
                    finish_reason: chat_resp.choices[0].finish_reason.clone(),
                },
            ],
            usage: None,
        };

        info!(
            "Successfully completed text completion request with ID: {}",
            response.id
        );
        Ok(response)
    }

    async fn embeddings(
        &self,
        request: open_router_blueprint_template_lib::llm::EmbeddingRequest,
    ) -> Result<open_router_blueprint_template_lib::llm::EmbeddingResponse, LlmError> {
        info!("Processing embedding request for model: {}", request.model);
        warn!("Embeddings are not implemented in this Ollama blueprint example");

        Err(LlmError::NotImplemented(
            "Ollama embeddings not implemented in this example".to_string(),
        ))
    }
}
