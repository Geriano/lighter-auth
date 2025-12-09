use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::users::Model;

#[::tracing::instrument(skip(db), fields(user_id = %id))]
pub async fn delete(db: &DatabaseConnection, id: Uuid) -> anyhow::Result<Success> {
    ::tracing::info!("Soft deleting user");

    let user = Model::find_by_id(db, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    user.soft_delete(db)
        .await
        .context("Failed to soft delete user from database")?;

    ::tracing::info!("User soft deleted successfully");

    Ok(Success)
}
