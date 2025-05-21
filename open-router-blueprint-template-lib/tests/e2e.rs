use blueprint_sdk::tangle::filters::MatchesServiceId;
use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::tangle::serde::to_field;
use blueprint_sdk::testing::tempfile;
use blueprint_sdk::testing::utils::setup_log;
use blueprint_sdk::testing::utils::tangle::TangleTestHarness;
use blueprint_sdk::Job;
use blueprint_sdk::Router;
use open_router_blueprint_template_lib::llm::{LlmRequest, TextCompletionRequest};
use open_router_blueprint_template_lib::{
    process_llm_request, OpenRouterContext, PROCESS_LLM_REQUEST_JOB_ID,
};
use std::collections::HashMap;
use tower::filter::FilterLayer;

const N: usize = 1;

#[tokio::test]
async fn test_blueprint() -> color_eyre::Result<()> {
    setup_log();

    // Change to the root directory of the project where the contracts are located
    let current_dir = std::env::current_dir()?;
    let root_dir = current_dir
        .parent()
        .ok_or_else(|| color_eyre::eyre::eyre!("Failed to find root directory"))?;
    std::env::set_current_dir(root_dir)?;

    let temp_dir = tempfile::TempDir::new()?;
    let context =
        OpenRouterContext::new(blueprint_sdk::runner::config::BlueprintEnvironment::default())
            .await?;
    let harness = TangleTestHarness::setup(temp_dir).await?;

    let (mut test_env, service_id, _) = harness.setup_services::<N>(false).await?;

    test_env.initialize().await?;

    // Create a Router and register the job with it
    let router = Router::new()
        .route(
            PROCESS_LLM_REQUEST_JOB_ID,
            process_llm_request.layer(TangleLayer),
        )
        .layer(FilterLayer::new(MatchesServiceId(service_id)));

    // Register the router with the test environment
    // test_env.set_router(router).await;

    test_env.start(context).await?;

    // Create a simple TextCompletionRequest to avoid issues with f32 serialization
    let request = LlmRequest::TextCompletion(TextCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Hello, world!".to_string(),
        max_tokens: Some(10),
        temperature: None, // Avoid using f32 values which might cause serialization issues
        top_p: None,       // Avoid using f32 values which might cause serialization issues
        stream: Some(false),
        additional_params: HashMap::new(),
    });

    // Serialize the request to a field
    let job_inputs = vec![to_field(request).unwrap()];
    let job = harness
        .submit_job(service_id, PROCESS_LLM_REQUEST_JOB_ID, job_inputs)
        .await?;

    let results = harness.wait_for_job_execution(service_id, job).await?;

    assert_eq!(results.service_id, service_id);
    Ok(())
}
