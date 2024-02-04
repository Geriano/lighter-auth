use lighter_common::{base58, prelude::*};
use serde::{Deserialize, Serialize};
use utoipa::{IntoResponses, ToSchema};

use crate::middlewares::v1::auth::internal::Auth;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;

#[derive(Clone, Debug, Deserialize, Serialize, ToSchema, IntoResponses, PartialEq, Eq, Hash)]
#[response(status = 201, description = "Auhenticated")]
pub struct Authenticated {
    #[schema()]
    pub token: String,
    #[schema()]
    pub user: UserWithPermissionAndRole,
}

impl From<Auth> for Authenticated {
    fn from(auth: Auth) -> Self {
        Self {
            token: base58::to_string(auth.id),
            user: (auth.user, auth.permissions, auth.roles).into(),
        }
    }
}

impl Responder for Authenticated {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Created().json(self)
    }
}
