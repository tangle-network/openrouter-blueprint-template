//! Job handlers for the OpenRouter Blueprint
//!
//! This module defines the job handlers for processing LLM requests
//! and reporting metrics.

use blueprint_sdk::extract::Context;
use blueprint_sdk::tangle::extract::{TangleArg, TangleResult};
use tracing::{debug, info, warn};

use crate::context::OpenRouterContext;
use crate::llm::{LlmRequest, LlmResponse};

/// Job ID for processing LLM requests
pub const PROCESS_LLM_REQUEST_JOB_ID: u32 = 0;

/// Job ID for reporting metrics
pub const REPORT_METRICS_JOB_ID: u32 = 1;

/// Process an LLM request
///
/// This job handler receives an LLM request from Tangle, processes it
/// through the local LLM, and returns the response.
///
/// # ASCII Diagram
/// ```
/// User -> OpenRouter -> Tangle -> Blueprint -> Local LLM
///                                    |
///                                    v
/// User <- OpenRouter <- Tangle <- Blueprint <- Local LLM
/// ```
///
/// # Expected Outcome
/// The request is processed by the local LLM and the response is returned to Tangle.
#[blueprint_sdk::debug_job]
pub async fn process_llm_request(
    Context(ctx): Context<OpenRouterContext>,
    TangleArg(request): TangleArg<LlmRequest>,
) -> blueprint_sdk::Result<TangleResult<LlmResponse>> {
    info!("Processing LLM request");
    
    // Process the request based on its type
    let response = match request {
        LlmRequest::ChatCompletion(req) => {
            debug!("Processing chat completion request for model: {}", req.model);
            let chat_response = ctx.llm_client.chat_completion(req).await
                .map_err(|e| blueprint_sdk::Error::Custom(e.to_string()))?;
            LlmResponse::ChatCompletion(chat_response)
        },
        LlmRequest::TextCompletion(req) => {
            debug!("Processing text completion request for model: {}", req.model);
            let text_response = ctx.llm_client.text_completion(req).await
                .map_err(|e| blueprint_sdk::Error::Custom(e.to_string()))?;
            LlmResponse::TextCompletion(text_response)
        },
        LlmRequest::Embedding(req) => {
            debug!("Processing embedding request for model: {}", req.model);
            let embedding_response = ctx.llm_client.embeddings(req).await
                .map_err(|e| blueprint_sdk::Error::Custom(e.to_string()))?;
            LlmResponse::Embedding(embedding_response)
        },
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
#[blueprint_sdk::debug_job]
pub async fn report_metrics(
    Context(ctx): Context<OpenRouterContext>,
) -> blueprint_sdk::Result<TangleResult<crate::llm::NodeMetrics>> {
    info!("Reporting metrics");
    
    // Update metrics before reporting
    ctx.update_metrics().await;
    
    // Get the current metrics
    let metrics = ctx.metrics.read().await.clone();
    
    info!("Metrics reported successfully");
    Ok(TangleResult(metrics))
}
