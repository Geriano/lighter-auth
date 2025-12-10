use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::metrics::AppMetrics;
use crate::responses::v1::role::Role;

#[::tracing::instrument(skip(db, metrics), fields(role_id = %id))]
pub async fn show(
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    id: Uuid,
) -> anyhow::Result<Role> {
    ::tracing::info!("Fetching role details");

    let role = Model::find_by_id(db, metrics, id)
        .await
        .context("Failed to query role from database")?
        .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

    ::tracing::info!("Role details fetched successfully");

    Ok(role.into())
}
