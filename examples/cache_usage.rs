/// Example demonstrating cache usage with both LocalCache and RedisCache
///
/// Run with:
/// ```bash
/// # LocalCache (no dependencies needed)
/// cargo run --example cache_usage --features sqlite
///
/// # RedisCache (requires Redis running on localhost:6379)
/// cargo run --example cache_usage --features "sqlite,redis-cache"
/// ```

use lighter_auth::cache::{Cache, CacheKey, LocalCache};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[cfg(feature = "redis-cache")]
use lighter_auth::cache::RedisCache;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct User {
    id: String,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Cache Usage Examples ===\n");

    // Example 1: LocalCache (in-memory)
    println!("1. LocalCache Example:");
    let local_cache = LocalCache::new();
    demo_cache_operations(&local_cache, "LocalCache").await?;

    // Example 2: RedisCache (distributed)
    #[cfg(feature = "redis-cache")]
    {
        println!("\n2. RedisCache Example:");
        match RedisCache::new("redis://localhost:6379", "lighter-auth").await {
            Ok(redis_cache) => {
                demo_cache_operations(&redis_cache, "RedisCache").await?;
            }
            Err(e) => {
                println!("   Failed to connect to Redis: {}", e);
                println!("   Make sure Redis is running on localhost:6379");
                println!("   Or run with: docker run -d -p 6379:6379 redis");
            }
        }
    }

    #[cfg(not(feature = "redis-cache"))]
    {
        println!("\n2. RedisCache Example: (disabled)");
        println!("   Run with --features redis-cache to enable RedisCache");
    }

    Ok(())
}

async fn demo_cache_operations<C: Cache>(cache: &C, cache_name: &str) -> anyhow::Result<()> {
    println!("   Using: {}", cache_name);

    // Create a user
    let user = User {
        id: "123e4567-e89b-12d3-a456-426614174000".to_string(),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    };

    // Use CacheKey helper for consistent key naming
    let user_key = CacheKey::user(&user.id);

    // Set value with 60 second TTL
    cache.set(&user_key, &user, Duration::from_secs(60)).await?;
    println!("   ✓ Set user: {}", user.name);

    // Get value
    let cached_user: Option<User> = cache.get(&user_key).await?;
    match cached_user {
        Some(u) => println!("   ✓ Retrieved user: {} ({})", u.name, u.email),
        None => println!("   ✗ User not found in cache"),
    }

    // Check if key exists
    let exists = cache.exists(&user_key).await?;
    println!("   ✓ Key exists: {}", exists);

    // Get cache statistics
    let stats = cache.stats().await?;
    println!("   ✓ Cache stats:");
    println!("      - Size: {} entries", stats.size);
    println!("      - Hits: {}", stats.hits);
    println!("      - Misses: {}", stats.misses);
    println!("      - Hit rate: {:.2}%", stats.hit_rate * 100.0);

    // Test cache miss
    let missing_key = CacheKey::user("nonexistent-id");
    let missing: Option<User> = cache.get(&missing_key).await?;
    println!("   ✓ Cache miss handled: {}", missing.is_none());

    // Delete specific key
    cache.delete(&user_key).await?;
    println!("   ✓ Deleted user from cache");

    // Verify deletion
    let deleted_user: Option<User> = cache.get(&user_key).await?;
    println!("   ✓ User after delete: {}", if deleted_user.is_none() { "Not found (correct)" } else { "Found (error!)" });

    // Demonstrate CacheKey helpers
    println!("\n   Cache Key Helpers:");
    println!("      - user:       {}", CacheKey::user("123"));
    println!("      - token:      {}", CacheKey::token("abc"));
    println!("      - permission: {}", CacheKey::permission("read"));
    println!("      - role:       {}", CacheKey::role("admin"));
    println!("      - session:    {}", CacheKey::session("sess-123"));
    println!("      - custom:     {}", CacheKey::custom("api", "rate-limit"));

    Ok(())
}
