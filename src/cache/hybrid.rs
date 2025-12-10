use async_trait::async_trait;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{Cache, CacheStats, LocalCache};
use super::RedisCache;

/// Hybrid cache with L1 (LocalCache) and optional L2 (RedisCache)
///
/// ## Architecture
/// - **L1**: LocalCache - fast in-memory cache
/// - **L2**: Optional RedisCache - distributed persistent cache
///
/// ## Cache Strategy
/// - **get()**: Check L1 first (fast path), on miss check L2, backfill L1 if found in L2
/// - **set()**: Write to both L1 and L2 (ignore L2 errors if Redis unavailable)
/// - **delete()**: Remove from both L1 and L2 (ignore L2 errors if Redis unavailable)
/// - **exists()**: Check L1 first, on miss check L2
/// - **clear()**: Clear both L1 and L2 (ignore L2 errors if Redis unavailable)
/// - **stats()**: Aggregate stats from both layers
///
/// ## Error Handling
/// If Redis is unavailable during operations, log warning and continue with L1 only.
/// This ensures graceful degradation to local-only mode without failing operations.
///
/// ## Example
/// ```ignore
/// use lighter_auth::cache::{HybridCache, LocalCache, RedisCache};
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // L1-only mode (no Redis)
///     let cache = HybridCache::local_only();
///
///     // With Redis (Redis is now always available)
///     {
///         let l1 = LocalCache::new();
///         let l2 = RedisCache::new("redis://localhost:6379", "app").await?;
///         let cache = HybridCache::new(l1, Some(l2));
///     }
///
///     Ok(())
/// }
/// ```
pub struct HybridCache {
    /// L1 cache: fast in-memory
    l1: LocalCache,

    /// L2 cache: optional distributed persistent cache
    l2: Option<RedisCache>,
}

impl HybridCache {
    /// Create a new HybridCache with L1 and optional L2
    ///
    /// # Arguments
    /// * `l1` - LocalCache instance for fast in-memory caching
    /// * `l2` - Optional RedisCache instance for distributed caching
    ///
    /// # Example
    /// ```ignore
    /// use lighter_auth::cache::{HybridCache, LocalCache};
    /// use lighter_auth::cache::RedisCache;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let l1 = LocalCache::new();
    ///     let l2 = RedisCache::new("redis://localhost:6379", "app").await?;
    ///     let cache = HybridCache::new(l1, Some(l2));
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn new(l1: LocalCache, l2: Option<RedisCache>) -> Self {
        Self { l1, l2 }
    }

    /// Create a HybridCache in L1-only mode (no Redis)
    ///
    /// This is useful for testing, development, or single-instance deployments
    /// where distributed caching is not required.
    ///
    /// # Example
    /// ```ignore
    /// use lighter_auth::cache::HybridCache;
    ///
    /// let cache = HybridCache::local_only();
    /// ```
    pub fn local_only() -> Self {
        Self {
            l1: LocalCache::new(),
            l2: None,
        }
    }

    /// Check if L2 cache is available
    fn has_l2(&self) -> bool {
        self.l2.is_some()
    }
}

impl std::fmt::Debug for HybridCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HybridCache")
            .field("l1", &self.l1)
            .field("l2_enabled", &self.has_l2())
            .finish()
    }
}

#[async_trait]
impl Cache for HybridCache {
    #[tracing::instrument(skip(self), fields(cache_key = %key))]
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de> + Serialize + Send + Sync,
    {
        // Check L1 first (fast path)
        match self.l1.get::<V>(key).await {
            Ok(Some(value)) => {
                ::tracing::debug!("L1 cache hit");
                return Ok(Some(value));
            }
            Ok(None) => {
                ::tracing::debug!("L1 cache miss");
            }
            Err(e) => {
                ::tracing::warn!(error = %e, "L1 cache error, continuing");
            }
        }

        // L1 miss, try L2 if available
        if let Some(ref l2) = self.l2 {
            match l2.get::<V>(key).await {
                Ok(Some(value)) => {
                    ::tracing::debug!("L2 cache hit, backfilling L1");

                    // Backfill L1 with value from L2
                    // Use a reasonable TTL for L1 backfill (5 minutes)
                    if let Err(e) = self.l1.set(key, &value, Duration::from_secs(300)).await {
                        ::tracing::warn!(error = %e, "Failed to backfill L1 cache");
                    }

                    return Ok(Some(value));
                }
                Ok(None) => {
                    ::tracing::debug!("L2 cache miss");
                }
                Err(e) => {
                    ::tracing::warn!(error = %e, "L2 cache error (Redis unavailable?), continuing with L1 only");
                }
            }
        }

        // Both L1 and L2 miss
        Ok(None)
    }

