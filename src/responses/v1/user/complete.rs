use lighter_common::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

use crate::responses::v1::permission::Permission;
use crate::responses::v1::role::Role;

use super::simple::User;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema, IntoResponses, PartialEq, Eq, Hash)]
#[response(status = 200, description = "OK")]
#[serde(rename_all = "camelCase")]
pub struct UserWithPermissionAndRole {
    #[schema()]
    pub id: Uuid,
    #[schema(example = "John")]
    pub name: String,
    #[schema(example = "john@example")]
    pub email: String,
    #[schema(example = "2021-01-01T00:00:00+00:00")]
    pub email_verified_at: Option<NaiveDateTime>,
    #[schema(example = "john")]
    pub username: String,
    #[schema()]
    pub roles: Vec<Role>,
    #[schema()]
    pub permissions: Vec<Permission>,
}

impl<U, P, R> From<(U, Vec<P>, Vec<R>)> for UserWithPermissionAndRole
where
    U: Into<User>,
    P: Into<Permission>,
    R: Into<Role>,
{
    fn from((user, permissions, roles): (U, Vec<P>, Vec<R>)) -> Self {
        let user: User = user.into();

        Self {
            id: user.id,
            name: user.name,
            email: user.email,
            email_verified_at: user.email_verified_at,
            username: user.username,
            roles: roles.into_iter().map(|r| r.into()).collect(),
            permissions: permissions.into_iter().map(|p| p.into()).collect(),
        }
    }
}

impl From<UserWithPermissionAndRole> for User {
    fn from(val: UserWithPermissionAndRole) -> Self {
        User {
            id: val.id,
            name: val.name,
            email: val.email,
            email_verified_at: val.email_verified_at,
            username: val.username,
        }
    }
}
