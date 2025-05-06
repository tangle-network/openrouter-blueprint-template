//! Tests for the load balancer module
//!
//! This module contains tests for the load balancing functionality.

use std::sync::Arc;
use tokio::test;

use crate::load_balancer::{LoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};
use crate::tests::{MockLlmClient, create_test_load_balancer, add_mock_clients};

/// Test that verifies adding and removing nodes from the load balancer works correctly
#[tokio::test]
async fn test_add_remove_nodes() {
    // Create a load balancer
    let load_balancer = create_test_load_balancer();
    
    // Add nodes
    let client1 = Arc::new(MockLlmClient::new());
    let client2 = Arc::new(MockLlmClient::new());
    
    load_balancer.add_node("node1".to_string(), client1.clone()).await;
    load_balancer.add_node("node2".to_string(), client2.clone()).await;
    
    // Verify nodes were added
    let nodes = load_balancer.get_nodes().await;
    assert_eq!(nodes.len(), 2);
    assert!(nodes.contains_key("node1"));
    assert!(nodes.contains_key("node2"));
    
    // Remove a node
    let removed = load_balancer.remove_node("node1").await;
    assert!(removed);
    
    // Verify node was removed
    let nodes = load_balancer.get_nodes().await;
    assert_eq!(nodes.len(), 1);
    assert!(!nodes.contains_key("node1"));
    assert!(nodes.contains_key("node2"));
    
    // Try to remove a non-existent node
    let removed = load_balancer.remove_node("node3").await;
    assert!(!removed);
}

/// Test that verifies updating node metrics works correctly
#[tokio::test]
async fn test_update_node_metrics() {
    // Create a load balancer
    let load_balancer = create_test_load_balancer();
    
    // Add a node
    let client = Arc::new(MockLlmClient::new());
    load_balancer.add_node("node1".to_string(), client.clone()).await;
    
    // Get the initial metrics
    let nodes = load_balancer.get_nodes().await;
    let node = nodes.get("node1").unwrap();
    let initial_metrics = node.metrics.clone();
    
    // Create updated metrics
    let mut updated_metrics = initial_metrics.clone();
    updated_metrics.cpu_utilization = 0.8;
    updated_metrics.memory_utilization = 0.7;
    updated_metrics.gpu_utilization = Some(0.9);
    updated_metrics.requests_per_minute = 200;
    updated_metrics.average_response_time_ms = 300;
    updated_metrics.active_requests = 10;
    
    // Update the metrics
    load_balancer.update_node_metrics("node1", updated_metrics.clone()).await;
    
    // Verify the metrics were updated
    let nodes = load_balancer.get_nodes().await;
    let node = nodes.get("node1").unwrap();
    assert_eq!(node.metrics.cpu_utilization, updated_metrics.cpu_utilization);
    assert_eq!(node.metrics.memory_utilization, updated_metrics.memory_utilization);
    assert_eq!(node.metrics.gpu_utilization, updated_metrics.gpu_utilization);
    assert_eq!(node.metrics.requests_per_minute, updated_metrics.requests_per_minute);
    assert_eq!(node.metrics.average_response_time_ms, updated_metrics.average_response_time_ms);
    assert_eq!(node.metrics.active_requests, updated_metrics.active_requests);
}

/// Test that verifies the round robin load balancing strategy works correctly
#[tokio::test]
async fn test_round_robin_strategy() {
    // Create a load balancer with round robin strategy
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::RoundRobin,
        max_retries: 3,
        selection_timeout_ms: 1000,
    };
    let load_balancer = Arc::new(LoadBalancer::new(config));
    
    // Add nodes
    add_mock_clients(&load_balancer, 3).await;
    
    // Select nodes multiple times and verify round robin behavior
    let node1 = load_balancer.select_node().await.unwrap();
    let node2 = load_balancer.select_node().await.unwrap();
    let node3 = load_balancer.select_node().await.unwrap();
    let node4 = load_balancer.select_node().await.unwrap();
    
    // Verify each node is different from the previous one
    assert_ne!(node1.id, node2.id);
    assert_ne!(node2.id, node3.id);
    assert_ne!(node3.id, node4.id);
    
    // Verify the fourth selection is the same as the first (round robin)
    assert_eq!(node1.id, node4.id);
}

