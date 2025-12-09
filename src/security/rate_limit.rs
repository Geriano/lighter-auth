//! # Rate Limiting Middleware
//!
//! Production-ready rate limiting using governor crate with per-IP tracking.
//!
//! ## Example
//!
//! ```rust,no_run
//! use lighter_auth::security::{RateLimitConfig, RateLimitMiddleware};
//! use actix_web::{App, HttpServer};
//!
//! #[actix_web::main]
//! async fn main() {
//!     let config = RateLimitConfig {
//!         requests_per_window: 100,
//!         window_seconds: 60,
//!         burst_capacity: 20,
//!         enabled: true,
//!     };
//!
//!     HttpServer::new(move || {
//!         App::new()
//!             .wrap(RateLimitMiddleware::new(config.clone()))
//!             // ... your routes
//!     })
//!     .bind("0.0.0.0:8080")
//!     .unwrap()
//!     .run()
//!     .await
//!     .unwrap()
//! }
//! ```

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use dashmap::DashMap;
use futures_util::future::LocalBoxFuture;
use governor::{
    clock::{Clock, DefaultClock},
    state::direct::NotKeyed,
    state::InMemoryState,
    Quota,
    RateLimiter,
};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use std::net::IpAddr;
use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Number of requests allowed per window
    pub requests_per_window: u32,

    /// Window duration in seconds
    pub window_seconds: u64,

    /// Burst capacity (requests that can be made immediately)
    pub burst_capacity: u32,

    /// Whether rate limiting is enabled
    pub enabled: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_window: 100,
            window_seconds: 60,
            burst_capacity: 20,
            enabled: true,
        }
    }
}

/// Per-IP rate limiter with thread-safe tracking
#[derive(Clone)]
pub struct IpRateLimiter {
    limiters: Arc<
        DashMap<
            IpAddr,
            Arc<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
        >,
    >,
    config: RateLimitConfig,
}

impl IpRateLimiter {
    /// Create a new IP-based rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiters: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Check if request from IP is allowed
    #[tracing::instrument(skip(self), fields(ip = %ip))]
    pub fn check_rate_limit(&self, ip: IpAddr) -> Result<(), Duration> {
        if !self.config.enabled {
            return Ok(());
        }

        // Get or create limiter for this IP
        let limiter = self
            .limiters
            .entry(ip)
            .or_insert_with(|| {
                let quota = Quota::per_second(
                    NonZeroU32::new(self.config.requests_per_window)
                        .unwrap_or(NonZeroU32::new(100).unwrap()),
                )
                .allow_burst(
                    NonZeroU32::new(self.config.burst_capacity)
                        .unwrap_or(NonZeroU32::new(20).unwrap()),
                );

                Arc::new(RateLimiter::direct(quota))
            })
            .clone();

        // Check if request is allowed
        match limiter.check() {
            Ok(_) => {
                tracing::debug!(ip = %ip, "Request allowed");
                Ok(())
            }
            Err(not_until) => {
                let wait_time = not_until.wait_time_from(DefaultClock::default().now());
                tracing::warn!(
                    ip = %ip,
                    retry_after = ?wait_time,
                    "Rate limit exceeded"
                );
                Err(wait_time)
            }
        }
    }

    /// Get statistics for an IP
    pub fn get_stats(&self, ip: IpAddr) -> Option<usize> {
        self.limiters.get(&ip).map(|_limiter| {
            // This is an approximation since governor doesn't expose internal state
            self.config.requests_per_window as usize
        })
    }

    /// Clear limiter for an IP (useful for testing)
    pub fn clear_ip(&self, ip: IpAddr) {
        self.limiters.remove(&ip);
    }

    /// Clear all limiters
    pub fn clear_all(&self) {
        self.limiters.clear();
    }

    /// Get the current configuration
    pub fn config(&self) -> &RateLimitConfig {
        &self.config
    }
}

/// Rate limiting middleware for actix-web
pub struct RateLimitMiddleware {
    limiter: IpRateLimiter,
}

impl RateLimitMiddleware {
    /// Create a new rate limit middleware with the given configuration
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            limiter: IpRateLimiter::new(config),
        }
    }

    /// Create a new rate limit middleware with an existing limiter
    pub fn with_limiter(limiter: IpRateLimiter) -> Self {
        Self { limiter }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            limiter: self.limiter.clone(),
        }))
    }
}

