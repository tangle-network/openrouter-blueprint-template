//! Context for the OpenRouter Blueprint
//!
//! This module defines the context structure that holds the shared state
//! for the OpenRouter Blueprint.

use std::sync::Arc;
use tokio::sync::RwLock;

use blueprint_sdk::runner::config::BlueprintEnvironment;
use tracing::info;

use crate::llm::{LlmClient, LocalLlmClient, LocalLlmConfig, NodeMetrics};
use crate::load_balancer::{LoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};

/// Context for the OpenRouter Blueprint
#[derive(Clone)]
pub struct OpenRouterContext {
    /// Blueprint environment configuration
    pub env: BlueprintEnvironment,
    
    /// Default LLM client for processing requests
    pub llm_client: Arc<dyn LlmClient>,
    
    /// Metrics for this node
    pub metrics: Arc<RwLock<NodeMetrics>>,
    
    /// Configuration for the LLM client
    pub config: Arc<RwLock<LocalLlmConfig>>,
    
    /// Load balancer for distributing requests across multiple LLM nodes
    pub load_balancer: Arc<LoadBalancer>,
}

impl OpenRouterContext {
    /// Create a new OpenRouter context
    pub async fn new(env: BlueprintEnvironment) -> Result<Self, blueprint_sdk::Error> {
        // Create a default configuration
        let config = LocalLlmConfig::default();
        
        // Create the default LLM client
        let llm_client = Arc::new(LocalLlmClient::new(config.clone()));
        
        // Get initial metrics
        let metrics = Arc::new(RwLock::new(llm_client.get_metrics()));
        
        // Create the load balancer with default configuration
        let load_balancer_config = LoadBalancerConfig {
            strategy: LoadBalancingStrategy::LeastLoaded,
            max_retries: 3,
            selection_timeout_ms: 1000,
        };
        let load_balancer = Arc::new(LoadBalancer::new(load_balancer_config));
        
        // Add the default LLM client to the load balancer
        load_balancer.add_node("default".to_string(), llm_client.clone()).await;
        
        info!("Created OpenRouter context with default LLM client and load balancer");
        
        Ok(Self {
            env,
            llm_client,
            metrics,
            config: Arc::new(RwLock::new(config)),
            load_balancer,
        })
    }
    
    /// Update the metrics for this node
    pub async fn update_metrics(&self) {
        let metrics = self.llm_client.get_metrics();
        *self.metrics.write().await = metrics;
        
        // Also update the metrics in the load balancer
        self.load_balancer.update_node_metrics("default", metrics).await;
    }
    
    /// Add an LLM node to the load balancer
    pub async fn add_llm_node(&self, id: String, client: Arc<dyn LlmClient>) {
        self.load_balancer.add_node(id, client).await;
    }
    
    /// Remove an LLM node from the load balancer
    pub async fn remove_llm_node(&self, id: &str) -> bool {
        self.load_balancer.remove_node(id).await
    }
    
    /// Get an LLM client for the specified model
    pub async fn get_llm_client_for_model(&self, model: &str) -> Option<Arc<dyn LlmClient>> {
        // Try to select a node from the load balancer
        let node = self.load_balancer.select_node_for_model(model).await?;
        Some(node.client)
    }
}
