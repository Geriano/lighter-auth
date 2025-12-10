use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::metrics::AppMetrics;

#[::tracing::instrument(skip(db, metrics), fields(user_id = %id))]
pub async fn delete(
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    id: Uuid,
) -> anyhow::Result<Success> {
    ::tracing::info!("Soft deleting user");

    let user = Model::find_by_id(db, metrics, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    user.soft_delete(db, metrics)
        .await
        .context("Failed to soft delete user from database")?;

    ::tracing::info!("User soft deleted successfully");

    Ok(Success)
}
