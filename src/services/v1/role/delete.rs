use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::metrics::AppMetrics;

#[::tracing::instrument(skip(db, metrics), fields(role_id = %id))]
pub async fn delete(
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    id: Uuid,
) -> anyhow::Result<Success> {
    ::tracing::info!("Deleting role");

    let role = Model::find_by_id(db, metrics, id)
        .await
        .context("Failed to query role from database")?
        .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

    role.delete(db, metrics)
        .await
        .context("Failed to delete role from database")?;

    ::tracing::info!("Role deleted successfully");

    Ok(Success)
}
