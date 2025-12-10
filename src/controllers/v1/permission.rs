use lighter_common::prelude::*;

use crate::metrics::AppMetrics;
use crate::middlewares::v1::auth::internal::Auth;
use crate::requests::v1::permission::PermissionRequest;
use crate::responses::v1::permission::{
    Permission, PermissionPaginationRequest, PermissionPaginationResponse,
};
use crate::services;

/// Paginate permissions
#[utoipa::path(
    tag = "Permission",
    security(("token" = [])),
    params(PermissionPaginationRequest),
    responses(
        PermissionPaginationResponse,
        BadRequest,
        Unauthorized,
        InternalServerError,
    )
)]
#[get("/v1/permission")]
pub async fn paginate(
    _: Auth,
    db: Data<DatabaseConnection>,
    QueryParam(request): QueryParam<PermissionPaginationRequest>,
) -> Result<impl Responder, HttpError> {
    let response = services::v1::permission::paginate::paginate(&db, request).await?;
    Ok(Json(response))
}

/// Store new permission
///
/// Code field will take from name field and convert to uppercase and replace space with underscore
///
/// Fail if code already exist
#[utoipa::path(
    tag = "Permission",
    security(("token" = [])),
    responses(Permission, BadRequest, Unauthorized, Validation, InternalServerError,)
)]
#[post("/v1/permission")]
pub async fn store(
    db: Data<DatabaseConnection>,
    metrics: Data<AppMetrics>,
    Json(request): Json<PermissionRequest>,
) -> Result<impl Responder, HttpError> {
    let response = services::v1::permission::store::store(&db, Some(&metrics), request).await?;
    Ok(Json(response))
}

/// Show permission by id
///
/// Fail if permission not found
#[utoipa::path(
    tag = "Permission",
    security(("token" = [])),
    responses(Permission, BadRequest, Unauthorized, NotFound, InternalServerError,)
)]
#[get("/v1/permission/{id}")]
pub async fn show(
    db: Data<DatabaseConnection>,
    metrics: Data<AppMetrics>,
    id: Path<Uuid>,
) -> Result<impl Responder, HttpError> {
    let response = services::v1::permission::show::show(&db, Some(&metrics), id.into_inner()).await?;
    Ok(Json(response))
}

/// Update permission by id
///
/// Fail if permission not found
#[utoipa::path(
    tag = "Permission",
    security(("token" = [])),
    responses(Permission, BadRequest, Unauthorized, NotFound, Validation, InternalServerError,)
)]
#[put("/v1/permission/{id}")]
pub async fn update(
    db: Data<DatabaseConnection>,
    metrics: Data<AppMetrics>,
    id: Path<Uuid>,
    Json(request): Json<PermissionRequest>,
) -> Result<impl Responder, HttpError> {
    let response = services::v1::permission::update::update(&db, Some(&metrics), id.into_inner(), request).await?;
    Ok(Json(response))
}

/// Delete permission by id
///
/// Fail if permission not found
#[utoipa::path(
    tag = "Permission",
    security(("token" = [])),
    responses(Success, BadRequest, Unauthorized, NotFound, InternalServerError,)
)]
#[delete("/v1/permission/{id}")]
pub async fn delete(
    db: Data<DatabaseConnection>,
    metrics: Data<AppMetrics>,
    id: Path<Uuid>,
) -> Result<impl Responder, HttpError> {
    let response = services::v1::permission::delete::delete(&db, Some(&metrics), id.into_inner()).await?;
    Ok(response)
}
