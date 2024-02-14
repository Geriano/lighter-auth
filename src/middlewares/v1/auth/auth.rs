use std::future::Future;
use std::pin::Pin;
use std::time::Instant;
use std::{env, fmt};

use actix_web::dev::Payload;
use actix_web::FromRequest;
use awc::Client;
use lighter_common::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::{IntoResponses, ToSchema};

use crate::responses::v1::permission::Permission;
use crate::responses::v1::role::Role;
use crate::responses::v1::user::complete::UserWithPermissionAndRole;
use crate::responses::v1::user::simple::User;

#[derive(Clone, Deserialize, Serialize, ToSchema, IntoResponses)]
#[serde(rename_all = "camelCase")]
#[response(status = 200, description = "OK")]
pub struct Auth {
    #[schema()]
    pub user: User,
    #[schema()]
    pub permissions: Vec<Permission>,
    #[schema()]
    pub roles: Vec<Role>,
}

impl Auth {
    pub fn json(&self) -> Value {
        serde_json::to_value(self).unwrap()
    }
}

impl fmt::Debug for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl fmt::Display for Auth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string_pretty(self).unwrap())
    }
}

impl Responder for Auth {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}

impl From<UserWithPermissionAndRole> for Auth {
    fn from(user: UserWithPermissionAndRole) -> Self {
        let permissions = user.permissions.clone();
        let roles = user.roles.clone();

        Self {
            user: user.into(),
            permissions,
            roles,
        }
    }
}

impl From<&UserWithPermissionAndRole> for Auth {
    fn from(user: &UserWithPermissionAndRole) -> Self {
        Self::from(user.clone())
    }
}

impl FromRequest for Auth {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let start = Instant::now();

        let header = match req.headers().get("Authorization").cloned() {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    tracing::error!("Failed to get authorization header");

                    Err(BadRequest::new("Missing authorization header").into())
                });
            }
        };

        Box::pin(async move {
            let url = env::var("AUTH_SERVICE_URL")
                .expect("AUTH_SERVICE_URL environment variable must be set");
            let request = Client::new()
                .get(url)
                .append_header(("authorization", header))
                .send()
                .await;

            let mut response = match request {
                Err(e) => {
                    return Err(Error::InternalServerError {
                        message: e.to_string(),
                    })
                }
                Ok(response) => response,
            };

            let status = response.status();
            let body = match response.body().await {
                Err(e) => {
                    return Err(Error::InternalServerError {
                        message: e.to_string(),
                    })
                }
                Ok(body) => serde_json::from_slice::<Value>(&body)?,
            };

            match status {
                StatusCode::OK | StatusCode::CREATED => (),
                _ => {
                    let message = match body.get("message") {
                        Some(message) => message.as_str().unwrap(),
                        None => "Unknown error",
                    }
                    .to_string();

                    match status {
                        StatusCode::BAD_REQUEST => return Err(Error::BadRequest { message }),
                        StatusCode::UNAUTHORIZED => return Err(Error::Unauthorized { message }),
                        _ => return Err(Error::InternalServerError { message }),
                    }
                }
            };

            let auth = serde_json::from_value::<Auth>(body)?;
            let elapsed = start.elapsed();

            tracing::info!("Authenticated in {:?}", elapsed);

            Ok(auth)
        })
    }
}
