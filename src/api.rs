use lighter_common::api::{Authentication, Builtin};
use utoipa::OpenApi;

use crate::{controllers, requests, responses};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Auth"),
        (name = "User"),
        (name = "Permission"),
        (name = "Role"),
        (name = "Health"),
    ),
    modifiers(&Builtin, &Authentication),
    paths(
        controllers::v1::user::paginate,
        controllers::v1::user::store,
        controllers::v1::user::show,
        controllers::v1::user::update_general_information,
        controllers::v1::user::update_password,
        controllers::v1::user::delete,

        controllers::v1::permission::paginate,
        controllers::v1::permission::store,
        controllers::v1::permission::show,
        controllers::v1::permission::update,
        controllers::v1::permission::delete,

        controllers::v1::role::paginate,
        controllers::v1::role::store,
        controllers::v1::role::show,
        controllers::v1::role::update,
        controllers::v1::role::delete,

        controllers::v1::auth::login,
        controllers::v1::auth::authenticated,
        controllers::v1::auth::logout,

        controllers::health::health,
        controllers::health::health_db,
        controllers::health::ready,
        controllers::health::live,
    ),
    components(schemas(
        requests::v1::auth::LoginRequest,
        requests::v1::user::UserStoreRequest,
        requests::v1::user::UserUpdateGeneralInformationRequest,
        requests::v1::user::UserUpdatePasswordRequest,
        requests::v1::permission::PermissionRequest,
        requests::v1::role::RoleRequest,

        responses::v1::user::simple::User,
        responses::v1::user::simple::UserPaginationSort,
        responses::v1::user::simple::UserPaginationOrder,
        responses::v1::user::simple::UserPaginationRequest,
        responses::v1::user::simple::UserPaginationResponse,
        responses::v1::user::complete::UserWithPermissionAndRole,

        responses::v1::permission::Permission,
        responses::v1::permission::PermissionPaginationSort,
        responses::v1::permission::PermissionPaginationOrder,
        responses::v1::permission::PermissionPaginationRequest,
        responses::v1::permission::PermissionPaginationResponse,

        responses::v1::role::Role,
        responses::v1::role::RolePaginationSort,
        responses::v1::role::RolePaginationOrder,
        responses::v1::role::RolePaginationRequest,
        responses::v1::role::RolePaginationResponse,

        controllers::health::HealthResponse,
        controllers::health::DatabaseHealthStatus,
    )),
)]
pub struct Definition;
