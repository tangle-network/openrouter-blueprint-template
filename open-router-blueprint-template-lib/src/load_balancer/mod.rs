//! Load balancing for the OpenRouter Blueprint
//!
//! This module provides load balancing functionality for distributing
//! requests across multiple LLM nodes.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use tracing::{debug, info};
use serde::{Serialize, Deserialize};

use crate::llm::{LlmClient, ModelInfo, NodeMetrics};

/// Load balancing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    /// Round-robin strategy
    RoundRobin,
    
    /// Least-loaded strategy (based on active requests)
    LeastLoaded,
    
    /// Capability-based strategy (route to nodes that support specific models)
    CapabilityBased,
    
    /// Latency-based strategy (route to nodes with lowest response time)
    LatencyBased,
}

impl Default for LoadBalancingStrategy {
    fn default() -> Self {
        Self::RoundRobin
    }
}

/// Configuration for the load balancer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// The load balancing strategy to use
    pub strategy: LoadBalancingStrategy,
    
    /// Maximum number of retries if a node fails
    pub max_retries: usize,
    
    /// Timeout for node selection in milliseconds
    pub selection_timeout_ms: u64,
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::default(),
            max_retries: 3,
            selection_timeout_ms: 1000,
        }
    }
}

/// A node in the load balancer
#[derive(Clone)]
pub struct LoadBalancerNode {
    /// Unique identifier for the node
    pub id: String,
    
    /// The LLM client for this node
    pub client: Arc<dyn LlmClient>,
    
    /// Last reported metrics for this node
    pub metrics: NodeMetrics,
    
    /// Whether this node is active
    pub active: bool,
}

impl std::fmt::Debug for LoadBalancerNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoadBalancerNode")
            .field("id", &self.id)
            .field("client", &"<dyn LlmClient>")
            .field("metrics", &self.metrics)
            .field("active", &self.active)
            .finish()
    }
}

/// Load balancer for distributing requests across multiple LLM nodes
pub struct LoadBalancer {
    /// Configuration for the load balancer
    config: LoadBalancerConfig,
    
    /// Nodes in the load balancer
    nodes: RwLock<HashMap<String, LoadBalancerNode>>,
    
    /// Current round-robin index
    round_robin_index: RwLock<usize>,
}

impl LoadBalancer {
    /// Create a new load balancer with the given configuration
    pub fn new(config: LoadBalancerConfig) -> Self {
        Self {
            config,
            nodes: RwLock::new(HashMap::new()),
            round_robin_index: RwLock::new(0),
        }
    }
    
    /// Add a node to the load balancer
    pub async fn add_node(&self, id: String, client: Arc<dyn LlmClient>) {
        let metrics = client.get_metrics();
        let node = LoadBalancerNode {
            id: id.clone(),
            client,
            metrics,
            active: true,
        };
        
        let mut nodes = self.nodes.write().await;
        nodes.insert(id.clone(), node);
        
        info!("Added node to load balancer: {}", id);
    }
    
    /// Remove a node from the load balancer
    pub async fn remove_node(&self, id: &str) -> bool {
        let mut nodes = self.nodes.write().await;
        let removed = nodes.remove(id).is_some();
        
        if removed {
            info!("Removed node from load balancer: {}", id);
        } else {
            debug!("Attempted to remove non-existent node: {}", id);
        }
        
        removed
    }
    
    /// Update the metrics for a node
    pub async fn update_node_metrics(&self, id: &str, metrics: NodeMetrics) -> bool {
        let mut nodes = self.nodes.write().await;
        
        if let Some(node) = nodes.get_mut(id) {
            node.metrics = metrics;
            true
        } else {
            debug!("Attempted to update metrics for non-existent node: {}", id);
            false
        }
    }
    
    /// Set the active state for a node
    pub async fn set_node_active(&self, id: &str, active: bool) -> bool {
        let mut nodes = self.nodes.write().await;
        
        if let Some(node) = nodes.get_mut(id) {
            node.active = active;
            true
        } else {
            debug!("Attempted to set active state for non-existent node: {}", id);
            false
        }
    }
    
    /// Get a node by ID
    pub async fn get_node(&self, id: &str) -> Option<LoadBalancerNode> {
        let nodes = self.nodes.read().await;
        nodes.get(id).cloned()
    }
    
    /// Get all nodes
    pub async fn get_all_nodes(&self) -> Vec<LoadBalancerNode> {
        let nodes = self.nodes.read().await;
        nodes.values().cloned().collect()
    }
    