    #[tracing::instrument(skip(self, value), fields(cache_key = %key, ttl_secs = ?ttl.as_secs()))]
    async fn set<V>(&self, key: &str, value: &V, ttl: Duration) -> Result<()>
    where
        V: Serialize + Send + Sync,
    {
        // Write to L1
        if let Err(e) = self.l1.set(key, value, ttl).await {
            ::tracing::error!(error = %e, "Failed to write to L1 cache");
            return Err(e).context("Failed to write to L1 cache");
        }
        ::tracing::debug!("Written to L1 cache");

        // Write to L2 if available (best effort, ignore errors)
        if let Some(ref l2) = self.l2 {
            if let Err(e) = l2.set(key, value, ttl).await {
                ::tracing::warn!(error = %e, "Failed to write to L2 cache (Redis unavailable?), continuing with L1 only");
            } else {
                ::tracing::debug!("Written to L2 cache");
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(cache_key = %key))]
    async fn delete(&self, key: &str) -> Result<()> {
        // Delete from L1
        if let Err(e) = self.l1.delete(key).await {
            ::tracing::error!(error = %e, "Failed to delete from L1 cache");
            return Err(e).context("Failed to delete from L1 cache");
        }
        ::tracing::debug!("Deleted from L1 cache");

        // Delete from L2 if available (best effort, ignore errors)
        if let Some(ref l2) = self.l2 {
            if let Err(e) = l2.delete(key).await {
                ::tracing::warn!(error = %e, "Failed to delete from L2 cache (Redis unavailable?), continuing with L1 only");
            } else {
                ::tracing::debug!("Deleted from L2 cache");
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(cache_key = %key))]
    async fn exists(&self, key: &str) -> Result<bool> {
        // Check L1 first (fast path)
        match self.l1.exists(key).await {
            Ok(true) => {
                ::tracing::debug!("Key exists in L1");
                return Ok(true);
            }
            Ok(false) => {
                ::tracing::debug!("Key not in L1");
            }
            Err(e) => {
                ::tracing::warn!(error = %e, "L1 exists check error, continuing");
            }
        }

        // L1 miss, try L2 if available
        if let Some(ref l2) = self.l2 {
            match l2.exists(key).await {
                Ok(exists) => {
                    if exists {
                        ::tracing::debug!("Key exists in L2");
                    } else {
                        ::tracing::debug!("Key not in L2");
                    }
                    return Ok(exists);
                }
                Err(e) => {
                    ::tracing::warn!(error = %e, "L2 exists check error (Redis unavailable?), returning false");
                }
            }
        }

        Ok(false)
    }

    #[tracing::instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        // Clear L1
        if let Err(e) = self.l1.clear().await {
            ::tracing::error!(error = %e, "Failed to clear L1 cache");
            return Err(e).context("Failed to clear L1 cache");
        }
        ::tracing::debug!("Cleared L1 cache");

        // Clear L2 if available (best effort, ignore errors)
        if let Some(ref l2) = self.l2 {
            if let Err(e) = l2.clear().await {
                ::tracing::warn!(error = %e, "Failed to clear L2 cache (Redis unavailable?), continuing with L1 only");
            } else {
                ::tracing::debug!("Cleared L2 cache");
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn stats(&self) -> Result<CacheStats> {
        // Get L1 stats
        let l1_stats = self.l1.stats().await.context("Failed to get L1 stats")?;

        // Get L2 stats if available
        if let Some(ref l2) = self.l2 {
            match l2.stats().await {
                Ok(l2_stats) => {
                    ::tracing::debug!("Aggregating stats from L1 and L2");

                    // Aggregate stats from both layers
                    let mut combined = CacheStats {
                        hits: l1_stats.hits + l2_stats.hits,
                        misses: l1_stats.misses + l2_stats.misses,
                        evictions: l1_stats.evictions + l2_stats.evictions,
                        size: l1_stats.size + l2_stats.size,
                        hit_rate: 0.0,
                    };

                    combined.calculate_hit_rate();
                    return Ok(combined);
                }
                Err(e) => {
                    ::tracing::warn!(error = %e, "Failed to get L2 stats (Redis unavailable?), returning L1 stats only");
                }
            }
        }

        // Return L1 stats only if L2 unavailable or disabled
        Ok(l1_stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_hybrid_cache_new() {
        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, None);

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    async fn test_hybrid_cache_local_only() {
        let cache = HybridCache::local_only();

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 0);
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);
    }

    #[tokio::test]
    async fn test_hybrid_cache_l1_hit() {
        let cache = HybridCache::local_only();

        // Set value in cache
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();

        // Get value (should hit L1)
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Check stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 1);
    }

    #[tokio::test]
    async fn test_hybrid_cache_l1_miss_l2_hit() {
        // Helper to create test Redis instance
        async fn test_redis() -> Option<RedisCache> {
            let url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());

            RedisCache::with_timeout(&url, "test-hybrid", Duration::from_secs(2))
                .await
                .ok()
        }

        let l2 = test_redis().await;

        if l2.is_none() {
            println!("Skipping test: Redis not available");
            return;
        }

        let l2 = l2.unwrap();

        // Clear L2 first
        let _ = l2.clear().await;

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2.clone()));

        // Set value directly in L2 (bypassing HybridCache)
        l2.set("key_l2_only", &"value_from_l2", Duration::from_secs(60)).await.unwrap();

        // Get value (should miss L1, hit L2, then backfill L1)
        let value: Option<String> = cache.get("key_l2_only").await.unwrap();
        assert_eq!(value, Some("value_from_l2".to_string()));

        // Get again (should now hit L1)
        let value: Option<String> = cache.get("key_l2_only").await.unwrap();
        assert_eq!(value, Some("value_from_l2".to_string()));
    }

