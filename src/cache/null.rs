use async_trait::async_trait;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::Duration;

use super::{Cache, CacheStats};

/// A no-op cache implementation for testing purposes
///
/// `NullCache` implements the `Cache` trait but performs no actual caching operations.
/// All operations are no-ops and return empty/false values. This is useful for testing
/// scenarios where cache functionality is not needed or should be disabled.
///
/// # Use Cases
/// - Unit testing components that depend on a cache without needing to mock
/// - Testing to ensure application works correctly without cache
/// - Development environments where caching is not required
/// - Scenarios where you want to measure performance impact of caching
///
/// # Example
/// ```no_run
/// use lighter_auth::cache::NullCache;
/// use std::time::Duration;
///
/// # async fn example() -> anyhow::Result<()> {
/// let cache = NullCache::new();
///
/// // get() always returns None
/// let value: Option<String> = cache.get("key").await?;
/// assert_eq!(value, None);
///
/// // set() does nothing
/// cache.set("key", &"value", Duration::from_secs(60)).await?;
///
/// // exists() always returns false
/// let exists = cache.exists("key").await?;
/// assert_eq!(exists, false);
///
/// // stats() returns empty statistics
/// let stats = cache.stats().await?;
/// assert_eq!(stats.hits, 0);
/// assert_eq!(stats.misses, 0);
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct NullCache;

impl NullCache {
    /// Create a new `NullCache` instance
    ///
    /// # Example
    /// ```
    /// use lighter_auth::cache::NullCache;
    ///
    /// let cache = NullCache::new();
    /// ```
    pub fn new() -> Self {
        NullCache
    }
}

impl Default for NullCache {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NullCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NullCache")
    }
}

#[async_trait]
impl Cache for NullCache {
    /// Get a value from the cache
    ///
    /// Always returns `Ok(None)` since no values are stored
    async fn get<V>(&self, _key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de> + Serialize + Send + Sync,
    {
        Ok(None)
    }

    /// Set a value in the cache
    ///
    /// Does nothing and returns `Ok(())`
    async fn set<V>(&self, _key: &str, _value: &V, _ttl: Duration) -> Result<()>
    where
        V: Serialize + Send + Sync,
    {
        Ok(())
    }

    /// Delete a key from the cache
    ///
    /// Does nothing and returns `Ok(())`
    async fn delete(&self, _key: &str) -> Result<()> {
        Ok(())
    }

    /// Check if a key exists in the cache
    ///
    /// Always returns `Ok(false)` since no values are stored
    async fn exists(&self, _key: &str) -> Result<bool> {
        Ok(false)
    }

    /// Clear all keys from the cache
    ///
    /// Does nothing and returns `Ok(())`
    async fn clear(&self) -> Result<()> {
        Ok(())
    }

    /// Get cache statistics
    ///
    /// Returns `Ok(CacheStats::new())` with all zero values
    async fn stats(&self) -> Result<CacheStats> {
        Ok(CacheStats::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_cache_new() {
        let cache = NullCache::new();
        assert_eq!(cache.to_string(), "NullCache");
    }

    #[tokio::test]
    async fn test_null_cache_get_returns_none() {
        let cache = NullCache::new();
        let value: Option<String> = cache.get("test_key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_null_cache_set_does_nothing() {
        let cache = NullCache::new();
        let result = cache.set("test_key", &"test_value", Duration::from_secs(60)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_null_cache_delete_does_nothing() {
        let cache = NullCache::new();
        let result = cache.delete("test_key").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_null_cache_exists_returns_false() {
        let cache = NullCache::new();
        let exists = cache.exists("test_key").await.unwrap();
        assert_eq!(exists, false);
    }

    #[tokio::test]
    async fn test_null_cache_clear_does_nothing() {
        let cache = NullCache::new();
        let result = cache.clear().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_null_cache_stats_returns_empty() {
        let cache = NullCache::new();
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.evictions, 0);
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hit_rate, 0.0);
    }

    #[tokio::test]
    async fn test_null_cache_default() {
        let cache = NullCache::default();
        let value: Option<String> = cache.get("key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_null_cache_multiple_operations() {
        let cache = NullCache::new();

        // Multiple sets
        for i in 0..10 {
            let key = format!("key_{}", i);
            cache.set(&key, &"value", Duration::from_secs(60)).await.unwrap();
        }

        // Multiple gets
        for i in 0..10 {
            let key = format!("key_{}", i);
            let value: Option<String> = cache.get(&key).await.unwrap();
            assert_eq!(value, None);
        }

        // Stats should still be empty
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 0);
    }
}
