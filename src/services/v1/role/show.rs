use lighter_common::prelude::*;

use crate::entities::v1::roles::Model;
use crate::responses::v1::role::Role;

pub async fn show(db: &DatabaseConnection, id: Uuid) -> Result<Role, Error> {
    match Model::find_by_id(db, id).await? {
        Some(role) => Ok(role.into()),
        None => Err(NotFound::new("Role not found").into()),
    }
}
