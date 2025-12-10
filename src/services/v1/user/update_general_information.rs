use anyhow::Context;
use lighter_common::prelude::*;
use sea_orm::ColumnTrait;
use sea_orm::prelude::*;

use crate::entities::v1::users::Model;
use crate::entities::v1::{permissions, roles};
use crate::metrics::AppMetrics;
use crate::requests::v1::user::UserUpdateGeneralInformationRequest;

#[::tracing::instrument(skip(db, metrics, request), fields(user_id = %id, email = %request.email, username = %request.username))]
pub async fn update(
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    id: Uuid,
    request: UserUpdateGeneralInformationRequest,
) -> anyhow::Result<Success> {
    ::tracing::info!("Updating user general information");

    // Validate request DTO
    if let Err(errors) = request.validate() {
        let mut validation = Validation::new();
        for error in errors {
            validation.add("validation", error);
        }
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

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

    let user = Model::find_by_id(db, metrics, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    user.update_general_information(
        db,
        metrics,
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

    ::tracing::info!("User general information updated successfully");

    Ok(Success)
}
