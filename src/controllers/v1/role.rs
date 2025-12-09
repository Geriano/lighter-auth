use lighter_common::prelude::*;

use crate::helpers::AnyhowResponder;
use crate::requests::v1::role::RoleRequest;
use crate::responses::v1::role::{Role, RolePaginationRequest, RolePaginationResponse};
use crate::services;

/// Paginate roles
#[utoipa::path(
    tag = "Role",
    security(("token" = [])),
    responses(
        RolePaginationResponse,
        BadRequest,
        Unauthorized,
        InternalServerError,
    )
)]
#[get("/v1/role")]
pub async fn paginate(
    db: Data<DatabaseConnection>,
    QueryParam(request): QueryParam<RolePaginationRequest>,
) -> impl Responder {
    AnyhowResponder(services::v1::role::paginate::paginate(&db, request).await)
}

/// Store new role
///
/// Code field will take from name field and convert to uppercase and replace space with underscore
///
/// Fail if code already exist
#[utoipa::path(
    tag = "Role",
    security(("token" = [])),
    responses(Role, BadRequest, Unauthorized, Validation, InternalServerError,)
)]
#[post("/v1/role")]
pub async fn store(
    db: Data<DatabaseConnection>,
    Json(request): Json<RoleRequest>,
) -> impl Responder {
    AnyhowResponder(services::v1::role::store::store(&db, request).await)
}

/// Show role by id
///
/// Fail if role not found
#[utoipa::path(
    tag = "Role",
    security(("token" = [])),
    responses(Role, BadRequest, Unauthorized, NotFound, InternalServerError,)
)]
#[get("/v1/role/{id}")]
pub async fn show(db: Data<DatabaseConnection>, id: Path<Uuid>) -> impl Responder {
    AnyhowResponder(services::v1::role::show::show(&db, id.into_inner()).await)
}

/// Update role by id
///
/// Fail if role not found
#[utoipa::path(
    tag = "Role",
    security(("token" = [])),
    responses(Role, BadRequest, Unauthorized, NotFound, Validation, InternalServerError,)
)]
#[put("/v1/role/{id}")]
pub async fn update(
    db: Data<DatabaseConnection>,
    id: Path<Uuid>,
    Json(request): Json<RoleRequest>,
) -> impl Responder {
    AnyhowResponder(services::v1::role::update::update(&db, id.into_inner(), request).await)
}

/// Delete role by id
///
/// Fail if role not found
#[utoipa::path(
    tag = "Role",
    security(("token" = [])),
    responses(Success, BadRequest, Unauthorized, NotFound, InternalServerError,)
)]
#[delete("/v1/role/{id}")]
pub async fn delete(db: Data<DatabaseConnection>, id: Path<Uuid>) -> impl Responder {
    AnyhowResponder(services::v1::role::delete::delete(&db, id.into_inner()).await)
}
