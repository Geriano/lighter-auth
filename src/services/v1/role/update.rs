use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::metrics::AppMetrics;
use crate::requests::v1::role::RoleRequest;
use crate::responses::v1::role::Role;

#[::tracing::instrument(skip(db, metrics, request), fields(role_id = %id, name = %request.name))]
pub async fn update(
    db: &DatabaseConnection,
    metrics: Option<&AppMetrics>,
    id: Uuid,
    request: RoleRequest,
) -> anyhow::Result<Role> {
    ::tracing::info!("Updating role");

    // Validate request DTO
    if let Err(errors) = request.validate() {
        let mut validation = Validation::new();
        for error in errors {
            validation.add("validation", error);
        }
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let name = request.name.trim().to_lowercase();

    let role = Model::find_by_id(db, metrics, id)
        .await
        .context("Failed to query role from database")?
        .ok_or_else(|| anyhow::anyhow!("Role not found"))?;

    role.update(db, metrics, name)
        .await
        .context("Failed to update role in database")?;

    ::tracing::info!("Role updated successfully");

    Ok(role.into())
}
