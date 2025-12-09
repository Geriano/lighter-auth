use serde::{Deserialize, Serialize};
use lighter_common::config::*;
use super::{AuthConfig, SecurityConfig, ResilienceConfig};

/// Top-level application configuration that aggregates all config modules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Application metadata
    pub app: AppMetadata,
    /// Server configuration (host, port, workers, TLS, HTTP/2)
    pub server: ServerConfig,
    /// Database configuration (connection pool, timeouts, migrations)
    pub database: DatabaseConfig,
    /// Cache configuration (local/redis, TTL, eviction)
    pub cache: CacheConfig,
    /// Metrics configuration (Prometheus settings)
    pub metrics: MetricsConfig,
    /// Observability configuration (tracing, logging)
    pub observability: ObservabilityConfig,
    /// Health check configuration
    pub health: HealthConfig,
    /// Authentication configuration (tokens, Argon2, JWT)
    pub auth: AuthConfig,
    /// Security configuration (CORS, rate limiting, headers)
    pub security: SecurityConfig,
    /// Resilience configuration (circuit breaker, retry)
    pub resilience: ResilienceConfig,
    /// API configuration (versioning, Swagger)
    pub api: ApiConfig,
}

/// Application metadata configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppMetadata {
    /// Application name
    #[serde(default = "default_app_name")]
    pub name: String,
    /// Application version
    #[serde(default = "default_app_version")]
    pub version: String,
    /// Application environment (development, staging, production)
    #[serde(default = "default_environment")]
    pub environment: String,
    /// Graceful shutdown timeout in seconds
    #[serde(default = "default_shutdown_timeout")]
    pub shutdown_timeout: u64,
}

/// API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    /// API version
    #[serde(default = "default_api_version")]
    pub version: String,
    /// Enable Swagger UI
    #[serde(default = "default_swagger_enabled")]
    pub swagger_enabled: bool,
    /// Swagger UI path
    #[serde(default = "default_swagger_path")]
    pub swagger_path: String,
}

// Default functions for AppMetadata
fn default_app_name() -> String {
    "lighter-auth".to_string()
}

fn default_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_environment() -> String {
    "development".to_string()
}

fn default_shutdown_timeout() -> u64 {
    30
}

// Default functions for ApiConfig
fn default_api_version() -> String {
    "v1".to_string()
}

fn default_swagger_enabled() -> bool {
    true
}

fn default_swagger_path() -> String {
    "/docs".to_string()
}

impl Default for AppMetadata {
    fn default() -> Self {
        Self {
            name: default_app_name(),
            version: default_app_version(),
            environment: default_environment(),
            shutdown_timeout: default_shutdown_timeout(),
        }
    }
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            version: default_api_version(),
            swagger_enabled: default_swagger_enabled(),
            swagger_path: default_swagger_path(),
        }
    }
}

impl Validate for AppMetadata {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.name.is_empty() {
            return Err(ConfigError::ValidationError("app.name cannot be empty".to_string()));
        }
        if self.version.is_empty() {
            return Err(ConfigError::ValidationError("app.version cannot be empty".to_string()));
        }
        if self.environment.is_empty() {
            return Err(ConfigError::ValidationError("app.environment cannot be empty".to_string()));
        }
        if self.shutdown_timeout == 0 {
            return Err(ConfigError::ValidationError("app.shutdown_timeout must be > 0".to_string()));
        }
        Ok(())
    }
}

impl Validate for ApiConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        if self.version.is_empty() {
            return Err(ConfigError::ValidationError("api.version cannot be empty".to_string()));
        }
        if self.swagger_enabled && self.swagger_path.is_empty() {
            return Err(ConfigError::ValidationError("api.swagger_path cannot be empty when swagger is enabled".to_string()));
        }
        Ok(())
    }
}

impl Validate for AppConfig {
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate all nested configurations
        self.app.validate()?;
        self.server.validate()?;
        self.database.validate()?;
        self.cache.validate()?;
        self.metrics.validate()?;
        self.observability.validate()?;
        self.health.validate()?;
        self.auth.validate()?;
        self.security.validate()?;
        self.resilience.validate()?;
        self.api.validate()?;
        Ok(())
    }
}

impl WithDefaults for AppConfig {
    fn with_defaults() -> Self {
        Self {
            app: AppMetadata::default(),
            server: ServerConfig::with_defaults(),
            database: DatabaseConfig::with_defaults(),
            cache: CacheConfig::with_defaults(),
            metrics: MetricsConfig::with_defaults(),
            observability: ObservabilityConfig::with_defaults(),
            health: HealthConfig::with_defaults(),
            auth: AuthConfig::with_defaults(),
            security: SecurityConfig::with_defaults(),
            resilience: ResilienceConfig::with_defaults(),
            api: ApiConfig::default(),
        }
    }
}

/// Load configuration from files and environment variables
///
/// Configuration loading follows this precedence (highest to lowest):
/// 1. Environment variables: LIGHTER_AUTH__SERVER__PORT=8080
/// 2. config/local.toml (git-ignored, developer overrides)
/// 3. config/{APP_ENV}.toml (development/staging/production)
/// 4. config/default.toml (base defaults)
pub fn load_config() -> Result<AppConfig, ConfigError> {
    use config::{Config, Environment, File};

    // Determine the environment
    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

    // Build configuration with layered sources
    let config = Config::builder()
        // Layer 1: Base defaults
        .add_source(File::with_name("config/default").required(false))
        // Layer 2: Environment-specific overrides
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        // Layer 3: Local developer overrides (git-ignored)
        .add_source(File::with_name("config/local").required(false))
        // Layer 4: Environment variables (highest precedence)
        .add_source(
            Environment::with_prefix("LIGHTER_AUTH")
                .separator("__")
        )
        .build()?;

    // Deserialize into AppConfig
    let app_config: AppConfig = config.try_deserialize()?;

    // Validate the configuration
    app_config.validate()?;

    Ok(app_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    async fn test_app_metadata_defaults() {
        let metadata = AppMetadata::default();
        assert_eq!(metadata.name, "lighter-auth");
        assert!(!metadata.version.is_empty());
        assert_eq!(metadata.environment, "development");
        assert_eq!(metadata.shutdown_timeout, 30);
    }

    #[test]
    async fn test_api_config_defaults() {
        let config = ApiConfig::default();
        assert_eq!(config.version, "v1");
        assert!(config.swagger_enabled);
        assert_eq!(config.swagger_path, "/docs");
    }

    #[test]
    async fn test_app_metadata_validation_empty_name() {
        let metadata = AppMetadata {
            name: "".to_string(),
            ..AppMetadata::default()
        };
        assert!(metadata.validate().is_err());
    }

    #[test]
    async fn test_app_metadata_validation_zero_shutdown_timeout() {
        let metadata = AppMetadata {
            shutdown_timeout: 0,
            ..AppMetadata::default()
        };
        assert!(metadata.validate().is_err());
    }

    #[test]
    async fn test_api_config_validation_empty_swagger_path() {
        let config = ApiConfig {
            swagger_enabled: true,
            swagger_path: "".to_string(),
            ..ApiConfig::default()
        };
        assert!(config.validate().is_err());
    }
}
