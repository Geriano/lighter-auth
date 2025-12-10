use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::tokens::Model;
use crate::metrics::AppMetrics;
use crate::middlewares::v1::auth::Authenticated as Cache;
use crate::middlewares::v1::auth::internal::Auth;

#[::tracing::instrument(skip(auth, db, metrics, cached), fields(user_id = %auth.user.id, token_id = %auth.id))]
pub async fn logout(
    auth: Auth,
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    cached: &Cache,
) -> anyhow::Result<Success> {
    ::tracing::info!("Processing logout request");

    Model::logout(db, metrics, auth.user.id)
        .await
        .context("Failed to logout user and delete tokens from database")?;

    // Remove from cache (log error but don't fail the logout if cache removal fails)
    if let Err(e) = cached.remove(auth.id).await {
        ::tracing::warn!(error = %e, "Failed to remove auth from cache, but logout succeeded in database");
    }

    ::tracing::info!("Logout successful");

    Ok(Success)
}
