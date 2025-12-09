use lighter_common::prelude::*;

use crate::middlewares::v1::auth::Authenticated as Cache;
use crate::middlewares::v1::auth::internal::Auth;
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
) -> Result<impl Responder, HttpError> {
    let response = services::v1::auth::login::login(&db, &cached, request).await?;
    Ok(Json(response))
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
pub async fn authenticated(auth: Auth) -> Result<impl Responder, HttpError> {
    let response = services::v1::auth::authenticated::authenticated(auth).await?;
    Ok(Json(response))
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
) -> Result<impl Responder, HttpError> {
    let response = services::v1::auth::logout::logout(auth, &db, &cached).await?;
    Ok(response)
}
