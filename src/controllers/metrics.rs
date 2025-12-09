use actix_web::{get, web, HttpResponse, Responder};
use crate::metrics::AppMetrics;

/// Metrics endpoint for Prometheus scraping
///
/// Returns metrics in Prometheus text format
#[get("/metrics")]
pub async fn metrics(metrics: web::Data<AppMetrics>) -> impl Responder {
    let output = metrics.render();

    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};
    use crate::metrics::AppMetrics;

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let app_metrics = AppMetrics::new();

        // Record some test metrics
        app_metrics.record_http_request("GET", "/test", 200, 0.05);
        app_metrics.set_users_total(100);

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_metrics))
                .service(metrics)
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = test::call_service(&app, req).await;

        // Check status code
        assert_eq!(resp.status().as_u16(), 200);

        // Check content type
        let content_type = resp.headers().get("content-type").unwrap();
        assert_eq!(content_type, "text/plain; version=0.0.4");

        // Get body
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Verify Prometheus format - metrics are present
        assert!(!body_str.is_empty(), "Metrics output should not be empty");

        // Verify our test metrics are present
        assert!(body_str.contains("http_requests_total"), "Should contain http_requests_total metric");
        assert!(body_str.contains("users_total"), "Should contain users_total metric");
    }

    #[actix_web::test]
    async fn test_metrics_prometheus_format() {
        let app_metrics = AppMetrics::new();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_metrics))
                .service(metrics)
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Verify Prometheus format - check for basic metric structure
        assert!(!body_str.is_empty(), "Metrics output should not be empty");

        // Check that the output is in Prometheus text format
        // Note: metrics-exporter-prometheus may not include metrics with zero values
        // We just verify that the output looks like valid Prometheus format
        assert!(
            body_str.contains("http_requests_total") || body_str.contains("http_requests_duration_seconds"),
            "Should contain at least one HTTP metrics"
        );
    }

    #[actix_web::test]
    async fn test_metrics_multiple_calls() {
        let app_metrics = AppMetrics::new();

        // Record multiple metrics
        app_metrics.record_http_request("GET", "/api/v1/users", 200, 0.01);
        app_metrics.record_http_request("POST", "/api/v1/users", 201, 0.05);
        app_metrics.record_cache_hit();
        app_metrics.record_cache_miss();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_metrics))
                .service(metrics)
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/metrics")
            .to_request();

        let resp = test::call_service(&app, req).await;
        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();

        // Verify all recorded metrics appear
        assert!(body_str.contains("http_requests_total"));
        assert!(body_str.contains("cache_hits_total"));
        assert!(body_str.contains("cache_misses_total"));
    }
}
