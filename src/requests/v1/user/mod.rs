use lighter_common::prelude::Uuid;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserStoreRequest {
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "john.doe@example")]
    pub email: String,
    #[schema(example = "john.doe")]
    pub username: String,
    #[schema(example = "password")]
    pub password: String,
    #[schema(example = "password")]
    pub password_confirmation: String,
    #[schema()]
    pub profile_photo_id: Option<String>,
    #[schema()]
    pub permissions: Vec<Uuid>,
    #[schema()]
    pub roles: Vec<Uuid>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdateGeneralInformationRequest {
    #[schema(example = "John Doe")]
    pub name: String,
    #[schema(example = "john.doe@example")]
    pub email: String,
    #[schema(example = "john.doe")]
    pub username: String,
    #[schema()]
    pub profile_photo_id: Option<String>,
    #[schema()]
    pub permissions: Vec<Uuid>,
    #[schema()]
    pub roles: Vec<Uuid>,
}

#[derive(Clone, Deserialize, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserUpdatePasswordRequest {
    #[schema(example = "password")]
    pub current_password: String,
    #[schema(example = "password")]
    pub new_password: String,
    #[schema(example = "password")]
    pub password_confirmation: String,
}
