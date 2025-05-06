# OpenRouter Blueprint Template

A Tangle Blueprint template for creating OpenRouter providers that can balance requests across locally hosted LLMs.

## Overview

This template provides a framework for creating LLM providers on OpenRouter that can balance requests across locally hosted LLMs. It is designed to be completely general and not tied to any specific LLM implementation, allowing you to create blueprints for any local LLM.

The template includes:

- Generic interfaces for interacting with LLMs
- Load balancing across multiple LLM nodes
- Streaming support for efficient handling of large responses
- Metrics collection for informed load balancing decisions

## Architecture

The OpenRouter Blueprint template follows a modular architecture:

- **LLM Interface**: Defines standard interfaces and data models for interacting with LLMs
- **Load Balancer**: Distributes requests across multiple LLM nodes based on various strategies
- **Context**: Manages shared state and provides access to LLM clients
- **Job Handlers**: Process requests from Tangle and return responses

## Documentation

- [Configuration Guide](CONFIGURATION.md): Detailed guide on configuring the OpenRouter Blueprint
- [Deployment Guide](DEPLOYMENT.md): Instructions for deploying the OpenRouter Blueprint

## Features

- **Generic LLM Interface**: Standardized interface for any LLM implementation
- **Load Balancing**: Built-in support for distributing requests across multiple LLM nodes
- **Streaming Support**: Framework for handling streaming responses from LLMs
- **Metrics Collection**: Standard metrics tracking for load balancing decisions
- **Configuration Management**: Flexible configuration via files and environment variables
- **Comprehensive Testing**: Robust test suite for ensuring reliability
- **Security Features**: Authentication and rate limiting for API endpoints

## Getting Started

### Prerequisites

- Rust toolchain (1.70+)
- Tangle node (for deployment)

### Installation

1. Clone this repository:

```bash
git clone https://github.com/your-username/open-router-blueprint-template.git
cd open-router-blueprint-template
```

2. Build the project:

```bash
cargo build --release
```

3. Create a configuration file:

```bash
cp config.sample.json config.json
```

4. Edit the configuration file to match your environment.

5. Run the blueprint:

```bash
./target/release/open-router-blueprint-template-bin --config config.json
```

For more detailed configuration options, see the [Configuration Guide](CONFIGURATION.md).

## Extending the Template

This template is designed to be extended for specific LLM implementations. Here's how to create a blueprint for your specific LLM:

1. **Create a new blueprint** based on this template
2. **Implement the LLM client** by implementing the `LlmClient` trait for your specific LLM
3. **Add streaming support** by implementing the `StreamingLlmClient` trait (optional)
4. **Update the context** to initialize your LLM client
5. **Configure the load balancer** to use your preferred strategy

### Example: Implementing a Custom LLM Client

```rust
use async_trait::async_trait;
use open_router_blueprint_template_lib::llm::{LlmClient, Result, ChatCompletionRequest, ChatCompletionResponse, /* ... */};

pub struct MyCustomLlmClient {
    // Your client-specific fields
}

#[async_trait]
impl LlmClient for MyCustomLlmClient {
    fn get_supported_models(&self) -> Vec<ModelInfo> {
        // Return the models supported by your LLM
    }

    fn get_capabilities(&self) -> LlmCapabilities {
        // Return the capabilities of your LLM
    }

    fn get_metrics(&self) -> NodeMetrics {
        // Return metrics for your LLM
    }

    async fn chat_completion(&self, request: ChatCompletionRequest) -> Result<ChatCompletionResponse> {
        // Implement chat completion for your LLM
    }

    // Implement other required methods...
}

## Load Balancing Strategies

The template supports several load balancing strategies:

- **RoundRobin**: Distributes requests evenly across all nodes
- **LeastLoaded**: Routes requests to the node with the fewest active requests
- **CapabilityBased**: Selects nodes based on their capabilities for specific models
- **LatencyBased**: Routes requests to the node with the lowest response time

## Testing

The OpenRouter Blueprint includes a comprehensive test suite to ensure reliability and correctness:

```bash
# Run all tests
cargo test

# Run specific tests
cargo test config_tests
cargo test load_balancer_tests
cargo test llm_tests
```

The test suite includes:

- **Unit Tests**: Tests for individual components
- **Integration Tests**: Tests for component interactions
- **Mock Implementations**: Mock LLM clients for testing without external dependencies

## License

This project is licensed under the [MIT License](LICENSE).
