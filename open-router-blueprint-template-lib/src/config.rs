//! Configuration for the OpenRouter Blueprint
//!
//! This module provides configuration management for the OpenRouter Blueprint,
//! including loading configuration from files and environment variables.

use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, info, warn};

use crate::llm::ModelInfo;
use crate::load_balancer::LoadBalancingStrategy;

/// Errors that can occur when loading configuration
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read configuration file: {0}")]
    FileReadError(#[from] std::io::Error),
    
    #[error("Failed to parse configuration: {0}")]
    ParseError(String),
    
    #[error("Missing required configuration value: {0}")]
    MissingValue(String),
    
    #[error("Invalid configuration value: {0}")]
    InvalidValue(String),
}

/// Result type for configuration operations
pub type Result<T> = std::result::Result<T, ConfigError>;

/// Configuration for the OpenRouter Blueprint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintConfig {
    /// Configuration for the LLM client
    #[serde(default)]
    pub llm: LlmConfig,
    
    /// Configuration for the load balancer
    #[serde(default)]
    pub load_balancer: LoadBalancerConfig,
    
    /// Configuration for the API server
    #[serde(default)]
    pub api: ApiConfig,
    
    /// Additional configuration parameters
    #[serde(default)]
    pub additional_params: HashMap<String, String>,
}

/// Configuration for the LLM client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// The base URL for the LLM API
    #[serde(default = "default_api_url")]
    pub api_url: String,
    
    /// The timeout for API requests in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,
    
    /// The maximum number of concurrent requests
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_requests: usize,
    
    /// The models available on this LLM instance
    #[serde(default)]
    pub models: Vec<ModelInfo>,
    
    /// Additional configuration parameters
    #[serde(default)]
    pub additional_params: HashMap<String, String>,
}

/// Configuration for the load balancer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerConfig {
    /// The load balancing strategy to use
    #[serde(default)]
    pub strategy: LoadBalancingStrategy,
    
    /// Maximum number of retries if a node fails
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    
    /// Timeout for node selection in milliseconds
    #[serde(default = "default_selection_timeout")]
    pub selection_timeout_ms: u64,
}

/// Configuration for the API server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// Whether to enable the API server
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// The host to bind the API server to
    #[serde(default = "default_host")]
    pub host: String,
    
    /// The port to bind the API server to
    #[serde(default = "default_port")]
    pub port: u16,
    
    /// Whether to enable authentication
    #[serde(default = "default_false")]
    pub auth_enabled: bool,
    
    /// The API key for authentication
    #[serde(default)]
    pub api_key: Option<String>,
    
    /// Whether to enable rate limiting
    #[serde(default = "default_true")]
    pub rate_limiting_enabled: bool,
    
    /// The maximum number of requests per minute
    #[serde(default = "default_rate_limit")]
    pub max_requests_per_minute: u32,
    
    /// The interval in seconds for reporting metrics
    #[serde(default = "default_metrics_interval")]
    pub metrics_interval_seconds: u64,
    
    /// The authentication token for API endpoints
    #[serde(default)]
    pub auth_token: Option<String>,
}

impl Default for BlueprintConfig {
    fn default() -> Self {
        Self {
            llm: LlmConfig::default(),
            load_balancer: LoadBalancerConfig::default(),
            api: ApiConfig::default(),
            additional_params: HashMap::new(),
        }
    }
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            api_url: default_api_url(),
            timeout_seconds: default_timeout(),
            max_concurrent_requests: default_max_concurrent(),
            models: default_models(),
            additional_params: HashMap::new(),
        }
    }
}

impl Default for LoadBalancerConfig {
    fn default() -> Self {
        Self {
            strategy: LoadBalancingStrategy::default(),
            max_retries: default_max_retries(),
            selection_timeout_ms: default_selection_timeout(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            enabled: default_true(),
            host: default_host(),
            port: default_port(),
            auth_enabled: default_false(),
            api_key: None,
            rate_limiting_enabled: default_true(),
            max_requests_per_minute: default_rate_limit(),
            metrics_interval_seconds: default_metrics_interval(),
            auth_token: None,
        }
    }
}

impl BlueprintConfig {
    /// Load configuration from a file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Parse the configuration based on the file extension
        let path = path.as_ref();
        let config = if let Some(extension) = path.extension() {
            match extension.to_str() {
                Some("json") => serde_json::from_str(&contents)
                    .map_err(|e| ConfigError::ParseError(format!("Failed to parse JSON: {}", e)))?,
                Some("toml") => toml::from_str(&contents)
                    .map_err(|e| ConfigError::ParseError(format!("Failed to parse TOML: {}", e)))?,
                Some("yaml") | Some("yml") => serde_yaml::from_str(&contents)
                    .map_err(|e| ConfigError::ParseError(format!("Failed to parse YAML: {}", e)))?,
                _ => return Err(ConfigError::ParseError(format!(
                    "Unsupported file extension: {:?}", extension
                ))),
            }
        } else {
            return Err(ConfigError::ParseError(
                "File has no extension".to_string()
            ));
        };
        
