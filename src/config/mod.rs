pub mod app;
pub mod auth;
pub mod security;
pub mod resilience;

// Re-export lighter_common config types
pub use lighter_common::config::*;

// Export auth-specific configs
pub use app::{AppConfig, AppMetadata, ApiConfig};
pub use auth::{AuthConfig, Argon2Config, JwtConfig, PasswordHashAlgorithm, JwtAlgorithm};
pub use security::{SecurityConfig, CorsConfig, RateLimitConfig, SecurityHeadersConfig};
pub use resilience::{ResilienceConfig, CircuitBreakerConfig, RetryConfig, RetryBackoff};

/// Load the application configuration from files and environment variables
pub fn load() -> Result<AppConfig, ConfigError> {
    app::load_config()
}
