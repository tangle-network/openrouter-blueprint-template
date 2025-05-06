use std::sync::Arc;
use tokio::sync::RwLock;

use blueprint_sdk::runner::config::BlueprintEnvironment;
use tracing::info;

use crate::config::BlueprintConfig;
use crate::llm::{LlmClient, LocalLlmClient, LocalLlmConfig, NodeMetrics};
use crate::load_balancer::{LoadBalancer, LoadBalancerConfig};

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

    /// Blueprint configuration
    pub blueprint_config: Arc<RwLock<BlueprintConfig>>,
}

impl OpenRouterContext {
    /// Create a new OpenRouter context
    pub async fn new(env: BlueprintEnvironment) -> Result<Self, blueprint_sdk::Error> {
        // Load configuration
        let blueprint_config = {
            // Try to load from the data directory if it exists
            let config_path = env.data_dir.as_ref().map(|dir| dir.join("config.json"));
            if let Some(path) = &config_path {
                if path.exists() {
                    info!("Loading configuration from {}", path.display());
                    match BlueprintConfig::load(path) {
                        Ok(config) => {
                            info!("Configuration loaded successfully");
                            config
                        }
                        Err(e) => {
                            info!("Failed to load configuration: {}, using default", e);
                            BlueprintConfig::from_env()
                        }
                    }
                } else {
                    info!("No configuration file found, using environment variables");
                    BlueprintConfig::from_env()
                }
            } else {
                info!("No data directory specified, using environment variables");
                BlueprintConfig::from_env()
            }
        };

        // Validate the configuration
        if let Err(e) = blueprint_config.validate() {
            info!("Configuration validation failed: {}, using default", e);
            // Continue with default configuration
        }

        // Create a local LLM config from the blueprint config
        let local_config = LocalLlmConfig {
            api_url: blueprint_config.llm.api_url.clone(),
            timeout_seconds: blueprint_config.llm.timeout_seconds,
            max_concurrent_requests: blueprint_config.llm.max_concurrent_requests,
            models: blueprint_config.llm.models.clone(),
            additional_params: blueprint_config.llm.additional_params.clone(),
        };

        // Create the default LLM client
        let llm_client = Arc::new(LocalLlmClient::new(local_config.clone()));

        // Get initial metrics
        let metrics = Arc::new(RwLock::new(llm_client.get_metrics()));

        // Create the load balancer with configuration from blueprint config
        let load_balancer_config = LoadBalancerConfig {
            strategy: blueprint_config.load_balancer.strategy,
            max_retries: blueprint_config.load_balancer.max_retries,
            selection_timeout_ms: blueprint_config.load_balancer.selection_timeout_ms,
        };
        let load_balancer = Arc::new(LoadBalancer::new(load_balancer_config));

        // Add the default LLM client to the load balancer
        load_balancer
            .add_node("default".to_string(), llm_client.clone())
            .await;

        info!("Created OpenRouter context with default LLM client and load balancer");

        Ok(Self {
            env,
            llm_client,
            metrics,
            config: Arc::new(RwLock::new(local_config)),
            load_balancer,
            blueprint_config: Arc::new(RwLock::new(blueprint_config)),
        })
    }

    /// Update the metrics for this node
    pub async fn update_metrics(&self) {
        let metrics = self.llm_client.get_metrics();
        let mut metrics_lock = self.metrics.write().await;
        *metrics_lock = metrics;

        let metrics = self.llm_client.get_metrics();
        self.load_balancer
            .update_node_metrics("default", metrics)
            .await;
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

    /// Reload configuration from file
    pub async fn reload_config(&self) -> Result<(), String> {
        // Try to load from the data directory
        if let Some(data_dir) = self.env.data_dir.as_ref() {
            let config_path = data_dir.join("config.json");
            if config_path.exists() {
                match BlueprintConfig::load(&config_path) {
                    Ok(config) => {
                        // Validate the configuration
                        if let Err(e) = config.validate() {
                            return Err(format!("Configuration validation failed: {}", e));
                        }

                        // Update the configuration
                        *self.blueprint_config.write().await = config;

                        // Update the local LLM config
                        let config = self.blueprint_config.read().await;
                        let mut local_config = self.config.write().await;
                        local_config.api_url = config.llm.api_url.clone();
                        local_config.timeout_seconds = config.llm.timeout_seconds;
                        local_config.max_concurrent_requests = config.llm.max_concurrent_requests;
                        local_config.models = config.llm.models.clone();
                        local_config.additional_params = config.llm.additional_params.clone();

                        info!("Configuration reloaded successfully");
                        Ok(())
                    }
                    Err(e) => Err(format!("Failed to load configuration: {}", e)),
                }
            } else {
                Err("Configuration file not found".to_string())
            }
        } else {
            Err("Data directory not specified".to_string())
        }
    }
}
