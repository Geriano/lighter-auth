// Comprehensive unit tests for cache implementations
// Tests LocalCache, HybridCache, and RedisCache

use lighter_auth::cache::{Cache, LocalCache, HybridCache};
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};

use lighter_auth::cache::RedisCache;

// ============================================================================
// Test Data Structures
// ============================================================================

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
struct SimpleData {
    id: u32,
    name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct ComplexData {
    id: u64,
    name: String,
    email: String,
    roles: Vec<String>,
    metadata: std::collections::HashMap<String, String>,
}

// ============================================================================
// LocalCache Tests
// ============================================================================

#[tokio::test]
async fn test_local_cache_set_and_get() {
    let cache = LocalCache::new();

    // Test simple string
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));
}

#[tokio::test]
async fn test_local_cache_set_and_get_numbers() {
    let cache = LocalCache::new();

    // Test integers
    cache.set("int_key", &42i32, Duration::from_secs(60)).await.unwrap();
    let value: Option<i32> = cache.get("int_key").await.unwrap();
    assert_eq!(value, Some(42));

    // Test floats
    cache.set("float_key", &42.5f64, Duration::from_secs(60)).await.unwrap();
    let value: Option<f64> = cache.get("float_key").await.unwrap();
    assert_eq!(value, Some(42.5));
}

#[tokio::test]
async fn test_local_cache_get_nonexistent() {
    let cache = LocalCache::new();

    let value: Option<String> = cache.get("nonexistent").await.unwrap();
    assert_eq!(value, None);

    // Verify miss was counted
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hits, 0);
}

#[tokio::test]
async fn test_local_cache_delete() {
    let cache = LocalCache::new();

    // Set, verify, delete, verify gone
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();

    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    cache.delete("key1").await.unwrap();

    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_local_cache_exists() {
    let cache = LocalCache::new();

    // Non-existent key
    assert!(!cache.exists("key1").await.unwrap());

    // Set and check
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    assert!(cache.exists("key1").await.unwrap());

    // Delete and check
    cache.delete("key1").await.unwrap();
    assert!(!cache.exists("key1").await.unwrap());
}

#[tokio::test]
async fn test_local_cache_clear() {
    let cache = LocalCache::new();

    // Set multiple values
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    cache.set("key2", &"value2", Duration::from_secs(60)).await.unwrap();
    cache.set("key3", &"value3", Duration::from_secs(60)).await.unwrap();

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 3);

    // Clear
    cache.clear().await.unwrap();

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 0);

    // Verify all values are gone
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_local_cache_ttl_expiration() {
    let cache = LocalCache::new();

    // Set with short TTL
    cache.set("key1", &"value1", Duration::from_millis(100)).await.unwrap();

    // Should exist immediately
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be expired
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, None);

    // Verify eviction was counted
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.evictions, 1);
}

#[tokio::test]
async fn test_local_cache_ttl_different_durations() {
    let cache = LocalCache::new();

    // Set multiple keys with different TTLs
    cache.set("short", &"value1", Duration::from_millis(50)).await.unwrap();
    cache.set("medium", &"value2", Duration::from_millis(150)).await.unwrap();
    cache.set("long", &"value3", Duration::from_secs(60)).await.unwrap();

    // Wait for short to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Short should be expired, others should exist
    let short: Option<String> = cache.get("short").await.unwrap();
    assert_eq!(short, None);

    let medium: Option<String> = cache.get("medium").await.unwrap();
    assert_eq!(medium, Some("value2".to_string()));

    let long: Option<String> = cache.get("long").await.unwrap();
    assert_eq!(long, Some("value3".to_string()));

    // Wait for medium to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    let medium: Option<String> = cache.get("medium").await.unwrap();
    assert_eq!(medium, None);

    let long: Option<String> = cache.get("long").await.unwrap();
    assert_eq!(long, Some("value3".to_string()));
}

