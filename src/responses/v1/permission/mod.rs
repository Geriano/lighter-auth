use lighter_common::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

#[derive(
    Clone,
    Debug,
    Deserialize,
    Serialize,
    ToSchema,
    IntoResponses,
    PartialEq,
    Eq,
    Hash,
    PaginationRequest,
    PaginationResponse,
)]
#[response(status = 200, description = "OK")]
pub struct Permission {
    #[schema()]
    pub id: Uuid,
    #[order(default)]
    #[schema(example = "CREATE_USER")]
    pub code: String,
    #[order]
    #[schema(example = "Create User")]
    pub name: String,
}
