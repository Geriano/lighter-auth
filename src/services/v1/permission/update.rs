use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;
use crate::requests::v1::permission::PermissionRequest;
use crate::responses::v1::permission::Permission;

#[::tracing::instrument(skip(db, request), fields(permission_id = %id, name = %request.name))]
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: PermissionRequest,
) -> anyhow::Result<Permission> {
    ::tracing::info!("Updating permission");

    // Validate request DTO
    if let Err(errors) = request.validate() {
        let mut validation = Validation::new();
        for error in errors {
            validation.add("validation", error);
        }
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let name = request.name.trim().to_lowercase();

    let permission = Model::find_by_id(db, id)
        .await
        .context("Failed to query permission from database")?
        .ok_or_else(|| anyhow::anyhow!("Permission not found"))?;

    permission
        .update(db, name)
        .await
        .context("Failed to update permission in database")?;

    ::tracing::info!("Permission updated successfully");

    Ok(permission.into())
}
