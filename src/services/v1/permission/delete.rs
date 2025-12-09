use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;

#[::tracing::instrument(skip(db), fields(permission_id = %id))]
pub async fn delete(db: &DatabaseConnection, id: Uuid) -> anyhow::Result<Success> {
    let permission = Model::find_by_id(db, id)
        .await
        .context("Failed to query permission from database")?
        .ok_or_else(|| anyhow::anyhow!("Permission not found"))?;

    permission
        .delete(db)
        .await
        .context("Failed to delete permission from database")?;

    Ok(Success)
}