#[tokio::test]
async fn test_local_cache_stats() {
    let cache = LocalCache::new();

    // Initial stats
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.evictions, 0);
    assert_eq!(stats.size, 0);
    assert_eq!(stats.hit_rate, 0.0);

    // Set some values
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    cache.set("key2", &"value2", Duration::from_secs(60)).await.unwrap();

    // Generate hits
    let _: Option<String> = cache.get("key1").await.unwrap();
    let _: Option<String> = cache.get("key2").await.unwrap();
    let _: Option<String> = cache.get("key1").await.unwrap(); // Another hit

    // Generate misses
    let _: Option<String> = cache.get("nonexistent1").await.unwrap();
    let _: Option<String> = cache.get("nonexistent2").await.unwrap();

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.hits, 3);
    assert_eq!(stats.misses, 2);
    assert_eq!(stats.size, 2);

    // Hit rate should be 3/5 = 0.6
    assert!((stats.hit_rate - 0.6).abs() < 0.001);
}

#[tokio::test]
async fn test_local_cache_concurrent_access() {
    let cache = Arc::new(LocalCache::new());
    let mut handles = vec![];

    // Spawn 20 tasks that concurrently access the cache
    for i in 0..20 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let key = format!("key{}", i);
            let value = format!("value{}", i);

            // Set value
            cache_clone.set(&key, &value, Duration::from_secs(60)).await.unwrap();

            // Get value multiple times
            for _ in 0..10 {
                let retrieved: Option<String> = cache_clone.get(&key).await.unwrap();
                assert_eq!(retrieved, Some(value.clone()));
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all values are in cache
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 20);
    assert!(stats.hits >= 200); // At least 200 hits (20 tasks * 10 gets)
}

#[tokio::test]
async fn test_local_cache_concurrent_writes() {
    let cache = Arc::new(LocalCache::new());
    let mut handles = vec![];

    // Spawn multiple tasks writing to the same key
    for i in 0..50 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let value = format!("value{}", i);
            cache_clone.set("shared_key", &value, Duration::from_secs(60)).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all writes
    for handle in handles {
        handle.await.unwrap();
    }

    // Should have one entry (last write wins)
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 1);

    // Should have some value
    let value: Option<String> = cache.get("shared_key").await.unwrap();
    assert!(value.is_some());
}

#[tokio::test]
async fn test_local_cache_serialization_complex_types() {
    let cache = LocalCache::new();

    let data = ComplexData {
        id: 123,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        roles: vec!["admin".to_string(), "user".to_string()],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("department".to_string(), "Engineering".to_string());
            map.insert("location".to_string(), "San Francisco".to_string());
            map
        },
    };

    // Set complex type
    cache.set("complex_key", &data, Duration::from_secs(60)).await.unwrap();

    // Get and verify
    let retrieved: Option<ComplexData> = cache.get("complex_key").await.unwrap();
    assert_eq!(retrieved, Some(data));
}

#[tokio::test]
async fn test_local_cache_serialization_vectors() {
    let cache = LocalCache::new();

    let data = vec![1, 2, 3, 4, 5];
    cache.set("vec_key", &data, Duration::from_secs(60)).await.unwrap();

    let retrieved: Option<Vec<i32>> = cache.get("vec_key").await.unwrap();
    assert_eq!(retrieved, Some(data));
}

#[tokio::test]
async fn test_local_cache_empty_key() {
    let cache = LocalCache::new();

    // Empty string key should work
    cache.set("", &"value", Duration::from_secs(60)).await.unwrap();
    let value: Option<String> = cache.get("").await.unwrap();
    assert_eq!(value, Some("value".to_string()));
}

#[tokio::test]
async fn test_local_cache_special_characters_in_key() {
    let cache = LocalCache::new();

    let keys = vec![
        "key:with:colons",
        "key/with/slashes",
        "key.with.dots",
        "key-with-dashes",
        "key_with_underscores",
        "key with spaces",
        "key@with@symbols",
    ];

    for key in keys {
        cache.set(key, &"value", Duration::from_secs(60)).await.unwrap();
        let value: Option<String> = cache.get(key).await.unwrap();
        assert_eq!(value, Some("value".to_string()), "Failed for key: {}", key);
    }
}

