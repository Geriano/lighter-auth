use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;

#[::tracing::instrument(skip(db), fields(role_id = %id))]
pub async fn delete(db: &DatabaseConnection, id: Uuid) -> anyhow::Result<Success> {
    let role = Model::find_by_id(db, id)
        .await
        .context("Failed to query role from database")?
        .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

    role.delete(db)
        .await
        .context("Failed to delete role from database")?;

    Ok(Success)
}
