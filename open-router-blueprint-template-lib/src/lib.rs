//! OpenRouter Blueprint Template Library
//!
//! This library provides the core functionality for the OpenRouter Blueprint,
//! enabling Tangle to act as a provider on OpenRouter, balancing requests
//! across locally hosted LLMs.

use std::sync::Arc;

// Export our modules
pub mod llm;
pub mod context;
pub mod jobs;
pub mod load_balancer;
pub mod config;

// Re-export key types and functions
pub use context::OpenRouterContext;
pub use jobs::{PROCESS_LLM_REQUEST_JOB_ID, REPORT_METRICS_JOB_ID, process_llm_request, report_metrics};
pub use load_balancer::{LoadBalancer, LoadBalancerConfig, LoadBalancingStrategy};
pub use config::{BlueprintConfig, LlmConfig, ApiConfig, ConfigError, Result as ConfigResult};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BlueprintConfig;
    
    #[test]
    fn test_default_config() {
        let config = BlueprintConfig::default();
        assert_eq!(config.llm.api_url, "http://localhost:8000");
        assert_eq!(config.llm.timeout_seconds, 60);
        assert_eq!(config.llm.max_concurrent_requests, 5);
        assert_eq!(config.load_balancer.max_retries, 3);
        assert_eq!(config.load_balancer.selection_timeout_ms, 1000);
        assert_eq!(config.api.host, "0.0.0.0");
        assert_eq!(config.api.port, 3000);
        assert_eq!(config.api.max_requests_per_minute, 60);
    }
    
    #[test]
    fn test_config_validation() {
        let mut config = BlueprintConfig::default();
        assert!(config.validate().is_ok());
        
        // Test invalid configuration
        config.llm.api_url = "".to_string();
        assert!(config.validate().is_err());
        
        // Reset and test another invalid configuration
        config = BlueprintConfig::default();
        config.llm.timeout_seconds = 0;
        assert!(config.validate().is_err());
    }
}
