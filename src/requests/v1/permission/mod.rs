use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct PermissionRequest {
    #[schema(example = "Create User")]
    pub name: String,
}
