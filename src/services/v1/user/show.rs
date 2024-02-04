use lighter_common::prelude::*;

use crate::entities::v1::users::Model;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;

pub async fn show(
    db: &DatabaseConnection,
    id: Uuid,
) -> Result<Json<UserWithPermissionAndRole>, Error> {
    let user = match Model::find_by_id(db, id).await {
        Some(user) => user,
        None => return Err(NotFound::new("User not found.").into()),
    };

    let permissions = user.permissions(db).await?;
    let roles = user.roles(db).await?;

    Ok(Json((user, permissions, roles).into()))
}
