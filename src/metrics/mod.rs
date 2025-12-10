pub mod middleware;

pub use middleware::MetricsMiddleware;

use metrics::{counter, describe_counter, describe_gauge, describe_histogram, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::sync::{Arc, OnceLock};

static PROMETHEUS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

#[derive(Clone)]
pub struct AppMetrics {
    prometheus_handle: Arc<PrometheusHandle>,
}

impl AppMetrics {
    pub fn new() -> Self {
        Self::with_config(None)
    }

    pub fn with_config(config: Option<&crate::config::AppConfig>) -> Self {
        let handle = PROMETHEUS_HANDLE.get_or_init(|| {
            let builder = PrometheusBuilder::new();

            // Add global labels from config
            let builder = if let Some(cfg) = config {
                builder
                    .add_global_label("service", cfg.app.name.clone())
                    .add_global_label("version", cfg.app.version.clone())
                    .add_global_label("environment", cfg.app.environment.clone())
                    .add_global_label("instance", cfg.observability.service_instance_id.clone())
            } else {
                builder
            };

            let builder = builder
                .set_buckets_for_metric(
                    Matcher::Full("http_requests_duration_seconds".to_string()),
                    &[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0],
                )
                .expect("Failed to set buckets for http_requests_duration_seconds")
                .set_buckets_for_metric(
                    Matcher::Full("db_queries_duration_seconds".to_string()),
                    &[0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0],
                )
                .expect("Failed to set buckets for db_queries_duration_seconds");

            // Describe all metrics
            Self::describe_metrics();

            builder
                .install_recorder()
                .expect("Failed to install Prometheus recorder")
        });

        Self {
            prometheus_handle: Arc::new(handle.clone()),
        }
    }

    fn describe_metrics() {
        // HTTP metrics
        describe_counter!(
            "http_requests_total",
            "Total number of HTTP requests"
        );
        describe_histogram!(
            "http_requests_duration_seconds",
            "HTTP request duration in seconds"
        );
        describe_gauge!(
            "http_requests_in_flight",
            "Number of HTTP requests currently being processed"
        );

        // Database metrics
        describe_counter!(
            "database_queries_total",
            "Total number of database queries"
        );
        describe_histogram!(
            "database_queries_duration_seconds",
            "Database query duration in seconds"
        );
        describe_gauge!(
            "database_connections_active",
            "Number of active database connections"
        );

        // Cache metrics
        describe_counter!("cache_hits_total", "Total number of cache hits");
        describe_counter!("cache_misses_total", "Total number of cache misses");
        describe_gauge!(
            "cache_size",
            "Current number of items in cache"
        );

        // Auth metrics
        describe_counter!(
            "auth_login_attempts_total",
            "Total number of login attempts"
        );
        describe_gauge!(
            "auth_tokens_active",
            "Number of currently active authentication tokens"
        );

        // Business metrics
        describe_gauge!(
            "users_total",
            "Total number of users in the system"
        );

        // System metrics
        describe_gauge!("system_cpu_usage", "CPU usage percentage");
        describe_gauge!("system_memory_usage", "Memory usage in bytes");
    }

    // HTTP metrics
    pub fn record_http_request(&self, method: &str, path: &str, status: u16, duration_secs: f64) {
        counter!(
            "http_requests_total",
            "method" => method.to_string(),
            "path" => path.to_string(),
            "status" => status.to_string()
        )
        .increment(1);

        histogram!(
            "http_requests_duration_seconds",
            "method" => method.to_string(),
            "path" => path.to_string()
        )
        .record(duration_secs);
    }

    pub fn http_request_start(&self) {
        gauge!("http_requests_in_flight").increment(1.0);
    }

    pub fn http_request_end(&self) {
        gauge!("http_requests_in_flight").decrement(1.0);
    }

    // Database metrics
    pub fn record_db_query(&self, operation: &str, duration_secs: f64) {
        ::tracing::debug!("Metrics: Recording database query operation={}, duration={:.4}s", operation, duration_secs);
        counter!("database_queries_total", "operation" => operation.to_string()).increment(1);
        histogram!("database_queries_duration_seconds", "operation" => operation.to_string())
            .record(duration_secs);
        ::tracing::debug!("Metrics: Database query metric recorded successfully");
    }

    pub fn set_db_connections(&self, count: usize) {
        gauge!("database_connections_active").set(count as f64);
    }

    // Cache metrics
    pub fn record_cache_hit(&self) {
        counter!("cache_hits_total").increment(1);
    }

    pub fn record_cache_miss(&self) {
        counter!("cache_misses_total").increment(1);
    }

    pub fn set_cache_size(&self, size: usize) {
        gauge!("cache_size").set(size as f64);
    }

    // Auth metrics
    pub fn record_login_attempt(&self, success: bool) {
        let status = if success { "true" } else { "false" };
        counter!("auth_login_attempts_total", "success" => status.to_string()).increment(1);
    }

    pub fn set_active_tokens(&self, count: usize) {
        gauge!("auth_tokens_active").set(count as f64);
    }

    // Business metrics
    pub fn set_users_total(&self, count: usize) {
        gauge!("users_total").set(count as f64);
    }

    // System metrics
    pub fn set_cpu_usage(&self, percentage: f64) {
        gauge!("system_cpu_usage").set(percentage);
    }

    pub fn set_memory_usage(&self, bytes: u64) {
        gauge!("system_memory_usage").set(bytes as f64);
    }

    // Prometheus export
    pub fn render(&self) -> String {
        self.prometheus_handle.render()
    }
}

impl Default for AppMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    async fn test_metrics_creation() {
        let metrics = super::AppMetrics::new();
        let output = metrics.render();
        assert!(!output.is_empty());
    }

    #[test]
    async fn test_http_metrics() {
        let metrics = super::AppMetrics::new();

        metrics.http_request_start();
        metrics.record_http_request("GET", "/api/v1/users", 200, 0.050);
        metrics.http_request_end();

        let output = metrics.render();
        assert!(output.contains("http_requests_total"));
        assert!(output.contains("http_requests_duration_seconds"));
    }

    #[test]
    async fn test_database_metrics() {
        let metrics = super::AppMetrics::new();

        metrics.record_db_query("SELECT", 0.010);
        metrics.set_db_connections(10);

        let output = metrics.render();
        assert!(output.contains("db_queries_total"));
        assert!(output.contains("db_connections_active"));
    }

    #[test]
    async fn test_cache_metrics() {
        let metrics = super::AppMetrics::new();

        metrics.record_cache_hit();
        metrics.record_cache_miss();
        metrics.set_cache_size(100);

        let output = metrics.render();
        assert!(output.contains("cache_hits_total"));
        assert!(output.contains("cache_misses_total"));
        assert!(output.contains("cache_size"));
    }

    #[test]
    async fn test_auth_metrics() {
        let metrics = super::AppMetrics::new();

        metrics.record_login_attempt(true);
        metrics.record_login_attempt(false);
        metrics.set_active_tokens(50);

        let output = metrics.render();
        assert!(output.contains("auth_login_attempts_total"));
        assert!(output.contains("auth_tokens_active"));
    }

    #[test]
    async fn test_business_metrics() {
        let metrics = super::AppMetrics::new();

        metrics.set_users_total(1000);

        let output = metrics.render();
        assert!(output.contains("users_total"));
    }

    #[test]
    async fn test_system_metrics() {
        let metrics = super::AppMetrics::new();

        metrics.set_cpu_usage(45.5);
        metrics.set_memory_usage(1024 * 1024 * 512); // 512 MB

        let output = metrics.render();
        assert!(output.contains("system_cpu_usage"));
        assert!(output.contains("system_memory_usage"));
    }

    #[test]
    async fn test_prometheus_format() {
        let metrics = super::AppMetrics::new();

        metrics.record_http_request("GET", "/health", 200, 0.001);

        let output = metrics.render();

        // Check Prometheus format - verify metrics are present
        assert!(output.contains("http_requests_total"));
        assert!(output.contains("http_requests_duration_seconds"));
        // Verify metric values and labels are formatted
        assert!(output.contains("method="));
        assert!(output.contains("path="));
        assert!(output.contains("status="));
    }

    #[test]
    async fn test_metric_descriptions() {
        let metrics = super::AppMetrics::new();

        // Record some metrics to ensure they appear in output
        metrics.record_http_request("GET", "/test", 200, 0.001);
        metrics.record_db_query("SELECT", 0.005);
        metrics.record_cache_hit();
        metrics.record_login_attempt(true);
        metrics.set_users_total(100);

        let output = metrics.render();

        // Verify metrics are present in output
        assert!(output.contains("http_requests_total"));
        assert!(output.contains("db_queries_total"));
        assert!(output.contains("cache_hits_total"));
        assert!(output.contains("auth_login_attempts_total"));
        assert!(output.contains("users_total"));
    }
}
