use lighter_common::prelude::Uuid;
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Clone, Deserialize, ToSchema)]
pub struct RoleRequest {
    #[schema(example = "Manager")]
    pub name: String,
    #[schema()]
    pub permissions: Vec<Uuid>,
}
