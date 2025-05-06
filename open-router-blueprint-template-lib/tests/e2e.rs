use blueprint_sdk::tangle::layers::TangleLayer;
use blueprint_sdk::testing::tempfile;
use blueprint_sdk::testing::utils::setup_log;
use blueprint_sdk::testing::utils::tangle::TangleTestHarness;
use blueprint_sdk::tangle::serde::to_field;
use open_router_blueprint_template_lib::{OpenRouterContext, process_llm_request, PROCESS_LLM_REQUEST_JOB_ID};

// The number of nodes to spawn in the test
const N: usize = 1;

#[tokio::test]
async fn test_blueprint() -> color_eyre::Result<()> {
    setup_log();

    // Initialize test harness (node, keys, deployment)
    let temp_dir = tempfile::TempDir::new()?;
    let context = OpenRouterContext::new(blueprint_sdk::runner::config::BlueprintEnvironment::default()).await?;
    let harness = TangleTestHarness::setup(temp_dir).await?;

    // Setup service with `N` nodes
    let (mut test_env, service_id, _) = harness.setup_services::<N>(false).await?;

    // Setup the node(s)
    test_env.initialize().await?;
    test_env.add_job(PROCESS_LLM_REQUEST_JOB_ID.layer(TangleLayer)).await;

    // Start the test environment. It is now ready to receive job calls.
    test_env.start(context).await?;

    // Create a simple chat completion request
    use open_router_blueprint_template_lib::llm::{ChatCompletionRequest, ChatMessage, LlmRequest};
    let request = LlmRequest::ChatCompletion(ChatCompletionRequest {
        model: "gpt-3.5-turbo".to_string(),
        messages: vec![
            ChatMessage {
                role: "user".to_string(),
                content: "Hello".to_string(),
                name: None,
            }
        ],
        max_tokens: Some(10),
        temperature: Some(0.7),
        top_p: None,
        stream: None,
        additional_params: Default::default(),
    });

    // Submit the job call with the request
    let job_inputs = vec![to_field(request).unwrap()];
    let job = harness.submit_job(service_id, PROCESS_LLM_REQUEST_JOB_ID, job_inputs).await?;

    let results = harness.wait_for_job_execution(service_id, job).await?;

    // Verify the job executed successfully
    assert_eq!(results.service_id, service_id);
    Ok(())
}