        Ok(config)
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::default();
        
        // LLM configuration
        if let Ok(api_url) = std::env::var("OPENROUTER_LLM_API_URL") {
            config.llm.api_url = api_url;
        }
        
        if let Ok(timeout) = std::env::var("OPENROUTER_LLM_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.llm.timeout_seconds = timeout;
            }
        }
        
        if let Ok(max_concurrent) = std::env::var("OPENROUTER_LLM_MAX_CONCURRENT") {
            if let Ok(max_concurrent) = max_concurrent.parse() {
                config.llm.max_concurrent_requests = max_concurrent;
            }
        }
        
        // Load balancer configuration
        if let Ok(strategy) = std::env::var("OPENROUTER_LOAD_BALANCER_STRATEGY") {
            config.load_balancer.strategy = match strategy.to_lowercase().as_str() {
                "round_robin" => LoadBalancingStrategy::RoundRobin,
                "least_loaded" => LoadBalancingStrategy::LeastLoaded,
                "capability_based" => LoadBalancingStrategy::CapabilityBased,
                "latency_based" => LoadBalancingStrategy::LatencyBased,
                _ => config.load_balancer.strategy,
            };
        }
        
        if let Ok(max_retries) = std::env::var("OPENROUTER_LOAD_BALANCER_MAX_RETRIES") {
            if let Ok(max_retries) = max_retries.parse() {
                config.load_balancer.max_retries = max_retries;
            }
        }
        
        if let Ok(timeout) = std::env::var("OPENROUTER_LOAD_BALANCER_TIMEOUT") {
            if let Ok(timeout) = timeout.parse() {
                config.load_balancer.selection_timeout_ms = timeout;
            }
        }
        
        // API configuration
        if let Ok(enabled) = std::env::var("OPENROUTER_API_ENABLED") {
            if let Ok(enabled) = enabled.parse() {
                config.api.enabled = enabled;
            }
        }
        
        if let Ok(host) = std::env::var("OPENROUTER_API_HOST") {
            config.api.host = host;
        }
        
        if let Ok(port) = std::env::var("OPENROUTER_API_PORT") {
            if let Ok(port) = port.parse::<u16>() {
                config.api.port = port;
            } else {
                warn!("Invalid API port in environment variable: {}", port);
            }
        }
        
        if let Ok(auth_enabled) = std::env::var("OPENROUTER_API_AUTH_ENABLED") {
            if let Ok(auth_enabled) = auth_enabled.parse::<bool>() {
                config.api.auth_enabled = auth_enabled;
            } else {
                warn!("Invalid API auth enabled flag in environment variable: {}", auth_enabled);
            }
        }
        
        if let Ok(api_key) = std::env::var("OPENROUTER_API_KEY") {
            config.api.api_key = Some(api_key);
        }
        
        if let Ok(auth_token) = std::env::var("OPENROUTER_API_AUTH_TOKEN") {
            config.api.auth_token = Some(auth_token);
        }
        
        if let Ok(rate_limiting_enabled) = std::env::var("OPENROUTER_API_RATE_LIMITING_ENABLED") {
            if let Ok(rate_limiting_enabled) = rate_limiting_enabled.parse::<bool>() {
                config.api.rate_limiting_enabled = rate_limiting_enabled;
            } else {
                warn!("Invalid API rate limiting enabled flag in environment variable: {}", rate_limiting_enabled);
            }
        }
        
        if let Ok(max_requests) = std::env::var("OPENROUTER_API_MAX_REQUESTS") {
            if let Ok(max_requests) = max_requests.parse::<u32>() {
                config.api.max_requests_per_minute = max_requests;
            } else {
                warn!("Invalid API max requests in environment variable: {}", max_requests);
            }
        }
        
        if let Ok(metrics_interval) = std::env::var("OPENROUTER_API_METRICS_INTERVAL") {
            if let Ok(metrics_interval) = metrics_interval.parse::<u64>() {
                config.api.metrics_interval_seconds = metrics_interval;
            } else {
                warn!("Invalid API metrics interval in environment variable: {}", metrics_interval);
            }
        }
        
