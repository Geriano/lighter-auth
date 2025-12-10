use serde::{Deserialize, Serialize};
use lighter_common::config::*;

/// Resilience configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct ResilienceConfig {
    /// Circuit breaker configuration
    #[serde(default = "CircuitBreakerConfig::default")]
    pub circuit_breaker: CircuitBreakerConfig,
    /// Retry configuration
    #[serde(default = "RetryConfig::default")]
    pub retry: RetryConfig,
}

/// Circuit breaker configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Enable circuit breaker
    #[serde(default = "default_circuit_breaker_enabled")]
    pub enabled: bool,
    /// Number of failures before opening the circuit
    #[serde(default = "default_circuit_breaker_threshold")]
    pub threshold: u32,
    /// Timeout in seconds before attempting to close the circuit
    #[serde(default = "default_circuit_breaker_timeout")]
    pub timeout: u64,
    /// Number of successful requests needed to close the circuit
    #[serde(default = "default_circuit_breaker_success_threshold")]
    pub success_threshold: u32,
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Enable retry
    #[serde(default = "default_retry_enabled")]
    pub enabled: bool,
    /// Maximum number of retry attempts
    #[serde(default = "default_retry_max_attempts")]
    pub max_attempts: u32,
    /// Backoff strategy
    #[serde(default = "default_retry_backoff")]
    pub backoff: RetryBackoff,
    /// Initial delay in milliseconds
    #[serde(default = "default_retry_initial_delay")]
    pub initial_delay: u64,
    /// Maximum delay in milliseconds
    #[serde(default = "default_retry_max_delay")]
    pub max_delay: u64,
    /// Multiplier for exponential backoff
    #[serde(default = "default_retry_multiplier")]
    pub multiplier: f64,
}

/// Retry backoff strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RetryBackoff {
    /// Exponential backoff (delay *= multiplier)
    Exponential,
    /// Linear backoff (delay += initial_delay)
    Linear,
    /// Constant backoff (delay = initial_delay)
    Constant,
}

// Default functions for CircuitBreakerConfig
fn default_circuit_breaker_enabled() -> bool {
    true
}

fn default_circuit_breaker_threshold() -> u32 {
    5
}

fn default_circuit_breaker_timeout() -> u64 {
    60 // 1 minute
}

fn default_circuit_breaker_success_threshold() -> u32 {
    2
}

// Default functions for RetryConfig
fn default_retry_enabled() -> bool {
    true
}

fn default_retry_max_attempts() -> u32 {
    3
}

fn default_retry_backoff() -> RetryBackoff {
    RetryBackoff::Exponential
}

fn default_retry_initial_delay() -> u64 {
    100 // milliseconds
}

fn default_retry_max_delay() -> u64 {
    10000 // milliseconds
}

fn default_retry_multiplier() -> f64 {
    2.0
}


impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            enabled: default_circuit_breaker_enabled(),
            threshold: default_circuit_breaker_threshold(),
            timeout: default_circuit_breaker_timeout(),
            success_threshold: default_circuit_breaker_success_threshold(),
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            enabled: default_retry_enabled(),
            max_attempts: default_retry_max_attempts(),
            backoff: default_retry_backoff(),
            initial_delay: default_retry_initial_delay(),
            max_delay: default_retry_max_delay(),
            multiplier: default_retry_multiplier(),
        }
    }
}

impl Validate for ResilienceConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        self.circuit_breaker.validate()?;
        self.retry.validate()?;
        Ok(())
    }
}

impl Validate for CircuitBreakerConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.threshold == 0 {
            return Err(ConfigError::ValidationError("resilience.circuit_breaker.threshold must be > 0 when circuit breaker is enabled".to_string()));
        }
        if self.enabled && self.timeout == 0 {
            return Err(ConfigError::ValidationError("resilience.circuit_breaker.timeout must be > 0 when circuit breaker is enabled".to_string()));
        }
        if self.enabled && self.success_threshold == 0 {
            return Err(ConfigError::ValidationError("resilience.circuit_breaker.success_threshold must be > 0 when circuit breaker is enabled".to_string()));
        }
        Ok(())
    }
}

impl Validate for RetryConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.max_attempts == 0 {
            return Err(ConfigError::ValidationError("resilience.retry.max_attempts must be > 0 when retry is enabled".to_string()));
        }
        if self.enabled && self.initial_delay == 0 {
            return Err(ConfigError::ValidationError("resilience.retry.initial_delay must be > 0 when retry is enabled".to_string()));
        }
        if self.enabled && self.max_delay == 0 {
            return Err(ConfigError::ValidationError("resilience.retry.max_delay must be > 0 when retry is enabled".to_string()));
        }
        if self.enabled && self.initial_delay > self.max_delay {
            return Err(ConfigError::ValidationError("resilience.retry.initial_delay must be <= max_delay".to_string()));
        }
        if self.enabled && self.multiplier <= 0.0 {
            return Err(ConfigError::ValidationError("resilience.retry.multiplier must be > 0.0 when retry is enabled".to_string()));
        }
        Ok(())
    }
}

impl WithDefaults for ResilienceConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for CircuitBreakerConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for RetryConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    async fn test_circuit_breaker_config_defaults() {
        let config = CircuitBreakerConfig::default();
        assert!(config.enabled);
        assert_eq!(config.threshold, 5);
        assert_eq!(config.timeout, 60);
        assert_eq!(config.success_threshold, 2);
    }

    #[test]
    async fn test_retry_config_defaults() {
        let config = RetryConfig::default();
        assert!(config.enabled);
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.backoff, RetryBackoff::Exponential);
        assert_eq!(config.initial_delay, 100);
        assert_eq!(config.max_delay, 10000);
        assert_eq!(config.multiplier, 2.0);
    }

    #[test]
    async fn test_circuit_breaker_config_validation_zero_threshold() {
        let config = CircuitBreakerConfig {
            enabled: true,
            threshold: 0,
            ..CircuitBreakerConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_circuit_breaker_config_validation_zero_timeout() {
        let config = CircuitBreakerConfig {
            enabled: true,
            timeout: 0,
            ..CircuitBreakerConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_retry_config_validation_zero_max_attempts() {
        let config = RetryConfig {
            enabled: true,
            max_attempts: 0,
            ..RetryConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_retry_config_validation_zero_initial_delay() {
        let config = RetryConfig {
            enabled: true,
            initial_delay: 0,
            ..RetryConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_retry_config_validation_initial_delay_greater_than_max() {
        let config = RetryConfig {
            enabled: true,
            initial_delay: 10000,
            max_delay: 100,
            ..RetryConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_retry_config_validation_zero_multiplier() {
        let config = RetryConfig {
            enabled: true,
            multiplier: 0.0,
            ..RetryConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_retry_config_validation_negative_multiplier() {
        let config = RetryConfig {
            enabled: true,
            multiplier: -1.0,
            ..RetryConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
