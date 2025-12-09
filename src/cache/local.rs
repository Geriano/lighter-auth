use async_trait::async_trait;
use anyhow::{Context, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

use super::{Cache, CacheStats};

/// Internal cache entry with expiration
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Serialized data using bincode
    data: Vec<u8>,
    /// Expiration timestamp
    expires_at: Instant,
}

impl CacheEntry {
    /// Create a new cache entry with TTL
    fn new(data: Vec<u8>, ttl: Duration) -> Self {
        Self {
            data,
            expires_at: Instant::now() + ttl,
        }
    }

    /// Check if the entry has expired
    fn is_expired(&self) -> bool {
        Instant::now() >= self.expires_at
    }
}

/// Local in-memory cache using DashMap
#[derive(Debug)]
pub struct LocalCache {
    /// DashMap storage with configurable shards
    store: Arc<DashMap<String, CacheEntry>>,
    /// Cache hit counter
    hits: Arc<AtomicU64>,
    /// Cache miss counter
    misses: Arc<AtomicU64>,
    /// Eviction counter
    evictions: Arc<AtomicU64>,
    /// Background cleanup task handle
    cleanup_handle: Option<JoinHandle<()>>,
}

impl LocalCache {
    /// Create a new LocalCache with default shard count (CPU count * 4)
    pub fn new() -> Self {
        Self::with_shard_count(num_cpus::get() * 4)
    }

    /// Create a new LocalCache with specific shard count
    pub fn with_shard_count(shard_count: usize) -> Self {
        let store = Arc::new(DashMap::with_shard_amount(shard_count));
        let hits = Arc::new(AtomicU64::new(0));
        let misses = Arc::new(AtomicU64::new(0));
        let evictions = Arc::new(AtomicU64::new(0));

        // Start background cleanup task
        let cleanup_handle = Self::start_cleanup_task(
            Arc::clone(&store),
            Arc::clone(&evictions),
        );

        Self {
            store,
            hits,
            misses,
            evictions,
            cleanup_handle: Some(cleanup_handle),
        }
    }

    /// Start background cleanup task that runs every 60 seconds
    fn start_cleanup_task(
        store: Arc<DashMap<String, CacheEntry>>,
        evictions: Arc<AtomicU64>,
    ) -> JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Remove expired entries
                let mut expired_keys = Vec::new();

                for entry in store.iter() {
                    if entry.value().is_expired() {
                        expired_keys.push(entry.key().clone());
                    }
                }

                for key in expired_keys {
                    store.remove(&key);
                    evictions.fetch_add(1, Ordering::Relaxed);
                }
            }
        })
    }
}

impl Default for LocalCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LocalCache {
    fn drop(&mut self) {
        // Abort cleanup task when LocalCache is dropped
        if let Some(handle) = self.cleanup_handle.take() {
            handle.abort();
        }
    }
}

#[async_trait]
impl Cache for LocalCache {
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de> + Send,
    {
        if let Some(entry) = self.store.get(key) {
            // Check if expired
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.store.remove(key);
                self.evictions.fetch_add(1, Ordering::Relaxed);
                self.misses.fetch_add(1, Ordering::Relaxed);
                return Ok(None);
            }

            // Deserialize and return
            let value: V = bincode::deserialize(&entry.data)
                .context("Failed to deserialize cached value")?;

            self.hits.fetch_add(1, Ordering::Relaxed);
            Ok(Some(value))
        } else {
            self.misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    async fn set<V>(&self, key: &str, value: &V, ttl: Duration) -> Result<()>
    where
        V: Serialize + Send + Sync,
    {
        // Serialize value
        let data = bincode::serialize(value)
            .context("Failed to serialize value")?;

        // Create cache entry
        let entry = CacheEntry::new(data, ttl);

        // Insert into store
        self.store.insert(key.to_string(), entry);

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        self.store.remove(key);
        Ok(())
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        if let Some(entry) = self.store.get(key) {
            if entry.is_expired() {
                // Remove expired entry
                drop(entry);
                self.store.remove(key);
                self.evictions.fetch_add(1, Ordering::Relaxed);
                Ok(false)
            } else {
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }

    async fn clear(&self) -> Result<()> {
        self.store.clear();
        Ok(())
    }

    async fn stats(&self) -> Result<CacheStats> {
        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let evictions = self.evictions.load(Ordering::Relaxed);
        let size = self.store.len();

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

    #[tokio::test]
    async fn test_local_cache_new() {
        let cache = LocalCache::new();
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    async fn test_local_cache_set_and_get() {
        let cache = LocalCache::new();

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
    async fn test_local_cache_get_nonexistent() {
        let cache = LocalCache::new();

        let value: Option<String> = cache.get("nonexistent").await.unwrap();
        assert_eq!(value, None);

        // Check stats
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.misses, 1);
    }

    #[tokio::test]
    async fn test_local_cache_delete() {
        let cache = LocalCache::new();

        // Set and delete
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.delete("key1").await.unwrap();

        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_local_cache_exists() {
        let cache = LocalCache::new();

        // Check non-existent key
        assert_eq!(cache.exists("key1").await.unwrap(), false);

        // Set a value
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();

        // Check existing key
        assert_eq!(cache.exists("key1").await.unwrap(), true);
    }

    #[tokio::test]
    async fn test_local_cache_clear() {
        let cache = LocalCache::new();

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
    async fn test_local_cache_ttl_expiration() {
        let cache = LocalCache::new();

        // Set a value with short TTL
        cache.set("key1", &"value1", Duration::from_millis(100)).await.unwrap();

        // Should exist immediately
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired and return None
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, None);

        // Check that eviction was counted
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.evictions, 1);
    }

    #[tokio::test]
    async fn test_local_cache_concurrent_access() {
        let cache = Arc::new(LocalCache::new());

        // Spawn multiple tasks that access the cache concurrently
        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let key = format!("key{}", i);
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
    async fn test_local_cache_hit_rate() {
        let cache = LocalCache::new();

        // Set some values
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("key2", &"value2", Duration::from_secs(60)).await.unwrap();

        // 2 hits
        let _: Option<String> = cache.get("key1").await.unwrap();
        let _: Option<String> = cache.get("key2").await.unwrap();

        // 1 miss
        let _: Option<String> = cache.get("key3").await.unwrap();

        // Check hit rate (2/3 = 0.666...)
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 1);
        assert!((stats.hit_rate - 0.6666).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_local_cache_with_shard_count() {
        let cache = LocalCache::with_shard_count(16);

        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        let value: Option<String> = cache.get("key1").await.unwrap();

        assert_eq!(value, Some("value1".to_string()));
    }
}
