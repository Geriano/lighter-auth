use std::sync::Arc;
use std::time::Duration;

use lighter_common::prelude::*;

use crate::cache::{Cache, CacheKey, HybridCache};

// Re-export the internal Auth type for use with the cache
pub use super::internal::Auth;

#[derive(Clone)]
pub struct Authenticated {
    cache: Arc<HybridCache>,
}

impl Authenticated {
    /// Create new Authenticated with a cache instance
    pub fn new(cache: Arc<HybridCache>) -> Self {
        Self { cache }
    }

    /// Get auth from cache by token ID
    #[::tracing::instrument(skip(self), fields(token_id = %token_id))]
    pub async fn get(&self, token_id: Uuid) -> anyhow::Result<Option<Auth>> {
        let key = CacheKey::token(&token_id);

        match self.cache.get::<Auth>(&key).await {
            Ok(result) => {
                if result.is_some() {
                    ::tracing::debug!("Auth cache hit");
                } else {
                    ::tracing::debug!("Auth cache miss");
                }
                Ok(result)
            }
            Err(e) => {
                ::tracing::error!(error = %e, "Failed to get auth from cache");
                Err(e)
            }
        }
    }

    /// Set auth in cache with 5-minute TTL
    #[::tracing::instrument(skip(self, auth), fields(token_id = %token_id, user_id = %auth.user.id))]
    pub async fn set(&self, token_id: Uuid, auth: &Auth) -> anyhow::Result<()> {
        let key = CacheKey::token(&token_id);

        match self.cache.set(&key, auth, Duration::from_secs(300)).await {
            Ok(_) => {
                ::tracing::debug!("Auth cached successfully");
                Ok(())
            }
            Err(e) => {
                ::tracing::error!(error = %e, "Failed to set auth in cache");
                Err(e)
            }
        }
    }

    /// Remove auth from cache (for logout)
    #[::tracing::instrument(skip(self), fields(token_id = %token_id))]
    pub async fn remove(&self, token_id: Uuid) -> anyhow::Result<()> {
        let key = CacheKey::token(&token_id);

        match self.cache.delete(&key).await {
            Ok(_) => {
                ::tracing::debug!("Auth removed from cache");
                Ok(())
            }
            Err(e) => {
                ::tracing::error!(error = %e, "Failed to remove auth from cache");
                Err(e)
            }
        }
    }

    /// Remove auth from cache after a delay
    #[::tracing::instrument(skip(self), fields(token_id = %token_id, delay_secs = ?delay.as_secs()))]
    pub async fn remove_delay(&self, token_id: Uuid, delay: Duration) -> anyhow::Result<()> {
        let cache = self.cache.clone();

        actix::spawn(async move {
            actix::clock::sleep(delay).await;
            let key = CacheKey::token(&token_id);

            if let Err(e) = cache.delete(&key).await {
                ::tracing::error!(error = %e, token_id = %token_id, "Failed to remove auth from cache after delay");
            } else {
                ::tracing::debug!(token_id = %token_id, "Auth removed from cache after delay");
            }
        });

        Ok(())
    }
}
