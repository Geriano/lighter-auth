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
pub struct Role {
    #[schema()]
    pub id: Uuid,
    #[order(default)]
    #[schema(example = "MANAGER")]
    pub code: String,
    #[order]
    #[schema(example = "Manager")]
    pub name: String,
}
