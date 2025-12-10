# Cache Implementation

lighter-auth provides three cache implementations: LocalCache (in-memory), RedisCache (distributed), and HybridCache (L1 + L2 tiered).

## LocalCache (Default)

In-memory cache using DashMap for high-performance concurrent access.

**Features:**
- Lock-free concurrent access
- Automatic expiration via background cleanup task
- Configurable shard count for better concurrency
- Zero external dependencies
- Metrics tracking (hits, misses, evictions)

**Usage:**
```rust
use lighter_auth::cache::{Cache, LocalCache};
use std::time::Duration;

let cache = LocalCache::new();

// Or with custom shard count
let cache = LocalCache::with_shard_count(16);

// Store value
cache.set("key", &value, Duration::from_secs(300)).await?;

// Retrieve value
let value: Option<String> = cache.get("key").await?;

// Check existence
let exists = cache.exists("key").await?;

// Delete
cache.delete("key").await?;

// Clear all
cache.clear().await?;

// Get stats
let stats = cache.stats().await?;
println!("Hit rate: {:.2}%", stats.hit_rate * 100.0);
```

**Limitations:**
- Not distributed (single-process only)
- Lost on application restart
- Memory consumption grows with data

## RedisCache (Optional)

Distributed cache using Redis with async connection pooling.

**Features:**
- Distributed across multiple instances
- Persistent storage (survives restarts)
- Automatic reconnection via ConnectionManager
- Key prefixing for namespace isolation
- Bincode serialization for binary efficiency
- TTL support via Redis SET EX
- Metrics tracking

**Dependencies:**

Redis is now always available as a required dependency (no feature flag needed):

```toml
[dependencies]
redis = { version = "0.27", features = ["aio", "tokio-comp", "connection-manager"] }
```

**Build and Test:**
```bash
cargo build
cargo test --features sqlite
```

**Usage:**
```rust
use lighter_auth::cache::{Cache, RedisCache};
use std::time::Duration;

// Connect to Redis
let cache = RedisCache::new("redis://localhost:6379", "lighter-auth").await?;

// Or from existing client
let client = redis::Client::open("redis://localhost:6379")?;
let cache = RedisCache::from_client(client, "lighter-auth").await?;

// Same API as LocalCache
cache.set("key", &value, Duration::from_secs(300)).await?;
let value: Option<String> = cache.get("key").await?;
```

**Key Prefixing:**

All keys are automatically prefixed with the application name:

```rust
let cache = RedisCache::new("redis://localhost:6379", "lighter-auth").await?;

// Internally stored as "lighter-auth:user:123"
cache.set("user:123", &user, ttl).await?;
```

This allows multiple applications to share the same Redis instance without key collisions.

**Environment Variables:**
```bash
# Redis connection string
REDIS_URL=redis://localhost:6379

# Redis with authentication
REDIS_URL=redis://:password@localhost:6379

# Redis with database selection
REDIS_URL=redis://localhost:6379/1

# Redis Cluster
REDIS_URL=redis://node1:6379,node2:6379,node3:6379
```

## Cache Trait

Both implementations conform to the same `Cache` trait:

```rust
#[async_trait]
pub trait Cache: Send + Sync + Debug {
    /// Get a value from the cache
    async fn get<V>(&self, key: &str) -> Result<Option<V>>
    where
        V: for<'de> Deserialize<'de> + Send;

    /// Set a value in the cache with a TTL
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
```

This allows you to swap implementations without changing application code.

## CacheKey Helper

Use the `CacheKey` helper for consistent key naming:

```rust
use lighter_auth::cache::CacheKey;

// Predefined patterns
let key = CacheKey::user(user_id);              // "user:{id}"
let key = CacheKey::token(token_id);            // "token:{id}"
let key = CacheKey::permission(perm_id);        // "permission:{id}"
let key = CacheKey::role(role_id);              // "role:{id}"
let key = CacheKey::session(session_id);        // "session:{id}"
let key = CacheKey::user_permissions(user_id);  // "user:{id}:permissions"
let key = CacheKey::user_roles(user_id);        // "user:{id}:roles"

// Custom patterns
let key = CacheKey::custom("api", "rate-limit");  // "api:rate-limit"
```

## Performance Comparison

### LocalCache
- **Latency**: < 1µs (memory access)
- **Throughput**: ~1M ops/sec per core
- **Concurrency**: Lock-free, scales with CPU cores
- **Memory**: ~100 bytes overhead per entry + data size
- **Distribution**: ❌ Single instance only
- **Persistence**: ❌ Lost on restart

### RedisCache
- **Latency**: ~1-5ms (network + Redis)
- **Throughput**: ~100K ops/sec (network limited)
- **Concurrency**: Excellent (Redis is single-threaded but pipelined)
- **Memory**: Redis memory + small client overhead
- **Distribution**: ✅ Shared across instances
- **Persistence**: ✅ Survives restarts