#[tokio::test]
async fn test_local_cache_cleanup_task() {
    let cache = LocalCache::new();

    // Set values with short TTL
    for i in 0..10 {
        let key = format!("cleanup_key{}", i);
        cache.set(&key, &"value", Duration::from_millis(50)).await.unwrap();
    }

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 10);

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Trigger eviction by trying to access one
    let _: Option<String> = cache.get("cleanup_key0").await.unwrap();

    // Check eviction counter increased
    let stats = cache.stats().await.unwrap();
    assert!(stats.evictions >= 1);
}

#[tokio::test]
async fn test_local_cache_with_shard_count() {
    let cache = LocalCache::with_shard_count(16);

    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Test concurrent access with custom shard count
    let cache = Arc::new(LocalCache::with_shard_count(4));
    let mut handles = vec![];

    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = tokio::spawn(async move {
            let key = format!("key{}", i);
            cache_clone.set(&key, &i, Duration::from_secs(60)).await.unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 10);
}

#[tokio::test]
async fn test_local_cache_overwrite_value() {
    let cache = LocalCache::new();

    // Set initial value
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Overwrite with new value
    cache.set("key1", &"value2", Duration::from_secs(60)).await.unwrap();
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value2".to_string()));

    // Size should still be 1
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 1);
}

// ============================================================================
// HybridCache Tests
// ============================================================================

#[tokio::test]
async fn test_hybrid_cache_local_only_new() {
    let cache = HybridCache::local_only();
    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 0);
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);
}

#[tokio::test]
async fn test_hybrid_cache_l1_hit() {
    let cache = HybridCache::local_only();

    // Set value
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();

    // Get value (should hit L1)
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.size, 1);
}

#[tokio::test]
async fn test_hybrid_cache_both_miss() {
    let cache = HybridCache::local_only();

    // Get non-existent key
    let value: Option<String> = cache.get("nonexistent").await.unwrap();
    assert_eq!(value, None);

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1);
}

#[tokio::test]
async fn test_hybrid_cache_delete() {
    let cache = HybridCache::local_only();

    // Set and verify
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Delete
    cache.delete("key1").await.unwrap();

    // Verify deleted
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_hybrid_cache_clear() {
    let cache = HybridCache::local_only();

    // Set multiple values
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    cache.set("key2", &"value2", Duration::from_secs(60)).await.unwrap();
    cache.set("key3", &"value3", Duration::from_secs(60)).await.unwrap();

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 3);

    // Clear
    cache.clear().await.unwrap();

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 0);
}

#[tokio::test]
async fn test_hybrid_cache_exists() {
    let cache = HybridCache::local_only();

    // Non-existent key
    assert!(!cache.exists("key1").await.unwrap());

    // Set and check
    cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
    assert!(cache.exists("key1").await.unwrap());

    // Delete and check
    cache.delete("key1").await.unwrap();
    assert!(!cache.exists("key1").await.unwrap());
}

#[tokio::test]
async fn test_hybrid_cache_ttl_expiration() {
    let cache = HybridCache::local_only();

    // Set with short TTL
    cache.set("key1", &"value1", Duration::from_millis(100)).await.unwrap();

    // Should exist immediately
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, Some("value1".to_string()));

    // Wait for expiration
    tokio::time::sleep(Duration::from_millis(150)).await;

    // Should be expired
    let value: Option<String> = cache.get("key1").await.unwrap();
    assert_eq!(value, None);
}

