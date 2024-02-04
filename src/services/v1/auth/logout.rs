use lighter_common::prelude::*;

use crate::entities::v1::tokens::Model;
use crate::middlewares::v1::auth::internal::Auth;
use crate::middlewares::v1::auth::Authenticated as Cache;

pub async fn logout(auth: Auth, db: &DatabaseConnection, cached: &Cache) -> Result<Success, Error> {
    Model::logout(db, auth.user.id).await?;
    cached.remove(auth.id).await;

    Ok(Success)
}
