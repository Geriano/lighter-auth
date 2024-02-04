use lighter_common::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

#[derive(
    Clone,
    Debug,
    Serialize,
    Deserialize,
    ToSchema,
    IntoResponses,
    PartialEq,
    Eq,
    Hash,
    PaginationRequest,
    PaginationResponse,
)]
#[response(status = 200, description = "OK")]
#[serde(rename_all = "camelCase")]
pub struct User {
    #[schema()]
    pub id: Uuid,
    #[order]
    #[schema(example = "John")]
    pub name: String,
    #[order]
    #[schema(example = "john@example")]
    pub email: String,
    #[order]
    #[schema(example = "2021-01-01T00:00:00+00:00")]
    pub email_verified_at: Option<NaiveDateTime>,
    #[order]
    #[schema(example = "john")]
    pub username: String,
}
