use lighter_common::prelude::*;

use crate::requests::v1::user::{
    UserStoreRequest, UserUpdateGeneralInformationRequest, UserUpdatePasswordRequest,
};
use crate::responses::v1::user::complete::UserWithPermissionAndRole;
use crate::responses::v1::user::simple::{UserPaginationRequest, UserPaginationResponse};
use crate::services;

/// Paginate users
#[utoipa::path(
    tag = "User",
    security(("token" = [])),
    params(UserPaginationRequest),
    responses(
        UserPaginationResponse,
        BadRequest,
        Unauthorized,
        InternalServerError,
    ),
)]
#[get("/v1/user")]
pub async fn paginate(
    db: Data<DatabaseConnection>,
    QueryParam(request): QueryParam<UserPaginationRequest>,
) -> impl Responder {
    services::v1::user::paginate::paginate(&db, request).await
}

/// Store new user
///
/// Fail if
/// - email already exist
/// - username already exist
/// - password is too short
#[utoipa::path(
    tag = "User",
    security(("token" = [])),
    responses(
        UserWithPermissionAndRole,
        BadRequest,
        Unauthorized,
        Validation,
        InternalServerError,
    ),
)]
#[post("/v1/user")]
pub async fn store(
    db: Data<DatabaseConnection>,
    Json(request): Json<UserStoreRequest>,
) -> impl Responder {
    services::v1::user::store::store(&db, request).await
}

/// Find user by id
///
/// Fail if user not found
#[utoipa::path(
    tag = "User",
    security(("token" = [])),
    responses(
        UserWithPermissionAndRole,
        NotFound,
        Unauthorized,
        InternalServerError,
    ),
)]
#[get("/v1/user/{id}")]
pub async fn show(db: Data<DatabaseConnection>, id: Path<Uuid>) -> impl Responder {
    services::v1::user::show::show(&db, id.into_inner()).await
}

/// Update general information user by id
///
/// Fail if
/// - user not found
/// - email already exist
/// - username already exist
#[utoipa::path(
    tag = "User",
    security(("token" = [])),
    responses(
        Success,
        NotFound,
        BadRequest,
        Unauthorized,
        Validation,
        InternalServerError,
    ),
)]
#[put("/v1/user/{id}")]
pub async fn update_general_information(
    db: Data<DatabaseConnection>,
    id: Path<Uuid>,
    Json(request): Json<UserUpdateGeneralInformationRequest>,
) -> impl Responder {
    services::v1::user::update_general_information::update(&db, id.into_inner(), request).await
}

/// Update user password by id
///
/// Fail if
/// - user not found
/// - password is too short
/// - password is not match with confirm password
/// - old password is not match with current password
#[utoipa::path(
    tag = "User",
    security(("token" = [])),
    responses(
        Success,
        NotFound,
        BadRequest,
        Unauthorized,
        Validation,
        InternalServerError,
    ),
)]
#[put("/v1/user/{id}/password")]
pub async fn update_password(
    db: Data<DatabaseConnection>,
    id: Path<Uuid>,
    Json(request): Json<UserUpdatePasswordRequest>,
) -> impl Responder {
    services::v1::user::update_password::update(&db, id.into_inner(), request).await
}

/// Delete user by id
///
/// Fail if user not found
#[utoipa::path(
    tag = "User",
    security(("token" = [])),
    responses(
        Success,
        NotFound,
        Unauthorized,
        InternalServerError,
    ),
)]
#[delete("/v1/user/{id}")]
pub async fn delete(db: Data<DatabaseConnection>, id: Path<Uuid>) -> impl Responder {
    services::v1::user::delete::delete(&db, id.into_inner()).await
}
