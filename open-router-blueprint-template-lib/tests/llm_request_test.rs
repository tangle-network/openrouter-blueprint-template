use std::sync::Arc;

use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::tangle::extract::TangleArg;
use blueprint_sdk::testing::utils::setup_log;
use blueprint_sdk::{extract::Context, IntoJobResult, JobResult};
use open_router_blueprint_template_lib::{
    context::OpenRouterContext,
    jobs::process_llm_request,
    llm::{
        ChatCompletionRequest, ChatMessage, EmbeddingRequest, LlmRequest, ModelInfo,
        TextCompletionRequest,
    },
};

/// Test that verifies the LLM request processing job can handle chat completion requests
#[tokio::test]
async fn test_process_chat_completion_request() -> color_eyre::Result<()> {
    setup_log();

    // Create a mock environment
    let env = BlueprintEnvironment::default();

    // Create the context
    let context = OpenRouterContext::new(env).await?;

    // Create a chat completion request
    let model = context
        .llm_client
        .get_supported_models()
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

    let request = ChatCompletionRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a helpful assistant.".to_string(),
                name: None,
            },
            ChatMessage {
                role: "user".to_string(),
                content: "Hello, how are you?".to_string(),
                name: None,
            },
        ],
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    };

    // Wrap the request in the LlmRequest enum
    let llm_request = LlmRequest::ChatCompletion(request);

    // Process the request
    let result = process_llm_request(Context(context), TangleArg(llm_request)).await?;

    // Convert the result to a JobResult
    let job_result = result.into_job_result().unwrap();

    // Verify that the job was successful
    match job_result {
        JobResult::Ok { .. } => Ok(()),
        JobResult::Err(error) => Err(color_eyre::eyre::eyre!("Job failed: {}", error)),
    }
}

/// Test that verifies the LLM request processing job can handle text completion requests
#[tokio::test]
async fn test_process_text_completion_request() -> color_eyre::Result<()> {
    setup_log();

    // Create a mock environment
    let env = BlueprintEnvironment::default();

    // Create the context
    let context = OpenRouterContext::new(env).await?;

    // Create a text completion request
    let model = context
        .llm_client
        .get_supported_models()
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "text-davinci-003".to_string());

    let request = TextCompletionRequest {
        model,
        prompt: "Once upon a time".to_string(),
        max_tokens: Some(100),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    };

    // Wrap the request in the LlmRequest enum
    let llm_request = LlmRequest::TextCompletion(request);

    // Process the request
    let result = process_llm_request(Context(context), TangleArg(llm_request)).await?;

    // Convert the result to a JobResult
    let job_result = result.into_job_result().unwrap();

    // Verify that the job was successful
    match job_result {
        JobResult::Ok { .. } => Ok(()),
        JobResult::Err(error) => Err(color_eyre::eyre::eyre!("Job failed: {}", error)),
    }
}

/// Test that verifies the LLM request processing job can handle embedding requests
#[tokio::test]
async fn test_process_embedding_request() -> color_eyre::Result<()> {
    setup_log();

    // Create a mock environment
    let env = BlueprintEnvironment::default();

    // Create the context
    let context = OpenRouterContext::new(env).await?;

    // Create an embedding request
    let model = context
        .llm_client
        .get_supported_models()
        .first()
        .map(|m| m.id.clone())
        .unwrap_or_else(|| "text-embedding-ada-002".to_string());

    let request = EmbeddingRequest {
        model,
        input: vec!["The quick brown fox jumps over the lazy dog".to_string()],
        additional_params: Default::default(),
    };

    // Wrap the request in the LlmRequest enum
    let llm_request = LlmRequest::Embedding(request);

    // Process the request
    let result = process_llm_request(Context(context), TangleArg(llm_request)).await?;

    // Convert the result to a JobResult
    let job_result = result.into_job_result().unwrap();

    // Verify that the job was successful
    match job_result {
        JobResult::Ok { .. } => Ok(()),
        JobResult::Err(error) => Err(color_eyre::eyre::eyre!("Job failed: {}", error)),
    }
}
