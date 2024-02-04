use lighter_common::prelude::*;

use crate::entities::v1::users::Model;

pub async fn delete(db: &DatabaseConnection, id: Uuid) -> Result<Success, Error> {
    match Model::find_by_id(db, id).await {
        None => return Err(NotFound::new("User not found.").into()),
        Some(user) => user.soft_delete(db).await?,
    };

    Ok(Success)
}
