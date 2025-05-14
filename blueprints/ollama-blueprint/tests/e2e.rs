//! E2E test for Ollama Blueprint with deepseek-r1 model
//!
//! This test demonstrates:
//! 1. Instantiating the client
//! 2. Sending a chat completion request
//! 3. Sending a text completion request
//! 4. Handling unsupported models
//! 5. Metrics and capabilities
//!
//! ASCII diagram:
//!
//!   +-----------+      +----------------+      +------------------+
//!   | Test Code | ---> | OllamaLlmClient | --> | Ollama REST API  |
//!   +-----------+      +----------------+      +------------------+
//!
//! Expected outcome: The client should return a valid response from the Ollama model, handle errors, and expose metrics/capabilities.

use ollama_blueprint::OllamaLlmClient;
use open_router_blueprint_template_lib::llm::{
    ChatCompletionRequest, ChatMessage, LlmClient, LlmError, TextCompletionRequest,
};
use std::collections::HashMap;
use std::process::{Command, Stdio};
// Removed unused import: std::thread::sleep
use std::time::Duration;
use tracing::{debug, error, info, warn};

// Helper: Ensure Ollama is running
async fn ensure_ollama_running() {
    info!("Checking if Ollama is running");
    let client = reqwest::Client::new();
    let url = "http://localhost:11434";

    match client.get(url).send().await {
        Ok(_) => {
            info!("Ollama is already running at {}", url);
        }
        Err(e) => {
            warn!("Ollama is not running: {}", e);
            info!("Attempting to start Ollama server");

            // Try to start ollama serve
            match Command::new("ollama")
                .arg("serve")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(_) => debug!("Ollama serve command started"),
                Err(e) => error!("Failed to start Ollama: {}", e),
            };

            // Wait for it to start
            info!("Waiting for Ollama to start (initial 3s delay)");
            tokio::time::sleep(Duration::from_secs(3)).await;

            // Check again
            let mut ollama_started = false;
            for attempt in 1..=5 {
                debug!("Checking if Ollama is running (attempt {})", attempt);
                match client.get(url).send().await {
                    Ok(_) => {
                        info!("Ollama is now running after {} attempts", attempt);
                        ollama_started = true;
                        break;
                    }
                    Err(e) => {
                        warn!("Ollama still not running (attempt {}): {}", attempt, e);
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            if !ollama_started {
                error!("Failed to start Ollama after multiple attempts");
            }
        }
    }
}

#[tokio::test]
async fn test_chat_and_text_completion() {
    // Setup tracing for the test (using info level by default)
    info!("Starting Ollama blueprint E2E test");

    // Ensure Ollama is running before starting tests
    ensure_ollama_running().await;

    let api_url = "http://localhost:11434".to_string();
    let model = "deepseek-r1".to_string();
    info!(
        "Creating OllamaLlmClient with API URL: {} and model: {}",
        api_url, model
    );
    let client = OllamaLlmClient::new(api_url, model.clone());

    // Test chat completion
    info!("Testing chat completion with model: {}", model);
    let chat_req = ChatCompletionRequest {
        model: model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            name: None,
            content: "Hello, who are you?".to_string(),
        }],
        max_tokens: None,
        temperature: None,
        top_p: None,
        stream: None,
        additional_params: HashMap::new(),
    };

    debug!("Sending chat completion request");
    let chat_resp = client.chat_completion(chat_req).await;

    match &chat_resp {
        Ok(resp) => {
            info!("Chat completion succeeded with response ID: {}", resp.id);
            debug!("Chat completion response: {:?}", resp);
        }
        Err(e) => {
            error!("Chat completion failed: {:?}", e);
        }
    }

    assert!(chat_resp.is_ok(), "Chat completion failed: {chat_resp:?}");
    let chat_resp = chat_resp.unwrap();
    assert!(!chat_resp.choices.is_empty(), "No choices returned");
    assert!(
        !chat_resp.choices[0].message.content.trim().is_empty(),
        "Empty response"
    );
    info!("Chat completion test passed");

    // Test text completion
    info!("Testing text completion with model: {}", model);
    let text_req = TextCompletionRequest {
        model: model.clone(),
        prompt: "Write a haiku about code.".to_string(),
        max_tokens: None,
        temperature: None,
        top_p: None,
        stream: None,
        additional_params: HashMap::new(),
    };

    debug!("Sending text completion request");
    let text_resp = client.text_completion(text_req).await;

    match &text_resp {
        Ok(resp) => {
            info!("Text completion succeeded with response ID: {}", resp.id);
            debug!("Text completion response: {:?}", resp);
        }
        Err(e) => {
            error!("Text completion failed: {:?}", e);
        }
    }

    assert!(text_resp.is_ok(), "Text completion failed: {text_resp:?}");
    let text_resp = text_resp.unwrap();
    assert!(!text_resp.choices.is_empty(), "No text choices returned");
    assert!(
        !text_resp.choices[0].text.trim().is_empty(),
        "Empty text completion"
    );
    info!("Text completion test passed");

    // Test error for unsupported model
    info!("Testing error handling for unsupported model");
    let bad_model = "not-a-real-model".to_string();
    let bad_req = ChatCompletionRequest {
        model: bad_model.clone(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            name: None,
            content: "Test".to_string(),
        }],
        max_tokens: None,
        temperature: None,
        top_p: None,
        stream: None,
        additional_params: HashMap::new(),
    };

    debug!(
        "Sending chat completion request with invalid model: {}",
        bad_model
    );
    let bad_resp = client.chat_completion(bad_req).await;

    match &bad_resp {
        Ok(_) => {
            warn!("Unexpectedly succeeded with invalid model: {}", bad_model);
        }
        Err(e) => {
            info!("Received expected error for invalid model: {:?}", e);
        }
    }

    assert!(
        matches!(bad_resp, Err(LlmError::ModelNotSupported(_))),
        "Expected ModelNotSupported error"
    );
    info!("Unsupported model error test passed");

    // Test capabilities and metrics
    info!("Testing capabilities and metrics");
    let caps = client.get_capabilities();
    debug!("Client capabilities: {:?}", caps);
    assert!(
        !caps.supports_streaming,
        "Ollama should not support streaming by default"
    );
    assert_eq!(caps.max_concurrent_requests, 1);

    let metrics = client.get_metrics();
    debug!("Client metrics: {:?}", metrics);
    // We can't guarantee specific values, but metrics should exist
    assert!(metrics.cpu_utilization >= 0.0);
    info!("Capabilities and metrics test passed");

    info!("All Ollama blueprint E2E tests completed successfully");
}
