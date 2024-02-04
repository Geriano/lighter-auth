use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;

pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<Success, Error> {
    match Model::find_by_id(db, id).await? {
        Some(role) => role.delete(db).await?,
        None => return Err(NotFound::new("Permission not found").into()),
    };

    Ok(Success)
}
