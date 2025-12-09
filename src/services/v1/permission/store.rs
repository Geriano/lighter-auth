use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;
use crate::requests::v1::permission::PermissionRequest;
use crate::responses::v1::permission::Permission;

#[::tracing::instrument(skip(db, request), fields(name = %request.name))]
pub async fn store(
    db: &DatabaseConnection,
    request: PermissionRequest,
) -> anyhow::Result<Permission> {
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

    // Check if permission code already exists
    if Model::code_exist(db, &code).await {
        validation.add("name", "Name already exist");
    }

    if !validation.is_empty() {
        return Err(anyhow::anyhow!("Validation failed: {:?}", validation));
    }

    let permission = Model {
        id: Uuid::new_v4(),
        code,
        name,
    };

    permission
        .store(db)
        .await
        .context("Failed to store permission to database")?;

    Ok(permission.into())
}
