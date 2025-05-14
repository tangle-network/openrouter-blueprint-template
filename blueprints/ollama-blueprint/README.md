# Ollama Blueprint for OpenRouter

This blueprint implements a concrete LLM client for [Ollama](https://ollama.ai/), a tool for running large language models locally. The blueprint extends the generic OpenRouter Blueprint template with Ollama-specific functionality.

## Features

- Connects to a local Ollama instance via its REST API
- Supports chat and text completions
- Handles error cases and metrics tracking
- Configurable API URL and model selection

## Prerequisites

- [Ollama](https://ollama.ai/) installed and available
- At least one model pulled (e.g., `ollama pull deepseek-r1`)

## Usage

### Configuration

The `OllamaLlmClient` can be configured with:

- `api_url`: The URL of the Ollama API (default: "http://localhost:11434")
- `model`: The name of the model to use (e.g., "deepseek-r1")

### Example

```rust
use ollama_blueprint::OllamaLlmClient;
use open_router_blueprint_template_lib::llm::{ChatCompletionRequest, ChatMessage, LlmClient};

#[tokio::main]
async fn main() {
    // Create a client for the deepseek-r1 model
    let client = OllamaLlmClient::new(
        "http://localhost:11434".to_string(),
        "deepseek-r1".to_string()
    );

    // Create a chat completion request
    let request = ChatCompletionRequest {
        model: "deepseek-r1".to_string(),
        messages: vec![ChatMessage {
            role: "user".to_string(),
            name: None,
            content: "What is the capital of France?".to_string(),
        }],
        max_tokens: None,
        temperature: None,
        top_p: None,
        stream: None,
        additional_params: std::collections::HashMap::new(),
    };

    // Send the request and get the response
    match client.chat_completion(request).await {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content);
        },
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
```

## Testing

The blueprint includes end-to-end tests that verify functionality with a running Ollama instance. Run the tests with:

```bash
cargo test
```

Note: Tests require Ollama to be installed and will attempt to use the "deepseek-r1" model by default. You can override this by setting the `OLLAMA_MODEL` environment variable.

## Integration with OpenRouter

This blueprint can be used as a provider for OpenRouter, allowing it to route requests to your local Ollama instance. Follow the OpenRouter Blueprint template documentation for details on how to deploy and register this blueprint with OpenRouter.
