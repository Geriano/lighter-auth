use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;
use crate::metrics::AppMetrics;

#[::tracing::instrument(skip(db, metrics), fields(permission_id = %id))]
pub async fn delete(
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    id: Uuid,
) -> anyhow::Result<Success> {
    ::tracing::info!("Deleting permission");

    let permission = Model::find_by_id(db, metrics, id)
        .await
        .context("Failed to query permission from database")?
        .ok_or_else(|| anyhow::anyhow!("Permission not found"))?;

    permission
        .delete(db, metrics)
        .await
        .context("Failed to delete permission from database")?;

    ::tracing::info!("Permission deleted successfully");

    Ok(Success)
}
