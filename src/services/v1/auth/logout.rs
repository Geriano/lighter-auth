use anyhow::Context;
use lighter_common::prelude::*;

use crate::entities::v1::tokens::Model;
use crate::middlewares::v1::auth::Authenticated as Cache;
use crate::middlewares::v1::auth::internal::Auth;

#[::tracing::instrument(skip(auth, db, cached), fields(user_id = %auth.user.id, token_id = %auth.id))]
pub async fn logout(auth: Auth, db: &DatabaseConnection, cached: &Cache) -> anyhow::Result<Success> {
    Model::logout(db, auth.user.id)
        .await
        .context("Failed to logout user and delete tokens from database")?;

    cached.remove(auth.id).await;

    Ok(Success)
}
