use crate::middlewares::v1::auth::internal::Auth;

#[::tracing::instrument(skip(auth), fields(user_id = %auth.user.id, token_id = %auth.id))]
pub async fn authenticated(auth: Auth) -> anyhow::Result<Auth> {
    ::tracing::debug!("Returning authenticated user information");
    Ok(auth)
}
