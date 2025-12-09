use actix_web::web::Json;
use anyhow::Context;
use lighter_common::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::config::auth::AuthConfig;
use crate::entities::v1::users::Model;
use crate::entities::v1::{permissions, roles};
use crate::requests::v1::user::UserStoreRequest;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;
use crate::security::PasswordHasher;

#[::tracing::instrument(skip(db, request), fields(email = %request.email, username = %request.username))]
pub async fn store(
    db: &DatabaseConnection,
    request: UserStoreRequest,
) -> anyhow::Result<Json<UserWithPermissionAndRole>> {
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
    let password = request.password;
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

    // Check if email already exists
    if Model::email_exists(db, &email).await {
        validation.add("email", "Email already exists.");
    }

    // Check if username already exists
    if Model::username_exists(db, &username).await {
        validation.add("username", "Username already exists.");
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

    // Create default AuthConfig for password hashing
    let default_config = AuthConfig::default();

    ::tracing::debug!("Creating password hasher with Argon2id");
    let hasher = PasswordHasher::from_config(&default_config)
        .map_err(|e| anyhow::anyhow!("Failed to create password hasher: {}", e))?;

    let id = Uuid::new_v4();

    ::tracing::debug!(user_id = %id, "Hashing password with Argon2id");
    let password = hasher.hash(&password)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    let model = Model {
        id,
        name,
        email,
        email_verified_at: None,
        username,
        password,
        profile_photo_id,
        created_at: now(),
        updated_at: now(),
        deleted_at: None,
    };

    model
        .store(db, permissions.clone(), roles.clone())
        .await
        .context("Failed to store user to database")?;

    ::tracing::info!(user_id = %id, "User created successfully");

    Ok(Json((model, permissions, roles).into()))
}
