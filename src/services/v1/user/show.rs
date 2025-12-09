use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;

#[::tracing::instrument(skip(db), fields(user_id = %id))]
pub async fn show(
    db: &DatabaseConnection,
    id: Uuid,
) -> anyhow::Result<Json<UserWithPermissionAndRole>> {
    let user = Model::find_by_id(db, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    let permissions = user
        .permissions(db)
        .await
        .context("Failed to query user permissions from database")?;

    let roles = user
        .roles(db)
        .await
        .context("Failed to query user roles from database")?;

    Ok(Json((user, permissions, roles).into()))
}
