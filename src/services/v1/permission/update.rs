use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;
use crate::requests::v1::permission::PermissionRequest;
use crate::responses::v1::permission::Permission;

pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: PermissionRequest,
) -> Result<Permission, Error> {
    let mut validation = Validation::new();
    let name = request.name.trim().to_lowercase();

    if name.is_empty() {
        validation.add("name", "Name is required");
    }

    if !validation.is_empty() {
        return Err(validation.into());
    }

    let permission = match Model::find_by_id(db, id).await? {
        Some(permission) => permission,
        None => return Err(NotFound::new("Permission not found").into()),
    };

    permission.update(db, name).await?;

    Ok(permission.into())
}
