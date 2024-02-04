use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    #[schema(example = "john.doe")]
    pub email_or_username: String,
    #[schema(example = "password")]
    pub password: String,
}
