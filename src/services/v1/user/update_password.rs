use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::requests::v1::user::UserUpdatePasswordRequest;

#[::tracing::instrument(skip(db, request), fields(user_id = %id))]
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: UserUpdatePasswordRequest,
) -> anyhow::Result<Success> {
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

    if !Hash::from(&user.password).verify(id, &current_password) {
        validation.add("current_password", "Current password is incorrect.");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    user.update_password(db, Hash::make(id, &new_password))
        .await
        .context("Failed to update user password in database")?;

    Ok(Success)
}