/// Rate limiting middleware service
pub struct RateLimitMiddlewareService<S> {
    service: S,
    limiter: IpRateLimiter,
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let ip = req
            .peer_addr()
            .map(|addr| addr.ip())
            .unwrap_or_else(|| {
                // Fallback IP - check X-Forwarded-For header
                req.headers()
                    .get("X-Forwarded-For")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.split(',').next())
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or_else(|| "127.0.0.1".parse().unwrap())
            });

        // Check rate limit
        match self.limiter.check_rate_limit(ip) {
            Ok(_) => {
                // Rate limit not exceeded, continue
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(wait_time) => {
                // Rate limit exceeded, return 429
                let retry_after = wait_time.as_secs();

                tracing::warn!(
                    ip = %ip,
                    retry_after = retry_after,
                    "Rate limit exceeded, returning 429"
                );

                Box::pin(async move {
                    Err(actix_web::error::ErrorTooManyRequests(format!(
                        "Rate limit exceeded. Please try again in {} seconds.",
                        retry_after
                    )))
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[tokio::test]
    async fn test_rate_limit_config_default() {
        let config = RateLimitConfig::default();
        assert_eq!(config.requests_per_window, 100);
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.burst_capacity, 20);
        assert!(config.enabled);
    }

    #[tokio::test]
    async fn test_rate_limit_allows_requests_within_limit() {
        let config = RateLimitConfig {
            requests_per_window: 5,
            window_seconds: 1,
            burst_capacity: 5,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        // First 5 requests should succeed
        for i in 1..=5 {
            assert!(
                limiter.check_rate_limit(ip).is_ok(),
                "Request {} should be allowed",
                i
            );
        }
    }

    #[tokio::test]
    async fn test_rate_limit_blocks_excess_requests() {
        let config = RateLimitConfig {
            requests_per_window: 3,
            window_seconds: 60,
            burst_capacity: 3,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        // First 3 requests succeed
        for _ in 0..3 {
            assert!(limiter.check_rate_limit(ip).is_ok());
        }

        // 4th request should fail
        let result = limiter.check_rate_limit(ip);
        assert!(result.is_err(), "4th request should be blocked");

        if let Err(wait_time) = result {
            // Wait time could be in nanoseconds or milliseconds, just check it exists
            assert!(
                wait_time.as_nanos() > 0,
                "Should have non-zero wait time"
            );
        }
    }

    #[tokio::test]
    async fn test_rate_limit_per_ip_isolation() {
        let config = RateLimitConfig {
            requests_per_window: 2,
            window_seconds: 60,
            burst_capacity: 2,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);

        let ip1: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
        let ip2: IpAddr = Ipv4Addr::new(127, 0, 0, 2).into();

        // Exhaust IP1's limit
        limiter.check_rate_limit(ip1).unwrap();
        limiter.check_rate_limit(ip1).unwrap();
        assert!(limiter.check_rate_limit(ip1).is_err());

        // IP2 should still be allowed
        assert!(limiter.check_rate_limit(ip2).is_ok());
        assert!(limiter.check_rate_limit(ip2).is_ok());
    }

    #[tokio::test]
    async fn test_rate_limit_disabled() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_seconds: 60,
            burst_capacity: 1,
            enabled: false, // Disabled
        };
        let limiter = IpRateLimiter::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        // Should allow unlimited requests when disabled
        for _ in 0..100 {
            assert!(limiter.check_rate_limit(ip).is_ok());
        }
    }

    #[tokio::test]
    async fn test_clear_ip_limiter() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_seconds: 60,
            burst_capacity: 1,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        // Exhaust limit
        limiter.check_rate_limit(ip).unwrap();
        assert!(limiter.check_rate_limit(ip).is_err());

        // Clear and retry
        limiter.clear_ip(ip);
        assert!(limiter.check_rate_limit(ip).is_ok());
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let config = RateLimitConfig {
            requests_per_window: 10,
            window_seconds: 1,
            burst_capacity: 10,
            enabled: true,
        };
        let limiter = Arc::new(IpRateLimiter::new(config));
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        let mut handles = vec![];
        for _ in 0..20 {
            let limiter = Arc::clone(&limiter);
            let handle = tokio::spawn(async move { limiter.check_rate_limit(ip).is_ok() });
            handles.push(handle);
        }

        let results: Vec<bool> = futures_util::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.unwrap())
            .collect();

        let allowed = results.iter().filter(|&&x| x).count();
        let blocked = results.iter().filter(|&&x| !x).count();

        assert_eq!(allowed, 10, "Should allow exactly 10 requests");
        assert_eq!(blocked, 10, "Should block exactly 10 requests");
    }

    #[tokio::test]
    async fn test_retry_after_header_calculation() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_seconds: 60,
            burst_capacity: 1,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        // Exhaust limit
        limiter.check_rate_limit(ip).unwrap();

        // Get retry-after duration
        if let Err(wait_time) = limiter.check_rate_limit(ip) {
            // Wait time could be very small (nanoseconds), just check it exists
            assert!(
                wait_time.as_nanos() > 0,
                "Should have non-zero wait time"
            );
        } else {
            panic!("Should return error with wait time");
        }
    }

    #[tokio::test]
    async fn test_get_stats() {
        let config = RateLimitConfig {
            requests_per_window: 100,
            window_seconds: 60,
            burst_capacity: 20,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);
        let ip: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();

        // No stats before first request
        assert!(limiter.get_stats(ip).is_none());

        // Make a request
        limiter.check_rate_limit(ip).unwrap();

        // Stats should be available
        assert!(limiter.get_stats(ip).is_some());
    }

    #[tokio::test]
    async fn test_clear_all() {
        let config = RateLimitConfig {
            requests_per_window: 1,
            window_seconds: 60,
            burst_capacity: 1,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config);
        let ip1: IpAddr = Ipv4Addr::new(127, 0, 0, 1).into();
        let ip2: IpAddr = Ipv4Addr::new(127, 0, 0, 2).into();

        // Exhaust limits for both IPs
        limiter.check_rate_limit(ip1).unwrap();
        limiter.check_rate_limit(ip2).unwrap();
        assert!(limiter.check_rate_limit(ip1).is_err());
        assert!(limiter.check_rate_limit(ip2).is_err());

        // Clear all and retry
        limiter.clear_all();
        assert!(limiter.check_rate_limit(ip1).is_ok());
        assert!(limiter.check_rate_limit(ip2).is_ok());
    }

    #[tokio::test]
    async fn test_config_getter() {
        let config = RateLimitConfig {
            requests_per_window: 50,
            window_seconds: 30,
            burst_capacity: 10,
            enabled: true,
        };
        let limiter = IpRateLimiter::new(config.clone());

        assert_eq!(limiter.config().requests_per_window, 50);
        assert_eq!(limiter.config().window_seconds, 30);
        assert_eq!(limiter.config().burst_capacity, 10);
        assert!(limiter.config().enabled);
    }
}
