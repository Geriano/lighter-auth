use std::time::Duration;

use anyhow::Context;
use lighter_common::prelude::*;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;

use crate::config::auth::AuthConfig;
use crate::entities::v1::users::{ActiveModel, Model};
use crate::middlewares::v1::auth::Authenticated as Cache;
use crate::middlewares::v1::auth::internal::Auth;
use crate::requests::v1::auth::LoginRequest;
use crate::responses::v1::auth::Authenticated;
use crate::security::PasswordHasher;

// 1 hour
pub const LIFETIME: Duration = Duration::from_secs(60 * 60);

#[::tracing::instrument(skip(db, cached, request), fields(email_or_username = %request.email_or_username))]
pub async fn login(
    db: &DatabaseConnection,
    cached: &Cache,
    request: LoginRequest,
) -> anyhow::Result<Authenticated> {
    // Validate request DTO
    if let Err(errors) = request.validate() {
        let mut validation = Validation::new();
        for error in errors {
            validation.add("validation", error);
        }
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    ::tracing::info!("Processing login request");

    let mut validation = Validation::new();
    let email_or_username = request.email_or_username.trim().to_lowercase();
    let password = request.password;

    // Check if email or username exists in database
    if !Model::email_or_username_exists(db, &email_or_username).await {
        validation.add("email_or_username", "Email or username not found");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let mut user = Model::find_by_email_or_username(db, &email_or_username)
        .await
        .context("Failed to find user by email or username")?;

    // Create default AuthConfig for password hashing
    let default_config = AuthConfig::default();

    ::tracing::debug!("Creating password hasher with Argon2id");
    let hasher = PasswordHasher::from_config(&default_config)
        .map_err(|e| anyhow::anyhow!("Failed to create password hasher: {}", e))?;

    // Check if password hash is in legacy SHA-256 format
    if user.password.len() == 64 && user.password.chars().all(|c| c.is_ascii_hexdigit()) {
        ::tracing::warn!(user_id = %user.id, "User has legacy SHA-256 password hash");
        validation.add("password", "Your account uses an outdated password format. Please reset your password.");
        return Err(anyhow::anyhow!("Legacy password format detected"));
    }

    // Verify password with Argon2id
    ::tracing::debug!(user_id = %user.id, "Verifying password");
    let is_valid = hasher.verify(&password, &user.password)
        .map_err(|e| anyhow::anyhow!("Failed to verify password: {}", e))?;

    if !is_valid {
        validation.add("password", "Password is incorrect");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    // Check if password needs rehashing (e.g., Argon2 parameters changed)
    if hasher.needs_rehash(&user.password).unwrap_or(false) {
        ::tracing::info!(user_id = %user.id, "Rehashing password with updated Argon2 parameters");

        let new_hash = hasher.hash(&password)
            .map_err(|e| anyhow::anyhow!("Failed to rehash password: {}", e))?;

        // Update password hash in database
        let mut active_user: ActiveModel = user.clone().into();
        active_user.password = Set(new_hash.clone());
        active_user.update(db)
            .await
            .context("Failed to update password hash")?;

        // Update user reference with new hash
        user.password = new_hash;
    }

    let token = user
        .generate_token(db, None)
        .await
        .context("Failed to generate authentication token")?;

    let permissions = user
        .permissions(db)
        .await
        .context("Failed to fetch user permissions")?;

    let roles = user
        .roles(db)
        .await
        .context("Failed to fetch user roles")?;

    // Log before moving user
    ::tracing::info!(user_id = %user.id, token_id = %token.id, "Login successful");

    let auth = Auth {
        id: token.id,
        user: user.into(),
        permissions: permissions
            .into_iter()
            .map(|permission| permission.into())
            .collect(),
        roles: roles.into_iter().map(|role| role.into()).collect(),
    };

    cached.set(token.id, &auth).await;
    cached.remove_delay(token.id, LIFETIME).await;

    Ok(auth.into())
}