        config
    }
    
    /// Load configuration from a file and override with environment variables
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file_config = Self::from_file(path)?;
        let env_config = Self::from_env();
        
        // Merge the configurations, with environment variables taking precedence
        let mut config = file_config;
        
        // Only override if the environment variable was explicitly set
        if env_config.llm.api_url != default_api_url() {
            config.llm.api_url = env_config.llm.api_url;
        }
        
        if env_config.llm.timeout_seconds != default_timeout() {
            config.llm.timeout_seconds = env_config.llm.timeout_seconds;
        }
        
        if env_config.llm.max_concurrent_requests != default_max_concurrent() {
            config.llm.max_concurrent_requests = env_config.llm.max_concurrent_requests;
        }
        
        if env_config.load_balancer.strategy != LoadBalancingStrategy::default() {
            config.load_balancer.strategy = env_config.load_balancer.strategy;
        }
        
        if env_config.load_balancer.max_retries != default_max_retries() {
            config.load_balancer.max_retries = env_config.load_balancer.max_retries;
        }
        
        if env_config.load_balancer.selection_timeout_ms != default_selection_timeout() {
            config.load_balancer.selection_timeout_ms = env_config.load_balancer.selection_timeout_ms;
        }
        
        if env_config.api.enabled != default_true() {
            config.api.enabled = env_config.api.enabled;
        }
        
        if env_config.api.host != default_host() {
            config.api.host = env_config.api.host;
        }
        
        if env_config.api.port != default_port() {
            config.api.port = env_config.api.port;
        }
        
        if env_config.api.auth_enabled != default_false() {
            config.api.auth_enabled = env_config.api.auth_enabled;
        }
        
        if env_config.api.api_key.is_some() {
            config.api.api_key = env_config.api.api_key;
        }
        
        if env_config.api.rate_limiting_enabled != default_true() {
            config.api.rate_limiting_enabled = env_config.api.rate_limiting_enabled;
        }
        
        if env_config.api.max_requests_per_minute != default_rate_limit() {
            config.api.max_requests_per_minute = env_config.api.max_requests_per_minute;
        }
        
        if env_config.api.metrics_interval_seconds != default_metrics_interval() {
            config.api.metrics_interval_seconds = env_config.api.metrics_interval_seconds;
        }
        
        Ok(config)
    }
    
    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate LLM configuration
        if self.llm.api_url.is_empty() {
            return Err(ConfigError::MissingValue("LLM API URL".to_string()));
        }
        
        if self.llm.timeout_seconds == 0 {
            return Err(ConfigError::InvalidValue("LLM timeout must be greater than 0".to_string()));
        }
        
        if self.llm.max_concurrent_requests == 0 {
            return Err(ConfigError::InvalidValue("LLM max concurrent requests must be greater than 0".to_string()));
        }
        
        // Validate load balancer configuration
        if self.load_balancer.max_retries == 0 {
            return Err(ConfigError::InvalidValue("Load balancer max retries must be greater than 0".to_string()));
        }
        
        if self.load_balancer.selection_timeout_ms == 0 {
            return Err(ConfigError::InvalidValue("Load balancer selection timeout must be greater than 0".to_string()));
        }
        
        // Validate API configuration
        if self.api.enabled {
            if self.api.host.is_empty() {
                return Err(ConfigError::MissingValue("API host".to_string()));
            }
            
            if self.api.port == 0 {
                return Err(ConfigError::InvalidValue("API port must be greater than 0".to_string()));
            }
            
            if self.api.auth_enabled && self.api.api_key.is_none() {
                return Err(ConfigError::MissingValue("API key is required when authentication is enabled".to_string()));
            }
            
            if self.api.rate_limiting_enabled && self.api.max_requests_per_minute == 0 {
                return Err(ConfigError::InvalidValue("API max requests per minute must be greater than 0".to_string()));
            }
            
            if self.api.metrics_interval_seconds == 0 {
                return Err(ConfigError::InvalidValue("API metrics interval must be greater than 0".to_string()));
            }
        }
        
        Ok(())
    }
}

// Default values for configuration parameters

fn default_api_url() -> String {
    "http://localhost:8000".to_string()
}

fn default_timeout() -> u64 {
    60
}

fn default_max_concurrent() -> usize {
    5
}

fn default_max_retries() -> usize {
    3
}

fn default_selection_timeout() -> u64 {
    1000
}

fn default_host() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_rate_limit() -> u32 {
    60
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            max_context_length: 4096,
            supports_chat: true,
            supports_text: true,
            supports_embeddings: false,
            parameters: HashMap::new(),
        },
        ModelInfo {
            id: "text-davinci-003".to_string(),
            name: "Text Davinci 003".to_string(),
            max_context_length: 4096,
            supports_chat: false,
            supports_text: true,
            supports_embeddings: false,
            parameters: HashMap::new(),
        },
        ModelInfo {
            id: "text-embedding-ada-002".to_string(),
            name: "Text Embedding Ada 002".to_string(),
            max_context_length: 8191,
            supports_chat: false,
            supports_text: false,
            supports_embeddings: true,
            parameters: HashMap::new(),
        },
    ]
}

fn default_metrics_interval() -> u64 {
    60
}
