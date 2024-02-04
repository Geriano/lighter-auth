use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::requests::v1::role::RoleRequest;
use crate::responses::v1::role::Role;

pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: RoleRequest,
) -> Result<Role, Error> {
    let mut validation = Validation::new();
    let name = request.name.trim().to_lowercase();

    if name.is_empty() {
        validation.add("name", "Name is required");
    }

    if !validation.is_empty() {
        return Err(validation.into());
    }

    let role = match Model::find_by_id(db, id).await? {
        Some(role) => role,
        None => return Err(NotFound::new("Role not found").into()),
    };

    role.update(db, name).await?;

    Ok(role.into())
}
