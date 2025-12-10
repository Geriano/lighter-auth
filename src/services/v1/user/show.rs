use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::metrics::AppMetrics;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;

#[::tracing::instrument(skip(db, metrics), fields(user_id = %id))]
pub async fn show(
    db: &DatabaseConnection,
    metrics: &AppMetrics,
    id: Uuid,
) -> anyhow::Result<Json<UserWithPermissionAndRole>> {
    ::tracing::info!("Fetching user details");

    let user = Model::find_by_id(db, Some(metrics), id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let permissions = user
        .permissions(db, Some(metrics))
        .await
        .context("Failed to query user permissions from database")?;

    let roles = user
        .roles(db, Some(metrics))
        .await
        .context("Failed to query user roles from database")?;

    ::tracing::info!("User details fetched successfully");

    Ok(Json((user, permissions, roles).into()))
}
