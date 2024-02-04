use crate::middlewares::v1::auth::internal::Auth;

pub async fn authenticated(auth: Auth) -> Auth {
    auth
}
