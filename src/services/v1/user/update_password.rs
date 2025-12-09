use anyhow::Context;
use lighter_common::prelude::*;

use crate::config::auth::AuthConfig;
use crate::entities::v1::users::Model;
use crate::requests::v1::user::UserUpdatePasswordRequest;
use crate::security::PasswordHasher;

#[::tracing::instrument(skip(db, request), fields(user_id = %id))]
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: UserUpdatePasswordRequest,
) -> anyhow::Result<Success> {
    ::tracing::info!("Updating user password");

    // Validate request DTO
    if let Err(errors) = request.validate() {
        let mut validation = Validation::new();
        for error in errors {
            validation.add("validation", error);
        }
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let mut validation = Validation::new();
    let current_password = request.current_password;
    let new_password = request.new_password;

    let user = Model::find_by_id(db, id)
        .await
        .ok_or_else(|| anyhow::anyhow!("User not found"))?;

    // Create default AuthConfig for password hashing
    let default_config = AuthConfig::default();

    ::tracing::debug!("Creating password hasher with Argon2id");
    let hasher = PasswordHasher::from_config(&default_config)
        .map_err(|e| anyhow::anyhow!("Failed to create password hasher: {}", e))?;

    // Verify current password
    ::tracing::debug!(user_id = %id, "Verifying current password");
    let is_valid = hasher.verify(&current_password, &user.password)
        .map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))?;

    if !is_valid {
        validation.add("current_password", "Current password is incorrect.");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    // Hash new password
    ::tracing::debug!(user_id = %id, "Hashing new password with Argon2id");
    let new_hash = hasher.hash(&new_password)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?;

    user.update_password(db, new_hash)
        .await
        .context("Failed to update user password in database")?;

    ::tracing::info!(user_id = %id, "Password updated successfully");
    Ok(Success)
}
