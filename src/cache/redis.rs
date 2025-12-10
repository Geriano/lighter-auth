use async_trait::async_trait;
use anyhow::{Context, Result};
use redis::aio::ConnectionManager;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

use super::{Cache, CacheStats};

/// Redis-backed cache with async support and connection pooling
///
/// Features:
/// - Async operations using redis::aio::ConnectionManager
/// - Automatic connection pooling and reconnection
/// - Key prefixing for namespace isolation
/// - Bincode serialization for efficient binary storage
/// - TTL support with SET EX commands
/// - Comprehensive error handling
/// - Metrics tracking (hits, misses, evictions)
#[derive(Clone)]
pub struct RedisCache {
    /// Redis client for creating connections
    #[allow(dead_code)]
    client: Client,
    /// Connection manager for pooled async operations
    conn_manager: Arc<ConnectionManager>,
    /// Key prefix for namespace isolation (e.g., "lighter-auth:")
    prefix: String,
    /// Cache hit counter
    hits: Arc<AtomicU64>,
    /// Cache miss counter
    misses: Arc<AtomicU64>,
    /// Eviction counter (manual deletes + expirations)
    evictions: Arc<AtomicU64>,
}

impl RedisCache {
    /// Create a new RedisCache with a connection string
    ///
    /// Uses a default timeout of 3 seconds for connection attempts.
    ///
    /// # Arguments
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `prefix` - Key prefix for namespace isolation (e.g., "lighter-auth")
    ///
    /// # Example
    /// ```no_run
    /// use lighter_auth::cache::RedisCache;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let cache = RedisCache::new("redis://localhost:6379", "lighter-auth").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn new(url: &str, prefix: &str) -> Result<Self> {
        Self::with_timeout(url, prefix, Duration::from_secs(3)).await
    }

    /// Create a new RedisCache with a connection string and custom timeout
    ///
    /// # Arguments
    /// * `url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `prefix` - Key prefix for namespace isolation (e.g., "lighter-auth")
    /// * `connection_timeout` - Maximum time to wait for connection
    ///
    /// # Example
    /// ```no_run
    /// use lighter_auth::cache::RedisCache;
    /// use std::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let cache = RedisCache::with_timeout(
    ///         "redis://localhost:6379",
    ///         "lighter-auth",
    ///         Duration::from_secs(5)
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn with_timeout(
        url: &str,
        prefix: &str,
        connection_timeout: Duration,
    ) -> Result<Self> {
        let client = Client::open(url)
            .context("Failed to create Redis client")?;

        let conn_manager = timeout(
            connection_timeout,
            ConnectionManager::new(client.clone())
        )
        .await
        .context(format!(
            "Redis connection timeout after {:?}. Check Redis is running at: {}",
            connection_timeout, url
        ))?
        .context("Failed to create Redis connection manager")?;

        Ok(Self {
            client,
            conn_manager: Arc::new(conn_manager),
            prefix: format!("{}:", prefix),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Create a new RedisCache from an existing client
    ///
    /// Uses a default timeout of 3 seconds for connection attempts.
    ///
    /// # Arguments
    /// * `client` - Redis client
    /// * `prefix` - Key prefix for namespace isolation
    pub async fn from_client(client: Client, prefix: &str) -> Result<Self> {
        Self::from_client_with_timeout(client, prefix, Duration::from_secs(3)).await
    }

    /// Create a new RedisCache from an existing client with custom timeout
    ///
    /// # Arguments
    /// * `client` - Redis client
    /// * `prefix` - Key prefix for namespace isolation
    /// * `connection_timeout` - Maximum time to wait for connection
    pub async fn from_client_with_timeout(
        client: Client,
        prefix: &str,
        connection_timeout: Duration,
    ) -> Result<Self> {
        let conn_manager = timeout(
            connection_timeout,
            ConnectionManager::new(client.clone())
        )
        .await
        .context(format!("Connection manager timeout after {:?}", connection_timeout))?
        .context("Failed to create connection manager")?;

        Ok(Self {
            client,
            conn_manager: Arc::new(conn_manager),
            prefix: format!("{}:", prefix),
            hits: Arc::new(AtomicU64::new(0)),
            misses: Arc::new(AtomicU64::new(0)),
            evictions: Arc::new(AtomicU64::new(0)),
        })
    }

    /// Build the full cache key with prefix
    fn build_key(&self, key: &str) -> String {
        format!("{}{}", self.prefix, key)
    }

    /// Get the connection manager (clone for use)
    fn connection(&self) -> ConnectionManager {
        (*self.conn_manager).clone()
    }
}

impl std::fmt::Debug for RedisCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisCache")
            .field("prefix", &self.prefix)
            .field("hits", &self.hits)
            .field("misses", &self.misses)
            .field("evictions", &self.evictions)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl Cache for RedisCache {
    #[tracing::instrument(skip(self), fields(cache_key = %key))]
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de> + Serialize + Send + Sync,
    {
        let full_key = self.build_key(key);
        let mut conn = self.connection();

        // Get binary data from Redis
        let data: Option<Vec<u8>> = conn
            .get(&full_key)
            .await
            .context("Failed to get value from Redis")?;

        match data {
            Some(bytes) => {
                // Deserialize using bincode
                let value: V = bincode::deserialize(&bytes)
                    .context("Failed to deserialize cached value")?;

                self.hits.fetch_add(1, Ordering::Relaxed);
                ::tracing::debug!("Cache hit");
                Ok(Some(value))
            }
            None => {
                self.misses.fetch_add(1, Ordering::Relaxed);
                ::tracing::debug!("Cache miss");
                Ok(None)
            }
        }
    }

    #[tracing::instrument(skip(self, value), fields(cache_key = %key, ttl_secs = ?ttl.as_secs()))]
    async fn set<V>(&self, key: &str, value: &V, ttl: Duration) -> Result<()>
    where
        V: Serialize + Send + Sync,
    {
        let full_key = self.build_key(key);
        let mut conn = self.connection();

        // Serialize value using bincode
        let data = bincode::serialize(value)
            .context("Failed to serialize value")?;

        // Set with expiration using SET EX
        let ttl_secs = ttl.as_secs();
        let _: () = conn
            .set_ex(&full_key, data, ttl_secs)
            .await
            .context("Failed to set value in Redis")?;

        ::tracing::debug!("Cache value set with TTL");
        Ok(())
    }

    #[tracing::instrument(skip(self), fields(cache_key = %key))]
    async fn delete(&self, key: &str) -> Result<()> {
        let full_key = self.build_key(key);
        let mut conn = self.connection();

        let deleted: u32 = conn
            .del(&full_key)
            .await
            .context("Failed to delete value from Redis")?;

        if deleted > 0 {
            self.evictions.fetch_add(deleted as u64, Ordering::Relaxed);
            ::tracing::debug!("Cache value deleted");
        }

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(cache_key = %key))]
    async fn exists(&self, key: &str) -> Result<bool> {
        let full_key = self.build_key(key);
        let mut conn = self.connection();

        let exists: bool = conn
            .exists(&full_key)
            .await
            .context("Failed to check if key exists in Redis")?;

        Ok(exists)
    }

    #[tracing::instrument(skip(self))]
    async fn clear(&self) -> Result<()> {
        let mut conn = self.connection();

        // Get all keys with our prefix
        let pattern = format!("{}*", self.prefix);
        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .context("Failed to get keys from Redis")?;

        if keys.is_empty() {
            ::tracing::info!("No keys to clear");
            return Ok(());
        }

        let count = keys.len();

        // Delete all keys
        let deleted: u32 = conn
            .del(&keys)
            .await
            .context("Failed to clear keys from Redis")?;

        self.evictions.fetch_add(deleted as u64, Ordering::Relaxed);
        ::tracing::info!(cleared_entries = count, "Cache cleared");

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn stats(&self) -> Result<CacheStats> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);

        let mut conn = self.connection();

        // Get count of keys with our prefix
        let pattern = format!("{}*", self.prefix);
        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .context("Failed to get key count from Redis")?;

        let size = keys.len();

        let mut stats = CacheStats {
            hits,
            misses,
            evictions,
            size,
            hit_rate: 0.0,
        };

        stats.calculate_hit_rate();

        Ok(stats)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // Helper to create a test Redis instance
    // Note: These tests require a running Redis instance
    async fn test_redis() -> RedisCache {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        RedisCache::new(&url, "test")
            .await
            .expect("Failed to create Redis cache")
    }

    #[tokio::test]
    #[ignore] // Ignore by default, run with --ignored flag when Redis is available
    async fn test_redis_cache_new() {
        let cache = test_redis().await;
        let _stats = cache.stats().await.unwrap();
        // Stats retrieved successfully
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_set_and_get() {
        let cache = test_redis().await;

        // Clear any existing test data
        cache.clear().await.unwrap();

        // Set a value
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();

        // Get the value
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Check stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 0);
        assert_eq!(stats.size, 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_get_nonexistent() {
        let cache = test_redis().await;

        let value: Option<String> = cache.get("nonexistent_key_12345").await.unwrap();
        assert_eq!(value, None);

        // Check stats
        let stats = cache.stats().await.unwrap();
        assert!(stats.misses >= 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_delete() {
        let cache = test_redis().await;

        // Set and delete
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.delete("key1").await.unwrap();

        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_exists() {
        let cache = test_redis().await;

        // Check non-existent key
        assert!(!cache.exists("nonexistent_exists_test").await.unwrap());

        // Set a value
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();

        // Check existing key
        assert!(cache.exists("key1").await.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_clear() {
        let cache = test_redis().await;

        // Set multiple values
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("key2", &"value2", Duration::from_secs(60)).await.unwrap();
        cache.set("key3", &"value3", Duration::from_secs(60)).await.unwrap();

        // Clear cache
        cache.clear().await.unwrap();

        // Verify all gone
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_ttl_expiration() {
        let cache = test_redis().await;

        // Set a value with short TTL
        cache.set("ttl_test", &"value1", Duration::from_millis(100)).await.unwrap();

        // Should exist immediately
        let value: Option<String> = cache.get("ttl_test").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired and return None
        let value: Option<String> = cache.get("ttl_test").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_concurrent_access() {
        let cache = Arc::new(test_redis().await);

        // Clear any existing test data
        cache.clear().await.unwrap();

        // Spawn multiple tasks that access the cache concurrently
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

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify all values are in cache
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 10);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_hit_rate() {
        let cache = test_redis().await;

        // Clear to get accurate stats
        cache.clear().await.unwrap();

        // Reset stats by creating a new instance
        let cache = test_redis().await;

        // Set some values
        cache.set("hr_key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("hr_key2", &"value2", Duration::from_secs(60)).await.unwrap();

        // 2 hits
        let _: Option<String> = cache.get("hr_key1").await.unwrap();
        let _: Option<String> = cache.get("hr_key2").await.unwrap();

        // 1 miss
        let _: Option<String> = cache.get("hr_key3").await.unwrap();

        // Check hit rate (2/3 = 0.666...)
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.6666).abs() < 0.001);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_key_prefix() {
        let cache1 = RedisCache::new("redis://localhost:6379", "app1")
            .await
            .unwrap();
        let cache2 = RedisCache::new("redis://localhost:6379", "app2")
            .await
            .unwrap();

        // Set same key in different namespaces
        cache1.set("shared_key", &"value1", Duration::from_secs(60)).await.unwrap();
        cache2.set("shared_key", &"value2", Duration::from_secs(60)).await.unwrap();

        // Verify isolation
        let value1: Option<String> = cache1.get("shared_key").await.unwrap();
        let value2: Option<String> = cache2.get("shared_key").await.unwrap();

        assert_eq!(value1, Some("value1".to_string()));
        assert_eq!(value2, Some("value2".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_serialization_complex_types() {
        use serde::{Deserialize, Serialize};

        #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
        struct User {
            id: u64,
            name: String,
            email: String,
            roles: Vec<String>,
        }

        let cache = test_redis().await;

        let user = User {
            id: 123,
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            roles: vec!["admin".to_string(), "user".to_string()],
        };

        // Set complex type
        cache.set("user:123", &user, Duration::from_secs(60)).await.unwrap();

        // Get and verify
        let retrieved: Option<User> = cache.get("user:123").await.unwrap();
        assert_eq!(retrieved, Some(user));
    }
}