    /// Get all active nodes
    pub async fn get_active_nodes(&self) -> Vec<LoadBalancerNode> {
        let nodes = self.nodes.read().await;
        nodes.values().filter(|n| n.active).cloned().collect()
    }
    
    /// Select a node for the given model using the configured strategy
    pub async fn select_node_for_model(&self, model: &str) -> Option<LoadBalancerNode> {
        let active_nodes = self.get_active_nodes().await;
        
        if active_nodes.is_empty() {
            debug!("No active nodes available for selection");
            return None;
        }
        
        // Filter nodes that support the requested model
        let supporting_nodes: Vec<_> = active_nodes
            .into_iter()
            .filter(|n| {
                n.client.get_supported_models().iter().any(|m| m.id == model)
            })
            .collect();
        
        if supporting_nodes.is_empty() {
            debug!("No nodes support the requested model: {}", model);
            return None;
        }
        
        // Select a node based on the configured strategy
        match self.config.strategy {
            LoadBalancingStrategy::RoundRobin => {
                self.select_round_robin(&supporting_nodes).await
            },
            LoadBalancingStrategy::LeastLoaded => {
                self.select_least_loaded(&supporting_nodes)
            },
            LoadBalancingStrategy::CapabilityBased => {
                self.select_capability_based(&supporting_nodes, model)
            },
            LoadBalancingStrategy::LatencyBased => {
                self.select_latency_based(&supporting_nodes)
            },
        }
    }
    
    /// Select a node using the round-robin strategy
    async fn select_round_robin(&self, nodes: &[LoadBalancerNode]) -> Option<LoadBalancerNode> {
        if nodes.is_empty() {
            return None;
        }
        
        let mut index = self.round_robin_index.write().await;
        let selected_index = *index % nodes.len();
        *index = (*index + 1) % nodes.len();
        
        Some(nodes[selected_index].clone())
    }
    
    /// Select a node using the least-loaded strategy
    fn select_least_loaded(&self, nodes: &[LoadBalancerNode]) -> Option<LoadBalancerNode> {
        if nodes.is_empty() {
            return None;
        }
        
        nodes.iter()
            .min_by_key(|n| n.metrics.active_requests)
            .cloned()
    }
    
    /// Select a node using the capability-based strategy
    fn select_capability_based(&self, nodes: &[LoadBalancerNode], model: &str) -> Option<LoadBalancerNode> {
        if nodes.is_empty() {
            return None;
        }
        
        // Find nodes that support the model and sort by capability
        let mut scored_nodes: Vec<_> = nodes.iter()
            .filter_map(|n| {
                let supported_models = n.client.get_supported_models();
                let model_info = supported_models
                    .iter()
                    .find(|m| m.id == model)?;
                
                // Score the node based on its capabilities
                let score = self.calculate_capability_score(n, model_info);
                Some((n, score))
            })
            .collect();
        
        // Sort by score (higher is better)
        scored_nodes.sort_by(|(_, score1), (_, score2)| score2.partial_cmp(score1).unwrap());
        
        // Return the highest-scoring node
        scored_nodes.first().map(|(node, _)| (*node).clone())
    }
    
    /// Calculate a capability score for a node and model
    fn calculate_capability_score(&self, node: &LoadBalancerNode, model_info: &ModelInfo) -> f32 {
        // Base score
        let mut score = 1.0;
        
        // Adjust score based on context length
        score += (model_info.max_context_length as f32) / 10000.0;
        
        // Adjust score based on node metrics
        score -= node.metrics.cpu_utilization * 0.5;
        score -= node.metrics.memory_utilization * 0.5;
        
        // Penalize nodes with high active requests
        score -= (node.metrics.active_requests as f32) * 0.1;
        
        score
    }
    
    /// Select a node using the latency-based strategy
    fn select_latency_based(&self, nodes: &[LoadBalancerNode]) -> Option<LoadBalancerNode> {
        if nodes.is_empty() {
            return None;
        }
        
        nodes.iter()
            .min_by_key(|n| n.metrics.average_response_time_ms)
            .cloned()
    }
    
    /// Calculate a score for a node based on its capabilities for a specific model
    async fn calculate_capability_score_for_model(&self, node_id: &str, model: &str) -> Option<f64> {
        let nodes = self.nodes.read().await;
        let n = nodes.get(node_id)?;
        
        // Find the model info
        let supported_models = n.client.get_supported_models();
        let model_info = supported_models
            .iter()
            .find(|m| m.id == model)?;
        
        let score = self.calculate_capability_score(n, model_info) as f64;
        
        Some(score)
    }
}
