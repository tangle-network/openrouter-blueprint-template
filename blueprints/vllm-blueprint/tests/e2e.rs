use open_router_blueprint_template_lib::llm::{
    ChatCompletionRequest, ChatMessage, LlmClient, LlmError, ModelInfo, TextCompletionRequest,
};
use std::time::Duration;
use vllm_blueprint::VllmLlmClient;

#[tokio::test]
async fn test_vllm_client_creation() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "llama3".to_string());
    assert_eq!(client.api_url, "http://localhost:8000");
    assert_eq!(client.model, "llama3");
}

#[tokio::test]
async fn test_vllm_capabilities() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "llama3".to_string());
    let capabilities = client.get_capabilities();
    
    assert!(capabilities.supports_streaming);
    assert_eq!(capabilities.max_concurrent_requests, 4);
    assert!(capabilities.supports_batching);
}

// The following tests require a running vLLM server
// They are disabled by default and can be enabled with the "integration" feature

#[tokio::test]
#[ignore]
async fn test_vllm_models() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "llama3".to_string());
    let models = client.get_supported_models();
    
    // This test assumes that the model "llama3" is available in the vLLM server
    assert!(!models.is_empty());
    assert_eq!(models[0].id, "llama3");
}

#[tokio::test]
#[ignore]
async fn test_vllm_chat_completion() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "llama3".to_string());
    
    let request = ChatCompletionRequest {
        model: "llama3".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
            name: None,
        }],
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    };
    
    let response = client.chat_completion(request).await;
    
    // This test assumes that the model "llama3" is available in the vLLM server
    assert!(response.is_ok());
    let completion = response.unwrap();
    assert!(!completion.choices.is_empty());
    assert!(!completion.choices[0].message.content.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_vllm_text_completion() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "llama3".to_string());
    
    let request = TextCompletionRequest {
        model: "llama3".to_string(),
        prompt: "Once upon a time".to_string(),
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    };
    
    let response = client.text_completion(request).await;
    
    // This test assumes that the model "llama3" is available in the vLLM server
    assert!(response.is_ok());
    let completion = response.unwrap();
    assert!(!completion.choices.is_empty());
    assert!(!completion.choices[0].text.is_empty());
}

#[tokio::test]
#[ignore]
async fn test_vllm_embeddings_not_implemented() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "llama3".to_string());
    
    let request = open_router_blueprint_template_lib::llm::EmbeddingRequest {
        model: "llama3".to_string(),
        input: vec!["Hello, world!".to_string()],
        additional_params: Default::default(),
    };
    
    let response = client.embeddings(request).await;
    
    // Embeddings are not implemented in this example
    assert!(response.is_err());
    match response {
        Err(LlmError::NotImplemented(_)) => (),
        _ => panic!("Expected NotImplemented error"),
    }
}

#[tokio::test]
#[ignore]
async fn test_vllm_invalid_model() {
    let client = VllmLlmClient::new("http://localhost:8000".to_string(), "invalid-model".to_string());
    
    let request = ChatCompletionRequest {
        model: "invalid-model".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
            name: None,
        }],
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    };
    
    let response = client.chat_completion(request).await;
    
    // This test assumes that the model "invalid-model" is not available in the vLLM server
    assert!(response.is_err());
    match response {
        Err(LlmError::ModelNotSupported(_)) => (),
        _ => panic!("Expected ModelNotSupported error"),
    }
}

#[tokio::test]
#[ignore]
async fn test_vllm_server_unavailable() {
    let client = VllmLlmClient::new("http://localhost:9999".to_string(), "llama3".to_string());
    
    let request = ChatCompletionRequest {
        model: "llama3".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            content: "Hello, how are you?".to_string(),
            name: None,
        }],
        max_tokens: Some(50),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    };
    
    let response = client.chat_completion(request).await;
    
    // This test assumes that there is no server running on port 9999
    assert!(response.is_err());
    match response {
        Err(LlmError::RequestFailed(_)) => (),
        _ => panic!("Expected RequestFailed error"),
    }
}
