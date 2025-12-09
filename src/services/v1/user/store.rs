use actix_web::web::Json;
use lighter_common::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

use crate::entities::v1::users::Model;
use crate::entities::v1::{permissions, roles};
use crate::requests::v1::user::UserStoreRequest;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;

pub async fn store(
    db: &DatabaseConnection,
    request: UserStoreRequest,
) -> Result<Json<UserWithPermissionAndRole>, Error> {
    let mut validation = Validation::new();
    let name = request.name.trim().to_lowercase();
    let email = request.email.trim().to_lowercase();
    let username = request.username.trim().to_lowercase();
    let password = request.password;
    let password_confirmation = request.password_confirmation;
    let profile_photo_id = request.profile_photo_id.map(|id| id.trim().to_string());
    let permissions = permissions::Entity::find()
        .filter(permissions::Column::Id.is_in(request.permissions.clone()))
        .all(db)
        .await?;
    let roles = roles::Entity::find()
        .filter(roles::Column::Id.is_in(request.roles.clone()))
        .all(db)
        .await?;

    if name.is_empty() {
        validation.add("name", "Name is required.");
    }

    if email.is_empty() {
        validation.add("email", "Email is required.");
    } else if Model::email_exists(db, &email).await {
        validation.add("email", "Email already exists.");
    }

    if username.is_empty() {
        validation.add("username", "Username is required.");
    } else if Model::username_exists(db, &username).await {
        validation.add("username", "Username already exists.");
    }

    if password.is_empty() {
        validation.add("password", "Password is required.");
    } else if password.len() < 8 {
        validation.add("password", "Password must be at least 8 characters.");
    }

    if password != password_confirmation {
        validation.add(
            "password_confirmation",
            "Password confirmation does not match.",
        );
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
        return Err(validation.into());
    }

    let id = Uuid::new_v4();
    let hash = Hash::make(id, &password);
    let password = hash.to_string();
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

    model.store(db, permissions.clone(), roles.clone()).await?;

    Ok(Json((model, permissions, roles).into()))
}
