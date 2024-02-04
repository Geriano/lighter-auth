use lighter_common::prelude::*;

use crate::entities::v1::permissions::Model;
use crate::responses::v1::permission::Permission;

pub async fn show(db: &DatabaseConnection, id: Uuid) -> Result<Permission, Error> {
    match Model::find_by_id(db, id).await? {
        Some(permission) => Ok(permission.into()),
        None => Err(NotFound::new("Permission not found").into()),
    }
}