    #[tokio::test]
    async fn test_hybrid_cache_both_miss() {
        let cache = HybridCache::local_only();

        // Get non-existent key
        let value: Option<String> = cache.get("nonexistent").await.unwrap();
        assert_eq!(value, None);

        // Check stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_hybrid_cache_set_writes_both() {
        async fn test_redis() -> Option<RedisCache> {
            let url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());

            RedisCache::with_timeout(&url, "test-hybrid-set", Duration::from_secs(2))
                .await
                .ok()
        }

        let l2 = test_redis().await;

        if l2.is_none() {
            println!("Skipping test: Redis not available");
            return;
        }

        let l2 = l2.unwrap();
        let _ = l2.clear().await;

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2.clone()));

        // Set value
        cache.set("key_both", &"value_both", Duration::from_secs(60)).await.unwrap();

        // Verify in L1 (via cache.l1)
        let l1_value: Option<String> = cache.l1.get("key_both").await.unwrap();
        assert_eq!(l1_value, Some("value_both".to_string()));

        // Verify in L2 (directly)
        let l2_value: Option<String> = l2.get("key_both").await.unwrap();
        assert_eq!(l2_value, Some("value_both".to_string()));
    }

    #[tokio::test]
    async fn test_hybrid_cache_delete_removes_both() {
        async fn test_redis() -> Option<RedisCache> {
            let url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());

            RedisCache::with_timeout(&url, "test-hybrid-delete", Duration::from_secs(2))
                .await
                .ok()
        }

        let l2 = test_redis().await;

        if l2.is_none() {
            println!("Skipping test: Redis not available");
            return;
        }

        let l2 = l2.unwrap();
        let _ = l2.clear().await;

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2.clone()));

        // Set value in both
        cache.set("key_delete", &"value_delete", Duration::from_secs(60)).await.unwrap();

        // Delete
        cache.delete("key_delete").await.unwrap();

        // Verify removed from L1
        let l1_value: Option<String> = cache.l1.get("key_delete").await.unwrap();
        assert_eq!(l1_value, None);

        // Verify removed from L2
        let l2_value: Option<String> = l2.get("key_delete").await.unwrap();
        assert_eq!(l2_value, None);
    }

    #[tokio::test]
    async fn test_hybrid_cache_clear_clears_both() {
        async fn test_redis() -> Option<RedisCache> {
            let url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());

            RedisCache::with_timeout(&url, "test-hybrid-clear", Duration::from_secs(2))
                .await
                .ok()
        }

        let l2 = test_redis().await;

        if l2.is_none() {
            println!("Skipping test: Redis not available");
            return;
        }

        let l2 = l2.unwrap();
        let _ = l2.clear().await;

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2.clone()));

        // Set multiple values
        cache.set("clear_key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("clear_key2", &"value2", Duration::from_secs(60)).await.unwrap();

        // Clear
        cache.clear().await.unwrap();

        // Verify L1 empty
        let stats = cache.l1.stats().await.unwrap();
        assert_eq!(stats.size, 0);

        // Verify L2 empty (count keys with our prefix)
        let l2_stats = l2.stats().await.unwrap();
        assert_eq!(l2_stats.size, 0);
    }

    #[tokio::test]
    async fn test_hybrid_cache_stats_aggregation() {
        async fn test_redis() -> Option<RedisCache> {
            let url = std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string());

            RedisCache::with_timeout(&url, "test-hybrid-stats", Duration::from_secs(2))
                .await
                .ok()
        }

        let l2 = test_redis().await;

        if l2.is_none() {
            println!("Skipping test: Redis not available");
            return;
        }

        let l2 = l2.unwrap();
        let _ = l2.clear().await;

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2.clone()));

        // Set values
        cache.set("stats_key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("stats_key2", &"value2", Duration::from_secs(60)).await.unwrap();

        // Generate hits and misses
        let _: Option<String> = cache.get("stats_key1").await.unwrap(); // L1 hit
        let _: Option<String> = cache.get("stats_key2").await.unwrap(); // L1 hit
        let _: Option<String> = cache.get("stats_nonexistent").await.unwrap(); // Both miss

        // Get aggregated stats
        let stats = cache.stats().await.unwrap();

        // Should have data from both layers
        assert!(stats.size >= 2); // At least 2 keys
        assert!(stats.hits >= 2); // At least 2 hits
        assert!(stats.misses >= 1); // At least 1 miss
        assert!(stats.hit_rate > 0.0); // Non-zero hit rate
    }

    #[tokio::test]
    #[ignore] // Only run manually when testing Redis unavailability
    async fn test_hybrid_cache_redis_unavailable() {
        // This test simulates Redis being unavailable
        // To test manually:
        // 1. Stop Redis: docker stop redis (or similar)
        // 2. Run: cargo test --features sqlite test_hybrid_cache_redis_unavailable -- --ignored --nocapture
        // 3. Start Redis again after test

        // Try to connect to Redis with 2 second timeout (should fail quickly)
        let l2_result = RedisCache::with_timeout(
            "redis://localhost:9999",
            "test-unavailable",
            Duration::from_secs(2)
        ).await;

        let l1 = LocalCache::new();
        let cache = if l2_result.is_ok() {
            HybridCache::new(l1, Some(l2_result.unwrap()))
        } else {
            println!("Redis unavailable (expected) - error: {:?}", l2_result.err());
            HybridCache::new(l1, None)
        };

        // Operations should still work with L1 only
        cache.set("key_unavailable", &"value", Duration::from_secs(60)).await.unwrap();

        let value: Option<String> = cache.get("key_unavailable").await.unwrap();
        assert_eq!(value, Some("value".to_string()));

        cache.delete("key_unavailable").await.unwrap();

        let value: Option<String> = cache.get("key_unavailable").await.unwrap();
        assert_eq!(value, None);

        // Stats should work
        let _stats = cache.stats().await.unwrap();
    }

    #[tokio::test]
    async fn test_hybrid_cache_concurrent_access() {
        let cache = Arc::new(HybridCache::local_only());

        // Spawn multiple tasks
        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let key = format!("concurrent_key{}", i);
                let value = format!("value{}", i);

                cache_clone.set(&key, &value, Duration::from_secs(60)).await.unwrap();

                let retrieved: Option<String> = cache_clone.get(&key).await.unwrap();
                assert_eq!(retrieved, Some(value));
            });
            handles.push(handle);
        }

        // Wait for all tasks
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all values
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 10);
    }

    #[tokio::test]
    async fn test_hybrid_cache_exists_check() {
        let cache = HybridCache::local_only();

        // Check non-existent key
        assert!(!cache.exists("nonexistent_exists").await.unwrap());

        // Set a value
        cache.set("exists_key", &"value", Duration::from_secs(60)).await.unwrap();

        // Check existing key
        assert!(cache.exists("exists_key").await.unwrap());

        // Delete and check again
        cache.delete("exists_key").await.unwrap();
        assert!(!cache.exists("exists_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_hybrid_cache_debug_trait() {
        let cache = HybridCache::local_only();
        let debug_str = format!("{:?}", cache);

        assert!(debug_str.contains("HybridCache"));
        assert!(debug_str.contains("l1"));
        assert!(debug_str.contains("l2_enabled"));
    }

    #[tokio::test]
    async fn test_hybrid_cache_complex_types() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct TestData {
            id: u64,
            name: String,
            tags: Vec<String>,
        }

        let cache = HybridCache::local_only();

        let data = TestData {
            id: 123,
            name: "Test".to_string(),
            tags: vec!["tag1".to_string(), "tag2".to_string()],
        };

        // Set complex type
        cache.set("complex_key", &data, Duration::from_secs(60)).await.unwrap();

        // Get and verify
        let retrieved: Option<TestData> = cache.get("complex_key").await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    async fn test_hybrid_cache_ttl_expiration() {
        let cache = HybridCache::local_only();

        // Set with short TTL
        cache.set("ttl_key", &"value", Duration::from_millis(100)).await.unwrap();

        // Should exist immediately
        let value: Option<String> = cache.get("ttl_key").await.unwrap();
        assert_eq!(value, Some("value".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        let value: Option<String> = cache.get("ttl_key").await.unwrap();
        assert_eq!(value, None);
    }
}