/// Test that verifies the least loaded load balancing strategy works correctly
#[tokio::test]
async fn test_least_loaded_strategy() {
    // Create a load balancer with least loaded strategy
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::LeastLoaded,
        max_retries: 3,
        selection_timeout_ms: 1000,
    };
    let load_balancer = Arc::new(LoadBalancer::new(config));
    
    // Add nodes with different loads
    let client1 = Arc::new(MockLlmClient::new());
    let client2 = Arc::new(MockLlmClient::new());
    let client3 = Arc::new(MockLlmClient::new());
    
    load_balancer.add_node("node1".to_string(), client1.clone()).await;
    load_balancer.add_node("node2".to_string(), client2.clone()).await;
    load_balancer.add_node("node3".to_string(), client3.clone()).await;
    
    // Update metrics to have different loads
    let mut metrics1 = client1.get_metrics();
    metrics1.cpu_utilization = 0.8;
    metrics1.active_requests = 10;
    
    let mut metrics2 = client2.get_metrics();
    metrics2.cpu_utilization = 0.3;
    metrics2.active_requests = 3;
    
    let mut metrics3 = client3.get_metrics();
    metrics3.cpu_utilization = 0.5;
    metrics3.active_requests = 5;
    
    load_balancer.update_node_metrics("node1", metrics1).await;
    load_balancer.update_node_metrics("node2", metrics2).await;
    load_balancer.update_node_metrics("node3", metrics3).await;
    
    // Select a node and verify it's the least loaded (node2)
    let selected = load_balancer.select_node().await.unwrap();
    assert_eq!(selected.id, "node2");
    
    // Update node2 to be heavily loaded
    let mut metrics2 = client2.get_metrics();
    metrics2.cpu_utilization = 0.9;
    metrics2.active_requests = 15;
    load_balancer.update_node_metrics("node2", metrics2).await;
    
    // Select a node again and verify it's the new least loaded (node3)
    let selected = load_balancer.select_node().await.unwrap();
    assert_eq!(selected.id, "node3");
}

/// Test that verifies the random load balancing strategy works correctly
#[tokio::test]
async fn test_random_strategy() {
    // Create a load balancer with random strategy
    let config = LoadBalancerConfig {
        strategy: LoadBalancingStrategy::Random,
        max_retries: 3,
        selection_timeout_ms: 1000,
    };
    let load_balancer = Arc::new(LoadBalancer::new(config));
    
    // Add nodes
    add_mock_clients(&load_balancer, 3).await;
    
    // Select nodes multiple times
    let mut selected_ids = std::collections::HashSet::new();
    for _ in 0..10 {
        let node = load_balancer.select_node().await.unwrap();
        selected_ids.insert(node.id.clone());
    }
    
    // Verify that at least 2 different nodes were selected (probabilistic)
    assert!(selected_ids.len() >= 2);
}

/// Test that verifies selecting a node for a specific model works correctly
#[tokio::test]
async fn test_select_node_for_model() {
    // Create a load balancer
    let load_balancer = create_test_load_balancer();
    
    // Add nodes with different supported models
    let mut client1 = MockLlmClient::new();
    client1.models = vec![
        crate::llm::ModelInfo {
            id: "model1".to_string(),
            name: "Model 1".to_string(),
            max_context_length: 4096,
            supports_chat: true,
            supports_text: true,
            supports_embeddings: true,
            parameters: Default::default(),
        },
    ];
    
    let mut client2 = MockLlmClient::new();
    client2.models = vec![
        crate::llm::ModelInfo {
            id: "model2".to_string(),
            name: "Model 2".to_string(),
            max_context_length: 4096,
            supports_chat: true,
            supports_text: true,
            supports_embeddings: true,
            parameters: Default::default(),
        },
    ];
    
    load_balancer.add_node("node1".to_string(), Arc::new(client1)).await;
    load_balancer.add_node("node2".to_string(), Arc::new(client2)).await;
    
    // Select a node for model1
    let selected = load_balancer.select_node_for_model("model1").await.unwrap();
    assert_eq!(selected.id, "node1");
    
    // Select a node for model2
    let selected = load_balancer.select_node_for_model("model2").await.unwrap();
    assert_eq!(selected.id, "node2");
    
    // Try to select a node for a non-existent model
    let selected = load_balancer.select_node_for_model("model3").await;
    assert!(selected.is_none());
}

/// Test that verifies the load balancer handles node failures correctly
#[tokio::test]
async fn test_node_failure_handling() {
    // Create a load balancer
    let load_balancer = create_test_load_balancer();
    
    // Add a failing node and a working node
    let failing_client = Arc::new(MockLlmClient::new().with_failure());
    let working_client = Arc::new(MockLlmClient::new());
    
    load_balancer.add_node("failing".to_string(), failing_client).await;
    load_balancer.add_node("working".to_string(), working_client).await;
    
    // Mark the failing node as failed
    load_balancer.mark_node_failed("failing").await;
    
    // Select a node and verify it's the working one
    let selected = load_balancer.select_node().await.unwrap();
    assert_eq!(selected.id, "working");
    
    // Reset the failing node
    load_balancer.reset_node_failure("failing").await;
    
    // Now both nodes should be available for selection
    let mut selected_ids = std::collections::HashSet::new();
    for _ in 0..10 {
        let node = load_balancer.select_node().await.unwrap();
        selected_ids.insert(node.id.clone());
    }
    
    // Verify that both nodes were selected
    assert!(selected_ids.contains("failing"));
    assert!(selected_ids.contains("working"));
}
