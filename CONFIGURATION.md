# OpenRouter Blueprint Configuration Guide

This guide explains how to configure the OpenRouter Blueprint template for your specific LLM implementation. The template supports configuration through both JSON files and environment variables.

## Configuration Methods

The OpenRouter Blueprint supports three methods of configuration, in order of precedence:

1. **Environment Variables**: These override all other settings
2. **Configuration File**: Loaded at startup, can be specified with the `--config` flag
3. **Default Values**: Used when no configuration is provided

## Configuration File

A sample configuration file is provided in `config.sample.json`. You can copy this file and modify it to suit your needs:

```bash
cp config.sample.json config.json
```

Then edit `config.json` with your preferred settings.

## Environment Variables

You can override any configuration setting using environment variables. The following environment variables are supported:

### LLM Configuration

- `OPENROUTER_LLM_API_URL`: The base URL for the LLM API
- `OPENROUTER_LLM_TIMEOUT`: Timeout for API requests in seconds
- `OPENROUTER_LLM_MAX_CONCURRENT`: Maximum number of concurrent requests
- `OPENROUTER_LLM_MODELS`: Comma-separated list of model IDs

### Load Balancer Configuration

- `OPENROUTER_LOAD_BALANCER_STRATEGY`: The load balancing strategy (`RoundRobin`, `LeastLoaded`, or `Random`)
- `OPENROUTER_LOAD_BALANCER_MAX_RETRIES`: Maximum number of retries if a node fails
- `OPENROUTER_LOAD_BALANCER_TIMEOUT`: Timeout for node selection in milliseconds

### API Configuration

- `OPENROUTER_API_ENABLED`: Whether to enable the API server
- `OPENROUTER_API_HOST`: The host to bind the API server to
- `OPENROUTER_API_PORT`: The port to bind the API server to
- `OPENROUTER_API_AUTH_ENABLED`: Whether to enable authentication
- `OPENROUTER_API_KEY`: The API key for authentication
- `OPENROUTER_API_AUTH_TOKEN`: The authentication token for API endpoints
- `OPENROUTER_API_RATE_LIMITING_ENABLED`: Whether to enable rate limiting
- `OPENROUTER_API_MAX_REQUESTS`: The maximum number of requests per minute
- `OPENROUTER_API_METRICS_INTERVAL`: The interval in seconds for reporting metrics

## Configuration Structure

The configuration is structured into the following sections:

### LLM Configuration

This section configures the LLM client:

```json
"llm": {
  "api_url": "http://localhost:8000",
  "timeout_seconds": 60,
  "max_concurrent_requests": 5,
  "models": [
    {
      "id": "model-id",
      "name": "Model Name",
      "max_context_length": 4096,
      "supports_chat": true,
      "supports_text": true,
      "supports_embeddings": false,
      "parameters": {}
    }
  ],
  "additional_params": {}
}
```

- `api_url`: The base URL for the LLM API
- `timeout_seconds`: Timeout for API requests in seconds
- `max_concurrent_requests`: Maximum number of concurrent requests
- `models`: List of models available on this LLM instance
  - `id`: The model ID
  - `name`: The human-readable name of the model
  - `max_context_length`: The maximum context length in tokens
  - `supports_chat`: Whether the model supports chat completions
  - `supports_text`: Whether the model supports text completions
  - `supports_embeddings`: Whether the model supports embeddings
  - `parameters`: Additional model-specific parameters
- `additional_params`: Additional configuration parameters for the LLM client

### Load Balancer Configuration

This section configures the load balancer:

```json
"load_balancer": {
  "strategy": "LeastLoaded",
  "max_retries": 3,
  "selection_timeout_ms": 1000
}
```

- `strategy`: The load balancing strategy to use
  - `RoundRobin`: Distribute requests evenly across all nodes
  - `LeastLoaded`: Send requests to the node with the lowest load
  - `Random`: Randomly select a node for each request
- `max_retries`: Maximum number of retries if a node fails
- `selection_timeout_ms`: Timeout for node selection in milliseconds

### API Configuration

This section configures the API server:

```json
"api": {
  "enabled": true,
  "host": "0.0.0.0",
  "port": 3000,
  "auth_enabled": false,
  "api_key": null,
  "auth_token": null,
  "rate_limiting_enabled": true,
  "max_requests_per_minute": 60,
  "metrics_interval_seconds": 60
}
```

- `enabled`: Whether to enable the API server
- `host`: The host to bind the API server to
- `port`: The port to bind the API server to
- `auth_enabled`: Whether to enable authentication
- `api_key`: The API key for authentication
- `auth_token`: The authentication token for API endpoints
- `rate_limiting_enabled`: Whether to enable rate limiting
- `max_requests_per_minute`: The maximum number of requests per minute
- `metrics_interval_seconds`: The interval in seconds for reporting metrics

### Additional Parameters

You can add custom configuration parameters in the `additional_params` section:

```json
"additional_params": {
  "custom_param1": "value1",
  "custom_param2": "value2"
}
```

## Extending the Configuration

When implementing your own LLM provider, you may need to extend the configuration to include additional settings. Here's how to do that:

1. Create a new configuration struct that extends the base configuration:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyLlmConfig {
    #[serde(flatten)]
    pub base: LlmConfig,
    
    // Add your custom fields here
    pub custom_field: String,
}
```

2. Implement the `Default` trait for your configuration:

```rust
impl Default for MyLlmConfig {
    fn default() -> Self {
        Self {
            base: LlmConfig::default(),
            custom_field: "default_value".to_string(),
        }
    }
}
```

3. Update the `from_env` and `load` methods to handle your custom fields:

```rust
impl MyLlmConfig {
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // Load base configuration
        let base_config = LlmConfig::from_env();
        config.base = base_config;
        
        // Load custom fields
        if let Ok(value) = std::env::var("MY_LLM_CUSTOM_FIELD") {
            config.custom_field = value;
        }
        
        config
    }
    
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        // Load from file
        let file_config = Self::from_file(path)?;
        
        // Override with environment variables
        let env_config = Self::from_env();
        
        // Create the final config
        let mut config = file_config;
        
        // Override base fields
        // ...
        
        // Override custom fields
        if env_config.custom_field != Self::default().custom_field {
            config.custom_field = env_config.custom_field;
        }
        
        Ok(config)
    }
}
```

## Runtime Configuration Reloading

The OpenRouter Blueprint supports reloading the configuration at runtime without restarting the service. This is useful for updating settings like API keys or rate limits without downtime.

To reload the configuration, you can call the `reload_config` method on the `OpenRouterContext`:

```rust
// Reload the configuration
let result = context.reload_config().await;
if let Err(e) = result {
    error!("Failed to reload configuration: {}", e);
} else {
    info!("Configuration reloaded successfully");
}
```

## Best Practices

1. **Use Environment Variables for Secrets**: Never store sensitive information like API keys in configuration files. Use environment variables instead.

2. **Validate Configuration**: Always validate the configuration before using it. The `validate` method checks for required fields and valid values.

3. **Provide Sensible Defaults**: Always provide sensible default values for all configuration parameters.

4. **Document Configuration Options**: Document all configuration options in your README or a separate configuration guide.

5. **Use Configuration Profiles**: Consider using different configuration profiles for development, testing, and production environments.

## Troubleshooting

If you encounter issues with your configuration:

1. **Check Environment Variables**: Make sure your environment variables are set correctly.

2. **Validate Configuration File**: Ensure your configuration file is valid JSON.

3. **Check Logs**: Look for error messages in the logs that might indicate configuration issues.

4. **Try Default Configuration**: If all else fails, try running with the default configuration to see if the issue is related to your custom settings.
