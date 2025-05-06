use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::{TangleArg, TangleResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::context::OpenRouterContext;
use crate::llm::{LlmClientExt, LlmRequest, LlmResponse, StreamingLlmClient};

/// Job ID for processing LLM requests
pub const PROCESS_LLM_REQUEST_JOB_ID: u32 = 0;

/// Job ID for reporting metrics
pub const REPORT_METRICS_JOB_ID: u32 = 1;

/// Process an LLM request
///
/// This job handler receives an LLM request from Tangle, processes it
/// through the selected LLM node, and returns the response.
///
/// # ASCII Diagram
/// ```
/// User -> OpenRouter -> Tangle -> Blueprint -> Load Balancer -> LLM Node
///                                    |
///                                    v
/// User <- OpenRouter <- Tangle <- Blueprint <- LLM Node
/// ```
///
/// # Expected Outcome
/// The request is processed by the selected LLM node and the response is returned to Tangle.
pub async fn process_llm_request(
    Context(ctx): Context<OpenRouterContext>,
    TangleArg(request): TangleArg<LlmRequest>,
) -> Result<TangleResult<LlmResponse>, blueprint_sdk::Error> {
    info!("Processing LLM request");

    // Get the model name from the request
    let model = match &request {
        LlmRequest::ChatCompletion(req) => &req.model,
        LlmRequest::TextCompletion(req) => &req.model,
        LlmRequest::Embedding(req) => &req.model,
    };

    // Select an LLM client for this model using the load balancer
    let llm_client = match ctx.get_llm_client_for_model(model).await {
        Some(client) => client,
        None => {
            // Fall back to the default client if no suitable node is found
            warn!(
                "No suitable LLM node found for model {}, using default client",
                model
            );
            ctx.llm_client.clone()
        }
    };

    // Check if streaming is requested
    let streaming = match &request {
        LlmRequest::ChatCompletion(req) => req.stream.unwrap_or(false),
        LlmRequest::TextCompletion(req) => req.stream.unwrap_or(false),
        LlmRequest::Embedding(_) => false,
    };

    // Process the request based on its type
    let response = if streaming {
        // Handle streaming requests if the client supports it
        match request {
            LlmRequest::ChatCompletion(req) => {
                debug!(
                    "Processing streaming chat completion request for model: {}",
                    req.model
                );

                // Try to get a streaming client
                if let Some(streaming_client) = llm_client.as_streaming() {
                    // Use the streaming client
                    let stream = streaming_client
                        .streaming_chat_completion(req)
                        .await
                        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;

                    // Collect the stream into a single response
                    let chat_response = crate::llm::collect_chat_completion_stream(stream)
                        .await
                        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;

                    LlmResponse::ChatCompletion(chat_response)
                } else {
                    // Fall back to non-streaming if the client doesn't support streaming
                    warn!("Selected LLM client doesn't support streaming, falling back to non-streaming");
                    let chat_response = llm_client
                        .chat_completion_ext(req)
                        .await
                        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;
                    LlmResponse::ChatCompletion(chat_response)
                }
            }
            LlmRequest::TextCompletion(req) => {
                debug!(
                    "Processing streaming text completion request for model: {}",
                    req.model
                );

                // Try to get a streaming client
                if let Some(streaming_client) = llm_client.as_streaming() {
                    // Use the streaming client
                    let stream = streaming_client
                        .streaming_text_completion(req)
                        .await
                        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;

                    // Collect the stream into a single response
                    let text_response = crate::llm::collect_text_completion_stream(stream)
                        .await
                        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;

                    LlmResponse::TextCompletion(text_response)
                } else {
                    // Fall back to non-streaming if the client doesn't support streaming
                    warn!("Selected LLM client doesn't support streaming, falling back to non-streaming");
                    let text_response = llm_client
                        .text_completion_ext(req)
                        .await
                        .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;
                    LlmResponse::TextCompletion(text_response)
                }
            }
            LlmRequest::Embedding(req) => {
                debug!("Processing embedding request for model: {}", req.model);
                let embedding_response = llm_client
                    .embeddings_ext(req)
                    .await
                    .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;
                LlmResponse::Embedding(embedding_response)
            }
        }
    } else {
        // Handle non-streaming requests
        match request {
            LlmRequest::ChatCompletion(req) => {
                debug!(
                    "Processing chat completion request for model: {}",
                    req.model
                );
                let chat_response = llm_client
                    .chat_completion_ext(req)
                    .await
                    .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;
                LlmResponse::ChatCompletion(chat_response)
            }
            LlmRequest::TextCompletion(req) => {
                debug!(
                    "Processing text completion request for model: {}",
                    req.model
                );
                let text_response = llm_client
                    .text_completion_ext(req)
                    .await
                    .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;
                LlmResponse::TextCompletion(text_response)
            }
            LlmRequest::Embedding(req) => {
                debug!("Processing embedding request for model: {}", req.model);
                let embedding_response = llm_client
                    .embeddings_ext(req)
                    .await
                    .map_err(|e| blueprint_sdk::Error::Other(e.to_string()))?;
                LlmResponse::Embedding(embedding_response)
            }
        }
    };

    // Update metrics after processing the request
    ctx.update_metrics().await;

    info!("LLM request processed successfully");
    Ok(TangleResult(response))
}

/// Report metrics for this node
///
/// This job handler reports the current metrics for this node back to Tangle.
/// This allows Tangle to make informed load balancing decisions.
///
/// # ASCII Diagram
/// ```
/// Tangle -> Blueprint
///             |
///             v
/// Tangle <- Blueprint (metrics)
/// ```
///
/// # Expected Outcome
/// The current metrics for this node are reported back to Tangle.
pub async fn report_metrics(
    Context(ctx): Context<OpenRouterContext>,
) -> Result<TangleResult<crate::llm::NodeMetrics>, blueprint_sdk::Error> {
    info!("Reporting metrics");

    // Update metrics before reporting
    ctx.update_metrics().await;

    // Get the current metrics
    let metrics = ctx.metrics.read().await.clone();

    info!("Metrics reported successfully");
    Ok(TangleResult(metrics))
}