### HybridCache (Recommended)
- **Latency**: < 1µs (L1 hit) / ~1-5ms (L2 hit)
- **Throughput**: ~1M ops/sec (after L1 warmup)
- **Concurrency**: Excellent (inherits from both layers)
- **Memory**: L1 + L2 combined
- **Distribution**: ✅ Shared across instances (via L2)
- **Persistence**: ✅ Survives restarts (via L2)
- **Resilience**: ✅ Falls back to L1 if Redis fails

**Recommendation:**
- Use **LocalCache** for:
  - Single-instance deployments
  - Hot path data (authentication tokens)
  - Sub-millisecond latency requirements
  - Development and testing

- Use **RedisCache** for:
  - Multi-instance deployments (load balanced)
  - Shared state across services
  - Persistent cache (survives restarts)
  - Cache invalidation across instances
  - When L1 memory overhead is unacceptable

- Use **HybridCache** for (RECOMMENDED):
  - Production multi-instance deployments
  - Need both low latency AND distribution
  - Hot path data with occasional misses
  - Need resilience to Redis failures
  - Best overall performance and reliability

## Testing

### LocalCache Tests
```bash
# Run all LocalCache tests
cargo test --features sqlite cache::local

# Specific test
cargo test --features sqlite test_local_cache_set_and_get
```

### RedisCache Tests
```bash
# Start Redis for testing
docker run -d -p 6379:6379 redis

# Run RedisCache tests (requires Redis)
cargo test --features sqlite cache::redis -- --ignored

# Run all tests including integration
cargo test --features sqlite -- --ignored
```

### HybridCache Tests
```bash
# Run HybridCache tests without Redis (L1-only mode)
cargo test --features sqlite cache::hybrid

# Run HybridCache tests with Redis (requires Redis)
docker run -d -p 6379:6379 redis
cargo test --features sqlite cache::hybrid -- --ignored

# Run all cache tests
cargo test --features sqlite cache:: -- --ignored
```

**Note**: RedisCache and some HybridCache tests are marked `#[ignore]` by default because they require a running Redis instance. Use `--ignored` flag to run them.

## Production Deployment

### Docker Compose with Redis

```yaml
version: '3.8'

services:
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    command: redis-server --appendonly yes
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 3s
      retries: 5

  auth:
    build: .
    depends_on:
      redis:
        condition: service_healthy
    environment:
      REDIS_URL: redis://redis:6379
      DATABASE_URL: postgres://user:pass@postgres/db
    ports:
      - "8080:8080"

volumes:
  redis_data:
```

### Redis Configuration

**Production Redis settings** (`redis.conf`):

```conf
# Memory
maxmemory 256mb
maxmemory-policy allkeys-lru

# Persistence (optional)
save 900 1
save 300 10
save 60 10000

# Performance
tcp-keepalive 60
timeout 300

# Security
requirepass your-strong-password
```

### Connection Pooling

RedisCache uses `redis::aio::ConnectionManager` which provides:
- Automatic connection pooling
- Reconnection on failure
- Async/await support
- Pipeline support

No additional configuration needed.

## Metrics and Monitoring

Both cache implementations track metrics:

```rust
let stats = cache.stats().await?;

println!("Cache Statistics:");
println!("  Size:       {} entries", stats.size);
println!("  Hits:       {}", stats.hits);
println!("  Misses:     {}", stats.misses);
println!("  Evictions:  {}", stats.evictions);
println!("  Hit Rate:   {:.2}%", stats.hit_rate * 100.0);
```

**Recommended Metrics to Export:**
- `cache_hits_total` (counter)
- `cache_misses_total` (counter)
- `cache_evictions_total` (counter)
- `cache_size_entries` (gauge)
- `cache_hit_rate` (gauge)
- `cache_operation_duration_seconds` (histogram)

## Migration Guide

### From LocalCache to RedisCache

**Before** (LocalCache):
```rust
use lighter_auth::cache::LocalCache;

let cache = LocalCache::new();
```

**After** (RedisCache):
```rust
use lighter_auth::cache::RedisCache;

let redis_url = std::env::var("REDIS_URL")
    .unwrap_or_else(|_| "redis://localhost:6379".to_string());

let cache = RedisCache::new(&redis_url, "lighter-auth").await?;
```

**No other code changes required** - the Cache trait API is identical.

## HybridCache (Recommended for Production)

Two-layer cache combining LocalCache (L1) and RedisCache (L2) for optimal performance and distribution.

**Features:**
- **L1 (LocalCache)**: Fast in-memory cache (< 1µs latency)
- **L2 (RedisCache)**: Distributed persistent cache (1-5ms latency)
- **Automatic L1 backfill**: L2 hits populate L1 for future fast access
- **Graceful degradation**: Falls back to L1-only if Redis unavailable
- **Write-through**: Writes to both layers simultaneously
- **Comprehensive metrics**: Aggregated stats from both layers

**Architecture:**
```
Request → Check L1 (fast) → Check L2 (distributed) → Return
          ↓ Hit              ↓ Hit (backfill L1)     ↓ Miss
          Return             Return                   None
```

**Usage:**

