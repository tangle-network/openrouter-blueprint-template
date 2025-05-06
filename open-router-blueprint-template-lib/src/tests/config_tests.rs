//! Tests for the configuration module
//!
//! This module contains tests for the configuration functionality.

use std::path::Path;
use std::fs;
use tempfile::tempdir;

use crate::config::{BlueprintConfig, LlmConfig, ApiConfig, LoadBalancerConfig, LoadBalancingStrategy};

/// Test that verifies the default configuration is valid
#[test]
fn test_default_config() {
    let config = BlueprintConfig::default();
    assert!(config.validate().is_ok());
}

/// Test that verifies configuration validation works correctly
#[test]
fn test_config_validation() {
    // Test with invalid LLM API URL
    let mut config = BlueprintConfig::default();
    config.llm.api_url = "".to_string();
    assert!(config.validate().is_err());
    
    // Test with invalid timeout
    config = BlueprintConfig::default();
    config.llm.timeout_seconds = 0;
    assert!(config.validate().is_err());
    
    // Test with invalid max concurrent requests
    config = BlueprintConfig::default();
    config.llm.max_concurrent_requests = 0;
    assert!(config.validate().is_err());
    
    // Test with invalid load balancer max retries
    config = BlueprintConfig::default();
    config.load_balancer.max_retries = 0;
    assert!(config.validate().is_err());
    
    // Test with invalid API port
    config = BlueprintConfig::default();
    config.api.port = 0;
    assert!(config.validate().is_err());
}

/// Test that verifies loading configuration from a file works correctly
#[test]
fn test_load_config_from_file() {
    // Create a temporary directory
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config.json");
    
    // Create a test configuration
    let config = BlueprintConfig {
        llm: LlmConfig {
            api_url: "http://test-api.com".to_string(),
            timeout_seconds: 30,
            max_concurrent_requests: 10,
            models: vec!["test-model".to_string()],
            additional_params: Default::default(),
        },
        load_balancer: LoadBalancerConfig {
            strategy: LoadBalancingStrategy::RoundRobin,
            max_retries: 5,
            selection_timeout_ms: 2000,
        },
        api: ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_requests_per_minute: 100,
            auth_enabled: true,
            auth_token: Some("test-token".to_string()),
        },
    };
    
    // Write the configuration to a file
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&file_path, json).unwrap();
    
    // Load the configuration from the file
    let loaded_config = BlueprintConfig::load(&file_path).unwrap();
    
    // Verify the loaded configuration matches the original
    assert_eq!(loaded_config.llm.api_url, config.llm.api_url);
    assert_eq!(loaded_config.llm.timeout_seconds, config.llm.timeout_seconds);
    assert_eq!(loaded_config.llm.max_concurrent_requests, config.llm.max_concurrent_requests);
    assert_eq!(loaded_config.llm.models, config.llm.models);
    assert_eq!(loaded_config.load_balancer.strategy, config.load_balancer.strategy);
    assert_eq!(loaded_config.load_balancer.max_retries, config.load_balancer.max_retries);
    assert_eq!(loaded_config.load_balancer.selection_timeout_ms, config.load_balancer.selection_timeout_ms);
    assert_eq!(loaded_config.api.host, config.api.host);
    assert_eq!(loaded_config.api.port, config.api.port);
    assert_eq!(loaded_config.api.max_requests_per_minute, config.api.max_requests_per_minute);
    assert_eq!(loaded_config.api.auth_enabled, config.api.auth_enabled);
    assert_eq!(loaded_config.api.auth_token, config.api.auth_token);
}

