# vLLM Blueprint

This blueprint implements the OpenRouter Blueprint template for [vLLM](https://github.com/vllm-project/vllm), a high-throughput and memory-efficient inference engine for LLMs.

## Overview

The vLLM Blueprint provides an implementation of the `LlmClient` trait from the OpenRouter Blueprint template, allowing you to use vLLM as a backend for your LLM applications. vLLM is designed for high-performance inference and supports a wide range of open-source models.

## Features

- Connect to a running vLLM server
- Query available models
- Send chat completion requests
- Send text completion requests
- Proper error handling and logging
- Integration with the Tangle network via the OpenRouter Blueprint template

## Usage

### Starting a vLLM Server

Before using this blueprint, you need to have a running vLLM server. You can start one using Docker or directly with Python:

```bash
# Using Python
pip install vllm
python -m vllm.entrypoints.openai.api_server --model <model_name> --host 0.0.0.0 --port 8000

# Using Docker
docker run --gpus all -p 8000:8000 ghcr.io/vllm-project/vllm:latest --model <model_name>
```

Replace `<model_name>` with the name of the model you want to use (e.g., `meta-llama/Llama-2-7b-chat-hf`).

### Using the Blueprint

To use this blueprint in your application:

1. Add the blueprint as a dependency in your `Cargo.toml`:

```toml
[dependencies]
vllm-blueprint = { path = "../blueprints/vllm-blueprint" }
```

2. Create a vLLM client and use it to send requests:

```rust
use vllm_blueprint::VllmLlmClient;
use open_router_blueprint_template_lib::llm::{ChatCompletionRequest, ChatMessage};

async fn example() {
    // Create a new vLLM client
    let client = VllmLlmClient::new(
        "http://localhost:8000".to_string(),  // vLLM server URL
        "meta-llama/Llama-2-7b-chat-hf".to_string()  // Default model
    );

    // Check if the model is available
    let models = client.get_supported_models();
    if models.is_empty() {
        println!("Model not available");
        return;
    }

    // Create a chat completion request
    let request = ChatCompletionRequest {
        model: "meta-llama/Llama-2-7b-chat-hf".to_string(),
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

    // Send the request
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content);
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
```

## Configuration

The vLLM Blueprint can be configured with the following parameters:

- `api_url`: The URL of the vLLM server (e.g., `http://localhost:8000`)
- `model`: The default model to use for requests

## Limitations

- Embeddings are not currently supported in this implementation
- Streaming responses are supported by vLLM but not fully implemented in this blueprint

## Testing

The blueprint includes unit tests and integration tests. To run the unit tests:

```bash
cargo test
```

To run the integration tests (requires a running vLLM server):

```bash
cargo test -- --ignored
```

## License

This blueprint is licensed under the same terms as the OpenRouter Blueprint template.
