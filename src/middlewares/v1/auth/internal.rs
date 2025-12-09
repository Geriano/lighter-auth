use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use actix_web::FromRequest;
use actix_web::dev::Payload;
use lighter_common::{base58, prelude::*};
use sea_orm::EntityTrait;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::v1::{tokens, users};
use crate::responses::v1::permission::Permission;
use crate::responses::v1::role::Role;
use crate::responses::v1::user::simple::User;

use super::Authenticated;

#[derive(Clone, Deserialize, Serialize, ToSchema)]
pub struct Auth {
    #[serde(skip)]
    pub id: Uuid,
    #[schema()]
    pub user: User,
    #[schema()]
    pub permissions: Vec<Permission>,
    #[schema()]
    pub roles: Vec<Role>,
}

impl FromRequest for Auth {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let start = std::time::Instant::now();

        let db = match req.app_data::<Data<DatabaseConnection>>().cloned() {
            Some(db) => db,
            None => {
                return Box::pin(async move {
                    ::tracing::error!("Failed to get database connection");

                    Err(InternalServerError::new("Failed to get database connection").into())
                });
            }
        };

        let authenticated = match req.app_data::<Data<Authenticated>>().cloned() {
            Some(authenticated) => authenticated,
            None => {
                return Box::pin(async move {
                    ::tracing::error!("Failed to get authenticated user");

                    Err(InternalServerError::new("Failed to get authenticated user").into())
                });
            }
        };

        let header = match req.headers().get("Authorization").cloned() {
            Some(header) => header,
            None => {
                return Box::pin(async move {
                    ::tracing::error!("Failed to get authorization header");

                    Err(BadRequest::new("Missing authorization header").into())
                });
            }
        };

        let header = match header.to_str() {
            Ok(header) => header,
            Err(e) => {
                return Box::pin(async move {
                    ::tracing::error!("Failed to convert header to string");
                    ::tracing::error!("Error: {}", e);

                    Err(BadRequest::new("Failed to convert header to string").into())
                });
            }
        };

        if !header.starts_with("Bearer ") {
            return Box::pin(async move {
                ::tracing::error!("Invalid authorization header");

                Err(BadRequest::new("Invalid authorization header").into())
            });
        }

        let token = header.trim_start_matches("Bearer ");
        let token = match base58::decode(token) {
            Ok(token) => token,
            Err(e) => {
                return Box::pin(async move {
                    ::tracing::error!("Failed to decode token");
                    ::tracing::error!("Error: {}", e);

                    Err(BadRequest::new("Failed to decode token").into())
                });
            }
        };

        let id = match Uuid::from_slice(&token) {
            Ok(id) => id,
            Err(e) => {
                return Box::pin(async move {
                    ::tracing::error!("Failed to convert token to uuid");
                    ::tracing::error!("Error: {}", e);

                    Err(BadRequest::new("Failed to convert token to uuid").into())
                });
            }
        };

        Box::pin(async move {
            // Check cache first
            let cached_auth = match authenticated.get(id).await {
                Ok(Some(auth)) => {
                    ::tracing::debug!("Cache hit, verifying token still exists in database");
                    Some(auth)
                }
                Ok(None) => {
                    ::tracing::debug!("Cache miss, querying database");
                    None
                }
                Err(e) => {
                    ::tracing::warn!(error = %e, "Cache error, falling back to database");
                    None
                }
            };

            // Always check database to ensure token wasn't deleted (e.g., after logout)
            let db: &DatabaseConnection = &db;
            let token = tokens::Entity::find_by_id(id)
                .find_with_related(users::Entity)
                .all(db)
                .await?;

            let token = token.first().cloned();
            let (token, user) = match token {
                Some(token) => token,
                None => {
                    ::tracing::error!("Token not found in database (may have been deleted)");

                    // Remove from cache if it was there
                    if cached_auth.is_some() {
                        if let Err(e) = authenticated.remove(id).await {
                            ::tracing::warn!(error = %e, "Failed to remove stale auth from cache");
                        }
                    }

                    return Err(Unauthorized::new("Token not found").into());
                }
            };

            if let Some(expired_at) = token.expired_at
                && expired_at < now()
            {
                ::tracing::error!("Token expired");

                // Remove expired token from cache
                if let Err(e) = authenticated.remove(id).await {
                    ::tracing::warn!(error = %e, "Failed to remove expired token from cache");
                }

                return Err(Unauthorized::new("Token expired").into());
            }

            // If we had cached auth and token is still valid in DB, use cached auth
            if let Some(auth) = cached_auth {
                ::tracing::info!("Authentication from cache (verified in DB) took: {:?}", start.elapsed());
                return Ok(auth);
            }

            // Cache miss: fetch full auth data from database
            let user = user.first().cloned().unwrap();
            let permissions = user.permissions(db).await?;
            let roles = user.roles(db).await?;
            let auth = Auth {
                id: token.id,
                user: user.into(),
                permissions: permissions
                    .into_iter()
                    .map(|permission| permission.into())
                    .collect(),
                roles: roles.into_iter().map(|role| role.into()).collect(),
            };

            // Cache the auth (log errors but don't fail)
            if let Err(e) = authenticated.set(id, &auth).await {
                ::tracing::warn!(error = %e, "Failed to cache auth");
            }

            if let Err(e) = authenticated.remove_delay(id, Duration::from_secs(60 * 5)).await {
                ::tracing::warn!(error = %e, "Failed to schedule cache removal");
            }

            ::tracing::info!("Authentication from database took: {:?}", start.elapsed());

            Ok(auth)
        })
    }
}

impl Responder for Auth {
    type Body = <Json<Self> as Responder>::Body;

    fn respond_to(self, req: &HttpRequest) -> HttpResponse<Self::Body> {
        Json(self).respond_to(req)
    }
}
