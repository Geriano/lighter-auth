use anyhow::Context;
use lighter_common::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::prelude::*;

use crate::entities::v1::users::Model;
use crate::entities::v1::{permissions, roles};
use crate::requests::v1::user::UserUpdateGeneralInformationRequest;

#[::tracing::instrument(skip(db, request), fields(user_id = %id, email = %request.email, username = %request.username))]
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: UserUpdateGeneralInformationRequest,
) -> anyhow::Result<Success> {
    let mut validation = Validation::new();
    let name = request.name.trim().to_lowercase();
    let email = request.email.trim().to_lowercase();
    let username = request.username.trim().to_lowercase();
    let profile_photo_id = request.profile_photo_id.map(|id| id.trim().to_string());
    let permissions = permissions::Entity::find()
        .filter(permissions::Column::Id.is_in(request.permissions.clone()))
        .all(db)
        .await
        .context("Failed to query permissions from database")?;
    let roles = roles::Entity::find()
        .filter(roles::Column::Id.is_in(request.roles.clone()))
        .all(db)
        .await
        .context("Failed to query roles from database")?;

    if name.is_empty() {
        validation.add("name", "Name is required.");
    }

    if email.is_empty() {
        validation.add("email", "Email is required.");
    }

    if username.is_empty() {
        validation.add("username", "Username is required.");
    }

    if !request.permissions.is_empty() {
        for permission_id in &request.permissions {
            if !permissions
                .iter()
                .any(|permission| permission.id == *permission_id)
            {
                validation.add(
                    "permissions",
                    format!("Permission {} does not exist.", permission_id),
                );
            }
        }
    }

    if !request.roles.is_empty() {
        for role_id in &request.roles {
            if !roles.iter().any(|role| role.id == *role_id) {
                validation.add("roles", format!("Role {} does not exist.", role_id));
            }
        }
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let user = Model::find_by_id(db, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    user.update_general_information(
        db,
        name,
        email,
        user.email_verified_at,
        username,
        profile_photo_id,
        permissions,
        roles,
    )
    .await
    .context("Failed to update user general information in database")?;

    Ok(Success)
}
