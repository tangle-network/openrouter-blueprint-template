//! Tests for the LLM module
//!
//! This module contains tests for the LLM client functionality.

use std::sync::Arc;
use tokio::test;

use crate::llm::{
    ChatCompletionRequest, ChatMessage, EmbeddingRequest, LlmClient, StreamingLlmClient,
    TextCompletionRequest,
};
use crate::tests::{
    MockLlmClient, MockStreamingLlmClient, create_test_chat_request, create_test_text_request,
    create_test_embedding_request,
};

/// Test that verifies the basic LLM client functionality works correctly
#[tokio::test]
async fn test_llm_client_basic() {
    // Create a mock LLM client
    let client = MockLlmClient::new();
    
    // Verify supported models
    let models = client.get_supported_models();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].id, "test-model");
    
    // Verify capabilities
    let capabilities = client.get_capabilities();
    assert!(capabilities.supports_streaming);
    assert_eq!(capabilities.max_concurrent_requests, 10);
    assert!(!capabilities.supports_batching);
    
    // Verify metrics
    let metrics = client.get_metrics();
    assert_eq!(metrics.cpu_utilization, 0.5);
    assert_eq!(metrics.memory_utilization, 0.3);
    assert_eq!(metrics.gpu_utilization, Some(0.7));
    assert_eq!(metrics.requests_per_minute, 100);
    assert_eq!(metrics.average_response_time_ms, 200);
    assert_eq!(metrics.active_requests, 5);
}

/// Test that verifies the chat completion functionality works correctly
#[tokio::test]
async fn test_chat_completion() {
    // Create a mock LLM client
    let client = MockLlmClient::new();
    
    // Create a chat completion request
    let request = create_test_chat_request();
    
    // Send the request
    let response = client.chat_completion(request.clone()).await;
    
    // Verify the response
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(response.model, request.model);
    
    // Test with a failing client
    let failing_client = MockLlmClient::new().with_failure();
    let response = failing_client.chat_completion(request).await;
    assert!(response.is_err());
}

/// Test that verifies the text completion functionality works correctly
#[tokio::test]
async fn test_text_completion() {
    // Create a mock LLM client
    let client = MockLlmClient::new();
    
    // Create a text completion request
    let request = create_test_text_request();
    
    // Send the request
    let response = client.text_completion(request.clone()).await;
    
    // Verify the response
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(response.model, request.model);
    
    // Test with a failing client
    let failing_client = MockLlmClient::new().with_failure();
    let response = failing_client.text_completion(request).await;
    assert!(response.is_err());
}

/// Test that verifies the embeddings functionality works correctly
#[tokio::test]
async fn test_embeddings() {
    // Create a mock LLM client
    let client = MockLlmClient::new();
    
    // Create an embedding request
    let request = create_test_embedding_request();
    
    // Send the request
    let response = client.embeddings(request.clone()).await;
    
    // Verify the response
    assert!(response.is_ok());
    let response = response.unwrap();
    assert_eq!(response.model, request.model);
    
    // Test with a failing client
    let failing_client = MockLlmClient::new().with_failure();
    let response = failing_client.embeddings(request).await;
    assert!(response.is_err());
}

/// Test that verifies the streaming chat completion functionality works correctly
#[tokio::test]
async fn test_streaming_chat_completion() {
    // Create a mock streaming LLM client
    let client = MockStreamingLlmClient::new();
    
    // Create a chat completion request
    let mut request = create_test_chat_request();
    request.stream = Some(true);
    
    // Send the request
    let response = client.streaming_chat_completion(request.clone()).await;
    
    // Verify the response
    assert!(response.is_ok());
    let mut stream = response.unwrap();
    
    // Read from the stream
    let chunk = stream.next().await;
    assert!(chunk.is_some());
    let chunk = chunk.unwrap();
    assert!(chunk.is_ok());
    let chunk = chunk.unwrap();
    assert_eq!(chunk.model, request.model);
    
    // Test with a failing client
    let failing_client = MockStreamingLlmClient::new().with_failure();
    let response = failing_client.streaming_chat_completion(request).await;
    assert!(response.is_err());
}

/// Test that verifies the streaming text completion functionality works correctly
#[tokio::test]
async fn test_streaming_text_completion() {
    // Create a mock streaming LLM client
    let client = MockStreamingLlmClient::new();
    
    // Create a text completion request
    let mut request = create_test_text_request();
    request.stream = Some(true);
    
    // Send the request
    let response = client.streaming_text_completion(request.clone()).await;
    
    // Verify the response
    assert!(response.is_ok());
    let mut stream = response.unwrap();
    
    // Read from the stream
    let chunk = stream.next().await;
    assert!(chunk.is_some());
    let chunk = chunk.unwrap();
    assert!(chunk.is_ok());
    let chunk = chunk.unwrap();
    assert_eq!(chunk.model, request.model);
    
    // Test with a failing client
    let failing_client = MockStreamingLlmClient::new().with_failure();
    let response = failing_client.streaming_text_completion(request).await;
    assert!(response.is_err());
}

/// Test that verifies the LLM client can be used through an Arc
#[tokio::test]
async fn test_llm_client_arc() {
    // Create a mock LLM client
    let client = Arc::new(MockLlmClient::new());
    
    // Create a chat completion request
    let request = create_test_chat_request();
    
    // Send the request
    let response = client.chat_completion(request).await;
    
    // Verify the response
    assert!(response.is_ok());
}

/// Test that verifies the streaming LLM client can be used through an Arc
#[tokio::test]
async fn test_streaming_llm_client_arc() {
    // Create a mock streaming LLM client
    let client = Arc::new(MockStreamingLlmClient::new());
    
    // Create a chat completion request
    let mut request = create_test_chat_request();
    request.stream = Some(true);
    
    // Send the request
    let response = client.streaming_chat_completion(request).await;
    
    // Verify the response
    assert!(response.is_ok());
}
