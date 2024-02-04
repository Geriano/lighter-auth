use std::time::Duration;

use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::middlewares::v1::auth::internal::Auth;
use crate::middlewares::v1::auth::Authenticated as Cache;
use crate::requests::v1::auth::LoginRequest;
use crate::responses::v1::auth::Authenticated;

// 1 hour
pub const LIFETIME: Duration = Duration::from_secs(60 * 60);

pub async fn login(
    db: &DatabaseConnection,
    cached: &Cache,
    request: LoginRequest,
) -> Result<Authenticated, Error> {
    let mut validation = Validation::new();
    let email_or_username = request.email_or_username.trim().to_lowercase();
    let password = request.password;

    if email_or_username.is_empty() {
        validation.add("email_or_username", "Email or username field is required");
    } else {
        if !Model::email_or_username_exists(db, &email_or_username).await {
            validation.add("email_or_username", "Email or username not found");
        }
    }

    if password.is_empty() {
        validation.add("password", "Password field is required");
    } else {
        if password.len() < 8 {
            validation.add("password", "Password must be at least 8 characters");
        }
    }

    if !validation.is_empty() {
        return Err(validation.into());
    }

    let user = Model::find_by_email_or_username(db, &email_or_username)
        .await
        .unwrap();

    if !Hash::from(&user.password).verify(user.id, &password) {
        validation.add("password", "Password is incorrect");
    }

    if !validation.is_empty() {
        return Err(validation.into());
    }

    let token = user.generate_token(db, None).await?;
    let permissions = user.permissions(db).await?;
    let roles = user.roles(db).await?;
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
