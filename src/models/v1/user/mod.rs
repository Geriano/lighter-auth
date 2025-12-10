use lighter_common::prelude::*;
use sea_orm::QuerySelect;
use sea_orm::prelude::*;
use std::time::Instant;

use crate::entities::v1::users::{ActiveModel, Column, Entity, Model};
use crate::entities::v1::{
    permission_role, permission_user, permissions, role_user, roles, tokens,
};
use crate::metrics::AppMetrics;
use crate::responses::v1::user::simple::User;

impl Model {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        id: Uuid,
    ) -> Option<Self> {
        let start = Instant::now();

        let query = Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::DeletedAt.is_null());

        let result = match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by id");
                ::tracing::error!("Error: {}", e);

                None
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("user_find_by_id", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn find_by_email<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        email: T,
    ) -> Option<Self> {
        let start = Instant::now();

        let query = Entity::find()
            .filter(Column::Email.eq(email.to_string()))
            .filter(Column::DeletedAt.is_null());

        let result = match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by email");
                ::tracing::error!("Error: {}", e);

                None
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("user_find_by_email", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn find_by_username<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        username: T,
    ) -> Option<Self> {
        let start = Instant::now();

        let query = Entity::find()
            .filter(Column::Username.eq(username.to_string()))
            .filter(Column::DeletedAt.is_null());

        let result = match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by username");
                ::tracing::error!("Error: {}", e);

                None
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("user_find_by_username", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn find_by_email_or_username<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        email_or_username: T,
    ) -> Option<Self> {
        let start = Instant::now();

        let query = Entity::find()
            .filter(
                Condition::any()
                    .add(Column::Username.eq(email_or_username.to_string()))
                    .add(Column::Email.eq(email_or_username.to_string())),
            )
            .filter(Column::DeletedAt.is_null());

        let result = match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by username or email");
                ::tracing::error!("Error: {}", e);

                None
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("user_find_by_email_or_username", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn email_or_username_exists<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        email_or_username: T,
    ) -> bool {
        let start = Instant::now();

        let query = Entity::find()
            .filter(
                Condition::any()
                    .add(Column::Username.eq(email_or_username.to_string()))
                    .add(Column::Email.eq(email_or_username.to_string())),
            )
            .count(db);

        let result = query.await.unwrap_or(0) > 0;

        if let Some(m) = metrics {
            m.record_db_query("user_email_or_username_exists", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn email_exists<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        email: T,
    ) -> bool {
        let start = Instant::now();

        let query = Entity::find()
            .filter(Column::Email.eq(email.to_string()))
            .count(db);

        let result = query.await.unwrap_or(0) > 0;

        if let Some(m) = metrics {
            m.record_db_query("user_email_exists", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn username_exists<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        username: T,
    ) -> bool {
        let start = Instant::now();

        let query = Entity::find()
            .filter(Column::Username.eq(username.to_string()))
            .count(db);

        let result = query.await.unwrap_or(0) > 0;

        if let Some(m) = metrics {
            m.record_db_query("user_username_exists", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn store(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        permissions: Vec<permissions::Model>,
        roles: Vec<roles::Model>,
    ) -> Result<Self, TransactionError<DbErr>> {
        ::tracing::debug!("Model: user.store called with metrics={}", metrics.is_some());
        let start = Instant::now();

        let result = db.transaction(|db| {
            let user = self.clone();
            let permissions = permissions
                .iter()
                .map(|permission| {
                    let permission_user = permission_user::Model {
                        id: Uuid::new_v4(),
                        permission_id: permission.id,
                        user_id: user.id,
                    };

                    permission_user::ActiveModel::from(permission_user)
                })
                .collect::<Vec<_>>();
            let roles = roles
                .iter()
                .map(|role| {
                    let role_user = role_user::Model {
                        id: Uuid::new_v4(),
                        role_id: role.id,
                        user_id: user.id,
                    };

                    role_user::ActiveModel::from(role_user)
                })
                .collect::<Vec<_>>();

            Box::pin(async move {
                let user = ActiveModel::from(user).insert(db).await?;

                if !permissions.is_empty() {
                    permission_user::Entity::insert_many(permissions)
                        .exec(db)
                        .await?;
                }

                if !roles.is_empty() {
                    role_user::Entity::insert_many(roles).exec(db).await?;
                }

                Ok(user)
            })
        })
        .await;

        if let Some(m) = metrics {
            let duration = start.elapsed().as_secs_f64();
            ::tracing::debug!("Model: Recording user_store metric, duration={:.4}s", duration);
            m.record_db_query("user_store", duration);
            ::tracing::debug!("Model: Metric recorded successfully");
        } else {
            ::tracing::warn!("Model: user.store called but metrics is None!");
        }

        result
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_general_information<Name, Email, Username>(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        name: Name,
        email: Email,
        email_verified_at: Option<NaiveDateTime>,
        username: Username,
        profile_photo_id: Option<String>,
        permissions: Vec<permissions::Model>,
        roles: Vec<roles::Model>,
    ) -> Result<Self, TransactionError<DbErr>>
    where
        Name: ToString,
        Email: ToString,
        Username: ToString,
    {
        let start = Instant::now();
        let name = name.to_string();
        let email = email.to_string();
        let username = username.to_string();

        let result = db.transaction(|db| {
            let user = self.clone();
            let permissions = permissions
                .iter()
                .map(|permission| {
                    permission_user::ActiveModel::from(permission_user::Model {
                        id: Uuid::new_v4(),
                        permission_id: permission.id,
                        user_id: user.id,
                    })
                })
                .collect::<Vec<_>>();
            let roles = roles
                .iter()
                .map(|role| {
                    role_user::ActiveModel::from(role_user::Model {
                        id: Uuid::new_v4(),
                        role_id: role.id,
                        user_id: user.id,
                    })
                })
                .collect::<Vec<_>>();

            Box::pin(async move {
                let mut model = ActiveModel::from(user);

                model.name = Set(name);
                model.email = Set(email);
                model.email_verified_at = Set(email_verified_at);
                model.username = Set(username);
                model.profile_photo_id = Set(profile_photo_id);
                model.updated_at = Set(now());

                let user = model.update(db).await?;

                permission_user::Entity::delete_many()
                    .filter(permission_user::Column::UserId.eq(user.id))
                    .exec(db)
                    .await?;
                role_user::Entity::delete_many()
                    .filter(role_user::Column::UserId.eq(user.id))
                    .exec(db)
                    .await?;

                if !permissions.is_empty() {
                    permission_user::Entity::insert_many(permissions)
                        .exec(db)
                        .await?;
                }

                if !roles.is_empty() {
                    role_user::Entity::insert_many(roles).exec(db).await?;
                }

                Ok(user)
            })
        })
        .await;

        if let Some(m) = metrics {
            m.record_db_query("user_update_general_information", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn update_password(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        password: impl Into<String>,
    ) -> Result<Self, DbErr> {
        let start = Instant::now();
        let mut model = ActiveModel::from(self.clone());

        model.password = Set(password.into());
        model.updated_at = Set(now());
        let result = model.update(db).await;

        if let Some(m) = metrics {
            m.record_db_query("user_update_password", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn soft_delete(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
    ) -> Result<Self, DbErr> {
        let start = Instant::now();
        let mut model = ActiveModel::from(self.clone());

        model.deleted_at = Set(Some(now()));
        let result = model.update(db).await;

        if let Some(m) = metrics {
            m.record_db_query("user_soft_delete", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn permissions(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
    ) -> Result<Vec<permissions::Model>, DbErr> {
        let start = Instant::now();

        let query = permissions::Entity::find()
            .join(
                JoinType::LeftJoin,
                permissions::Relation::PermissionUser.def(),
            )
            .join(
                JoinType::LeftJoin,
                permissions::Relation::PermissionRole.def(),
            )
            .join(JoinType::LeftJoin, permission_role::Relation::Roles.def())
            .join(JoinType::LeftJoin, roles::Relation::RoleUser.def())
            .filter(permissions::Column::Id.is_not_null())
            .filter(
                Condition::any()
                    .add(permission_user::Column::UserId.eq(self.id))
                    .add(role_user::Column::UserId.eq(self.id)),
            )
            .group_by(permissions::Column::Id);

        let result = query.all(db).await;

        if let Some(m) = metrics {
            m.record_db_query("user_permissions", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn roles(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
    ) -> Result<Vec<roles::Model>, DbErr> {
        let start = Instant::now();

        let query = roles::Entity::find()
            .inner_join(role_user::Entity)
            .filter(role_user::Column::UserId.eq(self.id));

        let result = query.all(db).await;

        if let Some(m) = metrics {
            m.record_db_query("user_roles", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn generate_token(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        expired_at: Option<NaiveDateTime>,
    ) -> Result<tokens::Model, DbErr> {
        let start = Instant::now();
        let token = tokens::Model {
            id: Uuid::new_v4(),
            user_id: self.id,
            expired_at,
        };

        let result = token.store(db, metrics).await;

        if let Some(m) = metrics {
            m.record_db_query("user_generate_token", start.elapsed().as_secs_f64());
        }

        result
    }
}

impl From<Model> for User {
    fn from(val: Model) -> Self {
        User {
            id: val.id,
            name: val.name,
            email: val.email,
            email_verified_at: val.email_verified_at,
            username: val.username,
        }
    }
}

impl From<&Model> for User {
    fn from(val: &Model) -> Self {
        User {
            id: val.id,
            name: val.name.clone(),
            email: val.email.clone(),
            email_verified_at: val.email_verified_at,
            username: val.username.clone(),
        }
    }
}