```rust
use lighter_auth::cache::{Cache, LocalCache, RedisCache, HybridCache};
use std::time::Duration;

// Create L1 cache (always present)
let l1 = LocalCache::new();

// Create L2 cache (optional)
let l2 = RedisCache::new("redis://localhost:6379", "lighter-auth").await.ok();

// Create hybrid cache
let cache = HybridCache::new(l1, l2);

// Same Cache trait API
cache.set("key", &value, Duration::from_secs(300)).await?;
let value: Option<String> = cache.get("key").await?;
```

**L1-Only Mode** (no Redis):
```rust
use lighter_auth::cache::HybridCache;

// Automatically falls back to LocalCache only
let cache = HybridCache::local_only();

// Works exactly the same, but only uses L1
cache.set("key", &value, Duration::from_secs(300)).await?;
```

**Cache Strategy:**

1. **get()**: Check L1 → on miss check L2 → on L2 hit backfill L1 → return
2. **set()**: Write to L1 (fail on error) → write to L2 (log warning if fails)
3. **delete()**: Delete from L1 → delete from L2 (log warning if fails)
4. **exists()**: Check L1 → on miss check L2
5. **clear()**: Clear L1 → clear L2 (log warning if fails)
6. **stats()**: Aggregate hits/misses/evictions from both layers

**Error Handling:**

HybridCache gracefully degrades to L1-only mode if Redis is unavailable:

```rust
// Redis connection fails - no problem!
let l1 = LocalCache::new();
let l2 = RedisCache::new("redis://unreachable:6379", "app").await.ok(); // Returns None

let cache = HybridCache::new(l1, l2); // Works with l2=None

// All operations work, just without distributed caching
cache.set("key", &value, Duration::from_secs(300)).await?; // Only L1
```

**Production Configuration:**

```rust
use lighter_auth::cache::{HybridCache, LocalCache, RedisCache};

async fn create_production_cache() -> HybridCache {
    // L1: In-memory cache with optimized shard count
    let l1 = LocalCache::with_shard_count(num_cpus::get() * 4);

    // L2: Redis with connection from environment
    let l2 = match std::env::var("REDIS_URL") {
        Ok(url) => RedisCache::new(&url, "lighter-auth").await.ok(),
        Err(_) => {
            tracing::warn!("REDIS_URL not set, using local-only cache");
            None
        }
    };

    HybridCache::new(l1, l2)
}
```

**Performance Characteristics:**

| Operation | L1 Hit | L2 Hit | Miss |
|-----------|--------|--------|------|
| get() latency | < 1µs | ~1-5ms | ~1-5ms |
| set() latency | ~1µs | ~1-5ms | ~1-5ms |
| Throughput | ~1M ops/s | ~100K ops/s | N/A |

**Benefits:**
- **Best of both worlds**: L1 speed + L2 distribution
- **Reduced Redis load**: Most reads served from L1 (after warmup)
- **High availability**: Continues working if Redis fails
- **Transparent**: Same Cache trait API as single-layer caches
- **Production-ready**: Comprehensive logging and error handling

**When to use:**
- ✅ Multi-instance deployments (load balanced)
- ✅ Need low latency AND distribution
- ✅ Hot path data with occasional misses
- ✅ Need resilience to Redis failures

**Testing:**

```bash
# Test with LocalCache only
cargo test --features sqlite cache::hybrid::test_hybrid_cache_local_only

# Test with Redis (requires running Redis)
cargo test --features "sqlite,redis-cache" cache::hybrid -- --ignored
```

## Troubleshooting

### RedisCache Connection Issues

**Error: "Failed to create Redis connection manager"**
```bash
# Check Redis is running
redis-cli ping
# Should return: PONG

# Check connection string
echo $REDIS_URL

# Test connection
redis-cli -u $REDIS_URL ping
```

**Error: "Connection refused"**
- Ensure Redis is running: `docker ps | grep redis`
- Check firewall rules
- Verify Redis is listening on correct interface: `netstat -an | grep 6379`

### Performance Issues

**High latency with RedisCache:**
- Use connection pooling (already enabled by default)
- Consider Redis pipeline for batch operations
- Check network latency: `redis-cli --latency`
- Use LocalCache for hot path data

**Memory issues with LocalCache:**
- Reduce TTL to expire entries faster
- Implement LRU eviction policy (not currently supported)
- Switch to RedisCache with maxmemory-policy

## Examples

See `examples/cache_usage.rs` for a complete working example:

```bash
# Run example with LocalCache only
cargo run --example cache_usage --features sqlite

# Run example with both caches (requires Redis)
cargo run --example cache_usage --features sqlite
```

## API Documentation

Generate and view full API documentation:

```bash
cargo doc --open
```

## Roadmap

Future enhancements:
- [ ] Redis Cluster support
- [ ] Redis Sentinel support
- [ ] Batch operations (MGET, MSET)
- [ ] TTL inspection
- [ ] Cache warming
- [ ] Distributed lock support
- [ ] Pub/Sub for cache invalidation
- [ ] Compression for large values
- [ ] Encryption at rest

## Contributing

When adding cache features:
1. Implement for both LocalCache and RedisCache
2. Add tests with and without Redis
3. Update this documentation
4. Add example usage
5. Ensure backward compatibility
