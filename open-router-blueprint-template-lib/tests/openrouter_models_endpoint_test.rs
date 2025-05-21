use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use blueprint_sdk::runner::config::BlueprintEnvironment;
use blueprint_sdk::testing::utils::setup_log;
use color_eyre::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use open_router_blueprint_template_lib::context::OpenRouterContext;
use open_router_blueprint_template_lib::llm::ModelInfo;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

// OpenRouter model format
#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterModel {
    id: String,
    name: String,
    created: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    context_length: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_completion_tokens: Option<usize>,
    quantization: String,
    pricing: OpenRouterPricing,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterPricing {
    prompt: String,
    completion: String,
    image: String,
    request: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenRouterModelsResponse {
    data: Vec<OpenRouterModel>,
}

// Convert internal ModelInfo to OpenRouter format
fn convert_to_openrouter_model(model: &ModelInfo) -> OpenRouterModel {
    // Get current timestamp in seconds since epoch
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Extract max_completion_tokens if available in parameters
    let max_completion_tokens = model
        .parameters
        .get("max_completion_tokens")
        .and_then(|v| v.parse::<usize>().ok());

    // Extract description if available in parameters
    let description = model.parameters.get("description").cloned();

    OpenRouterModel {
        id: model.id.clone(),
        name: model.name.clone(),
        created: now,
        description,
        context_length: model.max_context_length,
        max_completion_tokens,
        // Default quantization to "none" if not specified
        quantization: model
            .parameters
            .get("quantization")
            .cloned()
            .unwrap_or_else(|| "none".to_string()),
        pricing: OpenRouterPricing {
            prompt: model
                .parameters
                .get("pricing_prompt")
                .cloned()
                .unwrap_or_else(|| "0.000001".to_string()),
            completion: model
                .parameters
                .get("pricing_completion")
                .cloned()
                .unwrap_or_else(|| "0.000002".to_string()),
            image: model
                .parameters
                .get("pricing_image")
                .cloned()
                .unwrap_or_else(|| "0".to_string()),
            request: model
                .parameters
                .get("pricing_request")
                .cloned()
                .unwrap_or_else(|| "0".to_string()),
        },
    }
}

// HTTP handler for the models endpoint
async fn handle_request(
    req: Request<Body>,
    context: Arc<OpenRouterContext>,
) -> Result<Response<Body>, hyper::Error> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, "/v1/models") => {
            // Get models from the LLM client
            let models = context.llm_client.get_supported_models();

            // Convert to OpenRouter format
            let openrouter_models = models
                .iter()
                .map(convert_to_openrouter_model)
                .collect::<Vec<_>>();

            // Create response
            let response = OpenRouterModelsResponse {
                data: openrouter_models,
            };

            // Serialize to JSON
            let json = serde_json::to_string(&response).unwrap_or_default();

            // Return response
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(json))
                .unwrap())
        }
        // Return 404 for any other path
        _ => {
            let mut not_found = Response::default();
            *not_found.status_mut() = StatusCode::NOT_FOUND;
            Ok(not_found)
        }
    }
}

/// Test that verifies the OpenRouter models endpoint returns the correct format
#[tokio::test]
async fn test_openrouter_models_endpoint() -> Result<()> {
    setup_log();

    // Create a mock environment
    let env = BlueprintEnvironment::default();

    // Create the context
    let context = Arc::new(OpenRouterContext::new(env).await?);

    // Set up the HTTP server
    let addr = SocketAddr::from(([127, 0, 0, 1], 0)); // Use port 0 to let the OS assign a free port

    // Create a service
    let context_clone = context.clone();
    let make_svc = make_service_fn(move |_conn| {
        let context = context_clone.clone();
        async move { Ok::<_, hyper::Error>(service_fn(move |req| handle_request(req, context.clone()))) }
    });

    // Create a channel for the server to signal when it's ready to shut down
    let (tx, rx) = oneshot::channel::<()>();

    // Bind the server to get the actual address
    let server = Server::bind(&addr);
    let actual_addr = server.local_addr();
    println!("Server running on http://{}", actual_addr);

    // Now apply the service and graceful shutdown
    let server = server.serve(make_svc).with_graceful_shutdown(async {
        rx.await.ok();
    });

    // Spawn the server on a separate task
    let server_handle = tokio::spawn(server);

    // Give the server a moment to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Make a request to the models endpoint
    let client = reqwest::Client::new();
    let response = client
        .get(&format!("http://{}/v1/models", actual_addr))
        .send()
        .await?;

    // Verify the response status
    assert_eq!(response.status(), reqwest::StatusCode::OK);

    // Parse the response body
    let models_response = response.json::<OpenRouterModelsResponse>().await?;

    // Verify that the response contains models
    assert!(
        !models_response.data.is_empty(),
        "Models list should not be empty"
    );

    // Verify that each model has the required fields
    for model in &models_response.data {
        assert!(!model.id.is_empty(), "Model ID should not be empty");
        assert!(!model.name.is_empty(), "Model name should not be empty");
        assert!(model.created > 0, "Created timestamp should be positive");
        assert!(
            model.context_length > 0,
            "Context length should be positive"
        );

        // Verify pricing fields
        assert!(
            !model.pricing.prompt.is_empty(),
            "Prompt pricing should not be empty"
        );
        assert!(
            !model.pricing.completion.is_empty(),
            "Completion pricing should not be empty"
        );
        assert!(
            !model.pricing.image.is_empty(),
            "Image pricing should not be empty"
        );
        assert!(
            !model.pricing.request.is_empty(),
            "Request pricing should not be empty"
        );
    }

    // Shut down the server
    let _ = tx.send(());

    // Wait for the server to shut down
    let _ = server_handle.await;

    Ok(())
}