#[tokio::test]
async fn test_hybrid_cache_concurrent_access() {
    let cache = Arc::new(HybridCache::local_only());
    let mut handles = vec![];

    // Spawn 20 tasks
    for i in 0..20 {
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

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    let stats = cache.stats().await.unwrap();
    assert_eq!(stats.size, 20);
}

#[tokio::test]
async fn test_hybrid_cache_complex_types() {
    let cache = HybridCache::local_only();

    let data = ComplexData {
        id: 456,
        name: "Jane Smith".to_string(),
        email: "jane@example.com".to_string(),
        roles: vec!["user".to_string(), "moderator".to_string()],
        metadata: {
            let mut map = std::collections::HashMap::new();
            map.insert("team".to_string(), "Product".to_string());
            map
        },
    };

    cache.set("complex_key", &data, Duration::from_secs(60)).await.unwrap();

    let retrieved: Option<ComplexData> = cache.get("complex_key").await.unwrap();
    assert_eq!(retrieved, Some(data));
}

#[tokio::test]
async fn test_hybrid_cache_debug_trait() {
    let cache = HybridCache::local_only();
    let debug_str = format!("{:?}", cache);

    assert!(debug_str.contains("HybridCache"));
    assert!(debug_str.contains("l1"));
    assert!(debug_str.contains("l2_enabled"));
}

// ============================================================================
// RedisCache Tests (requires running Redis)
// ============================================================================

mod redis_tests {
    use super::*;

    // Helper to create test Redis instance
    async fn test_redis() -> Option<RedisCache> {
        let url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        RedisCache::new(&url, "test-cache-unit").await.ok()
    }

    #[tokio::test]
    #[ignore] // Run with --ignored when Redis is available
    async fn test_redis_cache_set_and_get() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => {
                println!("Skipping test: Redis not available");
                return;
            }
        };

        // Clear first
        cache.clear().await.unwrap();

        // Set and get
        cache.set("key1", &"value1", Duration::from_secs(60)).await.unwrap();
        let value: Option<String> = cache.get("key1").await.unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_get_nonexistent() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        let value: Option<String> = cache.get("nonexistent_redis_key").await.unwrap();
        assert_eq!(value, None);

        let stats = cache.stats().await.unwrap();
        assert!(stats.misses >= 1);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_delete() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        // Set and delete
        cache.set("delete_key", &"value", Duration::from_secs(60)).await.unwrap();
        cache.delete("delete_key").await.unwrap();

        let value: Option<String> = cache.get("delete_key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_exists() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        // Non-existent
        assert!(!cache.exists("exists_test_key").await.unwrap());

        // Set and check
        cache.set("exists_test_key", &"value", Duration::from_secs(60)).await.unwrap();
        assert!(cache.exists("exists_test_key").await.unwrap());
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_clear() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        // Set multiple
        cache.set("clear1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("clear2", &"value2", Duration::from_secs(60)).await.unwrap();

        // Clear
        cache.clear().await.unwrap();

        // Verify cleared
        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_ttl_expiration() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        // Set with short TTL
        cache.set("ttl_test", &"value", Duration::from_millis(100)).await.unwrap();

        // Should exist
        let value: Option<String> = cache.get("ttl_test").await.unwrap();
        assert_eq!(value, Some("value".to_string()));

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Should be expired
        let value: Option<String> = cache.get("ttl_test").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_concurrent_access() {
        let cache = match test_redis().await {
            Some(c) => Arc::new(c),
            None => return,
        };

        cache.clear().await.unwrap();

        let mut handles = vec![];

        for i in 0..10 {
            let cache_clone = Arc::clone(&cache);
            let handle = tokio::spawn(async move {
                let key = format!("concurrent{}", i);
                let value = format!("value{}", i);
                cache_clone.set(&key, &value, Duration::from_secs(60)).await.unwrap();

                let retrieved: Option<String> = cache_clone.get(&key).await.unwrap();
                assert_eq!(retrieved, Some(value));
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        let stats = cache.stats().await.unwrap();
        assert_eq!(stats.size, 10);
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_complex_types() {
        let cache = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        let data = ComplexData {
            id: 789,
            name: "Redis User".to_string(),
            email: "redis@example.com".to_string(),
            roles: vec!["admin".to_string()],
            metadata: std::collections::HashMap::new(),
        };

        cache.set("complex", &data, Duration::from_secs(60)).await.unwrap();

        let retrieved: Option<ComplexData> = cache.get("complex").await.unwrap();
        assert_eq!(retrieved, Some(data));
    }

    #[tokio::test]
    #[ignore]
    async fn test_redis_cache_key_prefix_isolation() {
        let cache1 = RedisCache::new("redis://localhost:6379", "app1")
            .await
            .unwrap();
        let cache2 = RedisCache::new("redis://localhost:6379", "app2")
            .await
            .unwrap();

        // Set same key in different namespaces
        cache1.set("shared", &"value1", Duration::from_secs(60)).await.unwrap();
        cache2.set("shared", &"value2", Duration::from_secs(60)).await.unwrap();

        // Verify isolation
        let value1: Option<String> = cache1.get("shared").await.unwrap();
        let value2: Option<String> = cache2.get("shared").await.unwrap();

        assert_eq!(value1, Some("value1".to_string()));
        assert_eq!(value2, Some("value2".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_hybrid_cache_with_redis_l1_miss_l2_hit() {
        let l2 = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        l2.clear().await.unwrap();

        // Create another L2 instance for direct access
        let l2_direct = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2));

        // Set value directly in L2
        l2_direct.set("l2_only", &"from_l2", Duration::from_secs(60)).await.unwrap();

        // Get via hybrid (should miss L1, hit L2, backfill L1)
        let value: Option<String> = cache.get("l2_only").await.unwrap();
        assert_eq!(value, Some("from_l2".to_string()));

        // Get again (should hit L1 now)
        let value: Option<String> = cache.get("l2_only").await.unwrap();
        assert_eq!(value, Some("from_l2".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_hybrid_cache_with_redis_write_both() {
        let l2 = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        l2.clear().await.unwrap();

        // Create another L2 instance for verification
        let l2_verify = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2));

        // Write via hybrid
        cache.set("write_both", &"value", Duration::from_secs(60)).await.unwrap();

        // Verify in L2 directly
        let value: Option<String> = l2_verify.get("write_both").await.unwrap();
        assert_eq!(value, Some("value".to_string()));
    }

    #[tokio::test]
    #[ignore]
    async fn test_hybrid_cache_with_redis_delete_both() {
        let l2 = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        l2.clear().await.unwrap();

        // Create another L2 instance for verification
        let l2_verify = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2));

        // Set via hybrid
        cache.set("delete_both", &"value", Duration::from_secs(60)).await.unwrap();

        // Delete via hybrid
        cache.delete("delete_both").await.unwrap();

        // Verify deleted from L2
        let value: Option<String> = l2_verify.get("delete_both").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    #[ignore]
    async fn test_hybrid_cache_with_redis_clear_both() {
        let l2 = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        l2.clear().await.unwrap();

        // Create another L2 instance for verification
        let l2_verify = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2));

        // Set multiple values
        cache.set("clear1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("clear2", &"value2", Duration::from_secs(60)).await.unwrap();

        // Clear via hybrid
        cache.clear().await.unwrap();

        // Verify L2 cleared
        let stats = l2_verify.stats().await.unwrap();
        assert_eq!(stats.size, 0);
    }

    #[tokio::test]
    #[ignore]
    async fn test_hybrid_cache_with_redis_stats_aggregation() {
        let l2 = match test_redis().await {
            Some(c) => c,
            None => return,
        };

        l2.clear().await.unwrap();

        let l1 = LocalCache::new();
        let cache = HybridCache::new(l1, Some(l2));

        // Set and access values
        cache.set("stats1", &"value1", Duration::from_secs(60)).await.unwrap();
        cache.set("stats2", &"value2", Duration::from_secs(60)).await.unwrap();

        // Generate hits and misses
        let _: Option<String> = cache.get("stats1").await.unwrap();
        let _: Option<String> = cache.get("stats2").await.unwrap();
        let _: Option<String> = cache.get("nonexistent").await.unwrap();

        // Get aggregated stats
        let stats = cache.stats().await.unwrap();

        assert!(stats.size >= 2);
        assert!(stats.hits >= 2);
        assert!(stats.misses >= 1);
    }
}
