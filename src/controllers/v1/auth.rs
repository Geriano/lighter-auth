use lighter_common::prelude::*;

use crate::middlewares::v1::auth::internal::Auth;
use crate::middlewares::v1::auth::Authenticated as Cache;
use crate::requests::v1::auth::LoginRequest;
use crate::responses::v1::auth::Authenticated;
use crate::services;

/// Create a new session
///
/// Fail if:
/// - email or username not found
/// - password is incorrect
#[utoipa::path(
    tag = "Auth",
    responses(
        Authenticated,
        BadRequest,
        Unauthorized,
        Validation,
        InternalServerError,
    )
)]
#[post("/login")]
pub async fn login(
    db: Data<DatabaseConnection>,
    cached: Data<Cache>,
    Json(request): Json<LoginRequest>,
) -> impl Responder {
    services::v1::auth::login::login(&db, &cached, request).await
}

/// Get current session
///
/// Fail if:
/// - token not found
/// - token is expired
#[utoipa::path(
    tag = "Auth",
    security(("token" = [])),
    responses(
        Authenticated,
        Unauthorized,
        InternalServerError,
    )
)]
#[get("/user")]
pub async fn authenticated(auth: Auth) -> impl Responder {
    services::v1::auth::authenticated::authenticated(auth).await
}

/// Destroy current session
///
/// Fail if:
/// - token not found
/// - token is expired
#[utoipa::path(
    tag = "Auth",
    security(("token" = [])),
    responses(
        Success,
        Unauthorized,
        InternalServerError,
    )
)]
#[delete("/logout")]
pub async fn logout(
    auth: Auth,
    db: Data<DatabaseConnection>,
    cached: Data<Cache>,
) -> impl Responder {
    services::v1::auth::logout::logout(auth, &db, &cached).await
}
