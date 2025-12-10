use serde::{Deserialize, Serialize};
use lighter_common::config::*;

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct SecurityConfig {
    /// CORS configuration
    #[serde(default = "CorsConfig::default")]
    pub cors: CorsConfig,
    /// Rate limiting configuration
    #[serde(default = "RateLimitConfig::default")]
    pub rate_limit: RateLimitConfig,
    /// Security headers configuration
    #[serde(default = "SecurityHeadersConfig::default")]
    pub headers: SecurityHeadersConfig,
}

/// CORS (Cross-Origin Resource Sharing) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    #[serde(default = "default_cors_enabled")]
    pub enabled: bool,
    /// Allowed origins (e.g., ["https://example.com", "*"])
    #[serde(default = "default_cors_origins")]
    pub origins: Vec<String>,
    /// Allowed HTTP methods
    #[serde(default = "default_cors_methods")]
    pub methods: Vec<String>,
    /// Allowed HTTP headers
    #[serde(default = "default_cors_headers")]
    pub headers: Vec<String>,
    /// Max age in seconds for preflight requests
    #[serde(default = "default_cors_max_age")]
    pub max_age: u64,
    /// Allow credentials (cookies, authorization headers)
    #[serde(default = "default_cors_allow_credentials")]
    pub allow_credentials: bool,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Enable rate limiting
    #[serde(default = "default_rate_limit_enabled")]
    pub enabled: bool,
    /// Number of requests allowed per window
    #[serde(default = "default_rate_limit_requests")]
    pub requests: u32,
    /// Time window in seconds
    #[serde(default = "default_rate_limit_window")]
    pub window: u64,
    /// Burst capacity (additional requests allowed temporarily)
    #[serde(default = "default_rate_limit_burst")]
    pub burst: u32,
}

/// Security headers configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeadersConfig {
    /// Enable security headers
    #[serde(default = "default_headers_enabled")]
    pub enabled: bool,
    /// Content Security Policy
    #[serde(default = "default_headers_csp")]
    pub csp: String,
    /// HSTS max age in seconds
    #[serde(default = "default_headers_hsts_max_age")]
    pub hsts_max_age: u64,
    /// X-Frame-Options
    #[serde(default = "default_headers_x_frame_options")]
    pub x_frame_options: String,
    /// X-Content-Type-Options
    #[serde(default = "default_headers_x_content_type_options")]
    pub x_content_type_options: String,
    /// Referrer-Policy
    #[serde(default = "default_headers_referrer_policy")]
    pub referrer_policy: String,
}

// Default functions for CorsConfig
fn default_cors_enabled() -> bool {
    true
}

fn default_cors_origins() -> Vec<String> {
    vec!["*".to_string()]
}

fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "PATCH".to_string(),
    ]
}

fn default_cors_headers() -> Vec<String> {
    vec![
        "Authorization".to_string(),
        "Content-Type".to_string(),
    ]
}

fn default_cors_max_age() -> u64 {
    3600 // 1 hour
}

fn default_cors_allow_credentials() -> bool {
    false
}

// Default functions for RateLimitConfig
fn default_rate_limit_enabled() -> bool {
    true
}

fn default_rate_limit_requests() -> u32 {
    100
}

fn default_rate_limit_window() -> u64 {
    60 // 1 minute
}

fn default_rate_limit_burst() -> u32 {
    20
}

// Default functions for SecurityHeadersConfig
fn default_headers_enabled() -> bool {
    true
}

fn default_headers_csp() -> String {
    "default-src 'self'".to_string()
}

fn default_headers_hsts_max_age() -> u64 {
    31536000 // 1 year
}

fn default_headers_x_frame_options() -> String {
    "DENY".to_string()
}

fn default_headers_x_content_type_options() -> String {
    "nosniff".to_string()
}

fn default_headers_referrer_policy() -> String {
    "strict-origin-when-cross-origin".to_string()
}


impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: default_cors_enabled(),
            origins: default_cors_origins(),
            methods: default_cors_methods(),
            headers: default_cors_headers(),
            max_age: default_cors_max_age(),
            allow_credentials: default_cors_allow_credentials(),
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: default_rate_limit_enabled(),
            requests: default_rate_limit_requests(),
            window: default_rate_limit_window(),
            burst: default_rate_limit_burst(),
        }
    }
}

impl Default for SecurityHeadersConfig {
    fn default() -> Self {
        Self {
            enabled: default_headers_enabled(),
            csp: default_headers_csp(),
            hsts_max_age: default_headers_hsts_max_age(),
            x_frame_options: default_headers_x_frame_options(),
            x_content_type_options: default_headers_x_content_type_options(),
            referrer_policy: default_headers_referrer_policy(),
        }
    }
}

impl Validate for SecurityConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        self.cors.validate()?;
        self.rate_limit.validate()?;
        self.headers.validate()?;
        Ok(())
    }
}

impl Validate for CorsConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.origins.is_empty() {
            return Err(ConfigError::ValidationError("security.cors.origins cannot be empty when CORS is enabled".to_string()));
        }
        if self.max_age == 0 {
            return Err(ConfigError::ValidationError("security.cors.max_age must be > 0".to_string()));
        }
        Ok(())
    }
}

impl Validate for RateLimitConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.requests == 0 {
            return Err(ConfigError::ValidationError("security.rate_limit.requests must be > 0 when rate limiting is enabled".to_string()));
        }
        if self.enabled && self.window == 0 {
            return Err(ConfigError::ValidationError("security.rate_limit.window must be > 0 when rate limiting is enabled".to_string()));
        }
        Ok(())
    }
}

impl Validate for SecurityHeadersConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.enabled && self.csp.is_empty() {
            return Err(ConfigError::ValidationError("security.headers.csp cannot be empty when security headers are enabled".to_string()));
        }
        if self.hsts_max_age == 0 {
            return Err(ConfigError::ValidationError("security.headers.hsts_max_age must be > 0".to_string()));
        }
        Ok(())
    }
}

impl WithDefaults for SecurityConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for CorsConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for RateLimitConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

impl WithDefaults for SecurityHeadersConfig {
    fn with_defaults() -> Self {
        Self::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    async fn test_cors_config_defaults() {
        let config = CorsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.origins, vec!["*"]);
        assert_eq!(config.methods.len(), 5);
        assert_eq!(config.headers.len(), 2);
        assert_eq!(config.max_age, 3600);
        assert!(!config.allow_credentials);
    }

    #[test]
    async fn test_rate_limit_config_defaults() {
        let config = RateLimitConfig::default();
        assert!(config.enabled);
        assert_eq!(config.requests, 100);
        assert_eq!(config.window, 60);
        assert_eq!(config.burst, 20);
    }

    #[test]
    async fn test_security_headers_config_defaults() {
        let config = SecurityHeadersConfig::default();
        assert!(config.enabled);
        assert_eq!(config.csp, "default-src 'self'");
        assert_eq!(config.hsts_max_age, 31536000);
        assert_eq!(config.x_frame_options, "DENY");
        assert_eq!(config.x_content_type_options, "nosniff");
        assert_eq!(config.referrer_policy, "strict-origin-when-cross-origin");
    }

    #[test]
    async fn test_cors_config_validation_empty_origins() {
        let config = CorsConfig {
            enabled: true,
            origins: vec![],
            ..CorsConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_rate_limit_config_validation_zero_requests() {
        let config = RateLimitConfig {
            enabled: true,
            requests: 0,
            ..RateLimitConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_rate_limit_config_validation_zero_window() {
        let config = RateLimitConfig {
            enabled: true,
            window: 0,
            ..RateLimitConfig::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    async fn test_security_headers_config_validation_empty_csp() {
        let config = SecurityHeadersConfig {
            enabled: true,
            csp: "".to_string(),
            ..SecurityHeadersConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
