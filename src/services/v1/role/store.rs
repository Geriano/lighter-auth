use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::requests::v1::role::RoleRequest;
use crate::responses::v1::role::Role;

#[::tracing::instrument(skip(db, request), fields(name = %request.name))]
pub async fn store(db: &DatabaseConnection, request: RoleRequest) -> anyhow::Result<Role> {
    // Validate request DTO
    if let Err(errors) = request.validate() {
        let mut validation = Validation::new();
        for error in errors {
            validation.add("validation", error);
        }
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let mut validation = Validation::new();
    let name = request.name.trim().to_lowercase();
    let code = name.replace(" ", "_").to_uppercase();

    // Check if role code already exists
    if Model::code_exist(db, &code).await {
        validation.add("name", "Name already exist");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let role = Model {
        id: Uuid::new_v4(),
        code,
        name,
    };

    role.store(db)
        .await
        .context("Failed to store role to database")?;

    ::tracing::info!(role_id = %role.id, "Role created successfully");

    Ok(role.into())
}
