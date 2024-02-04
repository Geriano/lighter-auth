use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::requests::v1::user::UserUpdatePasswordRequest;

pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: UserUpdatePasswordRequest,
) -> Result<Success, Error> {
    let mut validation = Validation::new();
    let current_password = request.current_password;
    let new_password = request.new_password;
    let password_confirmation = request.password_confirmation;

    if current_password.is_empty() {
        validation.add("current_password", "Current password is required.");
    }

    if new_password.is_empty() {
        validation.add("new_password", "New password is required.");
    }

    if password_confirmation.is_empty() {
        validation.add(
            "password_confirmation",
            "Password confirmation is required.",
        );
    }

    if new_password != password_confirmation {
        validation.add(
            "password_confirmation",
            "Password confirmation does not match.",
        );
    }

    if !validation.is_empty() {
        return Err(validation.into());
    }

    let user = match Model::find_by_id(db, id).await {
        None => return Err(NotFound::new("User not found.").into()),
        Some(user) => user,
    };

    if !Hash::from(&user.password).verify(id, &current_password) {
        validation.add("current_password", "Current password is incorrect.");
    }

    if !validation.is_empty() {
        return Err(validation.into());
    }

    user.update_password(db, Hash::make(id, &new_password))
        .await?;

    Ok(Success)
}
