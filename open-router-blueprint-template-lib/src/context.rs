//! Context for the OpenRouter Blueprint
//!
//! This module defines the context structure that holds the shared state
//! for the OpenRouter Blueprint.

use std::sync::Arc;
use tokio::sync::RwLock;

use blueprint_sdk::contexts::tangle::TangleClientContext;
use blueprint_sdk::runner::config::BlueprintEnvironment;

use crate::llm::{LlmClient, LocalLlmClient, LocalLlmConfig, NodeMetrics};

/// Context for the OpenRouter Blueprint
#[derive(Clone, TangleClientContext)]
pub struct OpenRouterContext {
    /// Blueprint environment configuration
    #[config]
    pub env: BlueprintEnvironment,
    
    /// LLM client for processing requests
    pub llm_client: Arc<dyn LlmClient>,
    
    /// Metrics for this node
    pub metrics: Arc<RwLock<NodeMetrics>>,
    
    /// Configuration for the LLM client
    pub config: Arc<RwLock<LocalLlmConfig>>,
}

impl OpenRouterContext {
    /// Create a new OpenRouter context
    pub async fn new(env: BlueprintEnvironment) -> blueprint_sdk::Result<Self> {
        // Create a default configuration
        let config = LocalLlmConfig::default();
        
        // Create the LLM client
        let llm_client = Arc::new(LocalLlmClient::new(config.clone()));
        
        // Get initial metrics
        let metrics = Arc::new(RwLock::new(llm_client.get_metrics()));
        
        Ok(Self {
            env,
            llm_client,
            metrics,
            config: Arc::new(RwLock::new(config)),
        })
    }
    
    /// Update the metrics for this node
    pub async fn update_metrics(&self) {
        let metrics = self.llm_client.get_metrics();
        *self.metrics.write().await = metrics;
    }
}
