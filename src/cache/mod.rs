use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::time::Duration;

/// Generic cache trait for storing and retrieving data
#[async_trait]
pub trait Cache: Send + Sync + Debug {
    /// Get a value from the cache
    ///
    /// Returns None if the key doesn't exist or has expired
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de> + Send;

    /// Set a value in the cache with a TTL (time-to-live)
    ///
    /// The value will automatically expire after the specified duration
    async fn set<V>(&self, key: &str, value: &V, ttl: Duration) -> Result<()>
    where
        V: Serialize + Send + Sync;

    /// Delete a key from the cache
    async fn delete(&self, key: &str) -> Result<()>;

    /// Check if a key exists in the cache
    async fn exists(&self, key: &str) -> Result<bool>;

    /// Clear all keys from the cache
    async fn clear(&self) -> Result<()>;

    /// Get cache statistics
    async fn stats(&self) -> Result<CacheStats>;
}

/// Statistics about cache performance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Total number of cache hits (successful gets)
    pub hits: u64,

    /// Total number of cache misses (failed gets)
    pub misses: u64,

    /// Total number of evictions (items removed due to size/memory limits)
    pub evictions: u64,

    /// Current number of items in the cache
    pub size: usize,

    /// Hit rate as a percentage (0.0 to 1.0)
    pub hit_rate: f64,
}

impl CacheStats {
    /// Create new cache stats with zero values
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            size: 0,
            hit_rate: 0.0,
        }
    }

    /// Calculate hit rate from hits and misses
    pub fn calculate_hit_rate(&mut self) {
        let total = self.hits + self.misses;
        self.hit_rate = if total > 0 {
            self.hits as f64 / total as f64
        } else {
            0.0
        };
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for consistent cache key naming
pub struct CacheKey;

impl CacheKey {
    /// Build a user cache key
    pub fn user(id: impl std::fmt::Display) -> String {
        format!("user:{}", id)
    }

    /// Build a token cache key
    pub fn token(id: impl std::fmt::Display) -> String {
        format!("token:{}", id)
    }

    /// Build a permission cache key
    pub fn permission(id: impl std::fmt::Display) -> String {
        format!("permission:{}", id)
    }

    /// Build a role cache key
    pub fn role(id: impl std::fmt::Display) -> String {
        format!("role:{}", id)
    }

    /// Build a session cache key
    pub fn session(id: impl std::fmt::Display) -> String {
        format!("session:{}", id)
    }

    /// Build a user permissions cache key (for caching a user's full permission list)
    pub fn user_permissions(user_id: impl std::fmt::Display) -> String {
        format!("user:{}:permissions", user_id)
    }

    /// Build a user roles cache key (for caching a user's full role list)
    pub fn user_roles(user_id: impl std::fmt::Display) -> String {
        format!("user:{}:roles", user_id)
    }

    /// Build a custom cache key with a prefix
    pub fn custom(prefix: &str, key: impl std::fmt::Display) -> String {
        format!("{}:{}", prefix, key)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    async fn test_cache_key_user() {
        let key = super::CacheKey::user("123e4567-e89b-12d3-a456-426614174000");
        assert_eq!(key, "user:123e4567-e89b-12d3-a456-426614174000");
    }

    #[test]
    async fn test_cache_key_token() {
        let key = super::CacheKey::token("abc123");
        assert_eq!(key, "token:abc123");
    }

    #[test]
    async fn test_cache_key_permission() {
        let key = super::CacheKey::permission("read-users");
        assert_eq!(key, "permission:read-users");
    }

    #[test]
    async fn test_cache_key_role() {
        let key = super::CacheKey::role("admin");
        assert_eq!(key, "role:admin");
    }

    #[test]
    async fn test_cache_key_session() {
        let key = super::CacheKey::session("session-123");
        assert_eq!(key, "session:session-123");
    }

    #[test]
    async fn test_cache_key_user_permissions() {
        let key = super::CacheKey::user_permissions("user-123");
        assert_eq!(key, "user:user-123:permissions");
    }

    #[test]
    async fn test_cache_key_user_roles() {
        let key = super::CacheKey::user_roles("user-456");
        assert_eq!(key, "user:user-456:roles");
    }

    #[test]
    async fn test_cache_key_custom() {
        let key = super::CacheKey::custom("api", "rate-limit");
        assert_eq!(key, "api:rate-limit");
    }

    #[test]
    async fn test_cache_stats_new() {
        let stats = super::CacheStats::new();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[test]
    async fn test_cache_stats_default() {
        let stats = super::CacheStats::default();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[test]
    async fn test_cache_stats_calculate_hit_rate() {
        let mut stats = super::CacheStats {
            hits: 80,
            misses: 20,
            evictions: 0,
            size: 100,
            hit_rate: 0.0,
        };

        stats.calculate_hit_rate();
        assert_eq!(stats.hit_rate, 0.8); // 80 hits / 100 total = 0.8
    }

    #[test]
    async fn test_cache_stats_calculate_hit_rate_zero_total() {
        let mut stats = super::CacheStats::new();
        stats.calculate_hit_rate();
        assert_eq!(stats.hit_rate, 0.0); // No hits or misses = 0.0
    }

    #[test]
    async fn test_cache_stats_calculate_hit_rate_all_misses() {
        let mut stats = super::CacheStats {
            hits: 0,
            misses: 50,
            evictions: 0,
            size: 0,
            hit_rate: 0.0,
        };

        stats.calculate_hit_rate();
        assert_eq!(stats.hit_rate, 0.0); // 0 hits / 50 total = 0.0
    }
}