/// Test that verifies loading configuration from environment variables works correctly
#[test]
fn test_load_config_from_env() {
    // Set environment variables
    std::env::set_var("OPENROUTER_LLM_API_URL", "http://env-api.com");
    std::env::set_var("OPENROUTER_LLM_TIMEOUT", "45");
    std::env::set_var("OPENROUTER_LLM_MAX_CONCURRENT", "15");
    std::env::set_var("OPENROUTER_LLM_MODELS", "env-model-1,env-model-2");
    std::env::set_var("OPENROUTER_LOAD_BALANCER_STRATEGY", "LeastLoaded");
    std::env::set_var("OPENROUTER_LOAD_BALANCER_MAX_RETRIES", "7");
    std::env::set_var("OPENROUTER_LOAD_BALANCER_TIMEOUT", "3000");
    std::env::set_var("OPENROUTER_API_HOST", "0.0.0.0");
    std::env::set_var("OPENROUTER_API_PORT", "9090");
    std::env::set_var("OPENROUTER_API_MAX_REQUESTS", "200");
    std::env::set_var("OPENROUTER_API_AUTH_ENABLED", "true");
    std::env::set_var("OPENROUTER_API_AUTH_TOKEN", "env-token");
    
    // Load the configuration from environment variables
    let config = BlueprintConfig::from_env();
    
    // Verify the configuration matches the environment variables
    assert_eq!(config.llm.api_url, "http://env-api.com");
    assert_eq!(config.llm.timeout_seconds, 45);
    assert_eq!(config.llm.max_concurrent_requests, 15);
    assert_eq!(config.llm.models, vec!["env-model-1", "env-model-2"]);
    assert_eq!(config.load_balancer.strategy, LoadBalancingStrategy::LeastLoaded);
    assert_eq!(config.load_balancer.max_retries, 7);
    assert_eq!(config.load_balancer.selection_timeout_ms, 3000);
    assert_eq!(config.api.host, "0.0.0.0");
    assert_eq!(config.api.port, 9090);
    assert_eq!(config.api.max_requests_per_minute, 200);
    assert_eq!(config.api.auth_enabled, true);
    assert_eq!(config.api.auth_token, Some("env-token".to_string()));
    
    // Clean up environment variables
    std::env::remove_var("OPENROUTER_LLM_API_URL");
    std::env::remove_var("OPENROUTER_LLM_TIMEOUT");
    std::env::remove_var("OPENROUTER_LLM_MAX_CONCURRENT");
    std::env::remove_var("OPENROUTER_LLM_MODELS");
    std::env::remove_var("OPENROUTER_LOAD_BALANCER_STRATEGY");
    std::env::remove_var("OPENROUTER_LOAD_BALANCER_MAX_RETRIES");
    std::env::remove_var("OPENROUTER_LOAD_BALANCER_TIMEOUT");
    std::env::remove_var("OPENROUTER_API_HOST");
    std::env::remove_var("OPENROUTER_API_PORT");
    std::env::remove_var("OPENROUTER_API_MAX_REQUESTS");
    std::env::remove_var("OPENROUTER_API_AUTH_ENABLED");
    std::env::remove_var("OPENROUTER_API_AUTH_TOKEN");
}

/// Test that verifies environment variables override file configuration
#[test]
fn test_env_overrides_file() {
    // Create a temporary directory
    let dir = tempdir().unwrap();
    let file_path = dir.path().join("config.json");
    
    // Create a test configuration
    let config = BlueprintConfig {
        llm: LlmConfig {
            api_url: "http://file-api.com".to_string(),
            timeout_seconds: 30,
            max_concurrent_requests: 10,
            models: vec!["file-model".to_string()],
            additional_params: Default::default(),
        },
        load_balancer: LoadBalancerConfig {
            strategy: LoadBalancingStrategy::RoundRobin,
            max_retries: 5,
            selection_timeout_ms: 2000,
        },
        api: ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            max_requests_per_minute: 100,
            auth_enabled: false,
            auth_token: None,
        },
    };
    
    // Write the configuration to a file
    let json = serde_json::to_string_pretty(&config).unwrap();
    fs::write(&file_path, json).unwrap();
    
    // Set environment variables
    std::env::set_var("OPENROUTER_LLM_API_URL", "http://override-api.com");
    std::env::set_var("OPENROUTER_API_PORT", "9999");
    
    // Load the configuration with environment overrides
    let loaded_config = BlueprintConfig::load(&file_path).unwrap();
    
    // Verify the environment variables override the file configuration
    assert_eq!(loaded_config.llm.api_url, "http://override-api.com");
    assert_eq!(loaded_config.api.port, 9999);
    
    // Verify the other values are from the file
    assert_eq!(loaded_config.llm.timeout_seconds, config.llm.timeout_seconds);
    assert_eq!(loaded_config.llm.max_concurrent_requests, config.llm.max_concurrent_requests);
    assert_eq!(loaded_config.llm.models, config.llm.models);
    assert_eq!(loaded_config.load_balancer.strategy, config.load_balancer.strategy);
    assert_eq!(loaded_config.load_balancer.max_retries, config.load_balancer.max_retries);
    assert_eq!(loaded_config.load_balancer.selection_timeout_ms, config.load_balancer.selection_timeout_ms);
    assert_eq!(loaded_config.api.host, config.api.host);
    assert_eq!(loaded_config.api.max_requests_per_minute, config.api.max_requests_per_minute);
    assert_eq!(loaded_config.api.auth_enabled, config.api.auth_enabled);
    assert_eq!(loaded_config.api.auth_token, config.api.auth_token);
    
    // Clean up environment variables
    std::env::remove_var("OPENROUTER_LLM_API_URL");
    std::env::remove_var("OPENROUTER_API_PORT");
}
