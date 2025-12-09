use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;
use crate::responses::v1::permission::Permission;

#[::tracing::instrument(skip(db), fields(permission_id = %id))]
pub async fn show(db: &DatabaseConnection, id: Uuid) -> anyhow::Result<Permission> {
    ::tracing::info!("Fetching permission details");

    let permission = Model::find_by_id(db, id)
        .await
        .context("Failed to query permission from database")?
        .ok_or_else(|| anyhow::anyhow!("Permission not found"))?;

    ::tracing::info!("Permission details fetched successfully");

    Ok(permission.into())
}
