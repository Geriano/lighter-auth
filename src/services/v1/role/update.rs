use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::requests::v1::role::RoleRequest;
use crate::responses::v1::role::Role;

#[::tracing::instrument(skip(db, request), fields(role_id = %id, name = %request.name))]
pub async fn update(
    db: &DatabaseConnection,
    id: Uuid,
    request: RoleRequest,
) -> anyhow::Result<Role> {
    let mut validation = Validation::new();
    let name = request.name.trim().to_lowercase();

    if name.is_empty() {
        validation.add("name", "Name is required");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let role = Model::find_by_id(db, id)
        .await
        .context("Failed to query role from database")?
        .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

    role.update(db, name)
        .await
        .context("Failed to update role in database")?;

    Ok(role.into())
}
