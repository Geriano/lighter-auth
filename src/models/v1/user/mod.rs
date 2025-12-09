use lighter_common::prelude::*;
use sea_orm::QuerySelect;
use sea_orm::prelude::*;

use crate::entities::v1::users::{ActiveModel, Column, Entity, Model};
use crate::entities::v1::{
    permission_role, permission_user, permissions, role_user, roles, tokens,
};
use crate::responses::v1::user::simple::User;

impl Model {
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Option<Self> {
        let query = Entity::find()
            .filter(Column::Id.eq(id))
            .filter(Column::DeletedAt.is_null());

        match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by id");
                ::tracing::error!("Error: {}", e);

                None
            }
        }
    }

    pub async fn find_by_email<T: ToString>(db: &DatabaseConnection, email: T) -> Option<Self> {
        let query = Entity::find()
            .filter(Column::Email.eq(email.to_string()))
            .filter(Column::DeletedAt.is_null());

        match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by email");
                ::tracing::error!("Error: {}", e);

                None
            }
        }
    }

    pub async fn find_by_username<T: ToString>(
        db: &DatabaseConnection,
        username: T,
    ) -> Option<Self> {
        let query = Entity::find()
            .filter(Column::Username.eq(username.to_string()))
            .filter(Column::DeletedAt.is_null());

        match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by username");
                ::tracing::error!("Error: {}", e);

                None
            }
        }
    }

    pub async fn find_by_email_or_username<T: ToString>(
        db: &DatabaseConnection,
        email_or_username: T,
    ) -> Option<Self> {
        let query = Entity::find()
            .filter(
                Condition::any()
                    .add(Column::Username.eq(email_or_username.to_string()))
                    .add(Column::Email.eq(email_or_username.to_string())),
            )
            .filter(Column::DeletedAt.is_null());

        match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find user by username or email");
                ::tracing::error!("Error: {}", e);

                None
            }
        }
    }

    pub async fn email_or_username_exists<T: ToString>(
        db: &DatabaseConnection,
        email_or_username: T,
    ) -> bool {
        let query = Entity::find()
            .filter(
                Condition::any()
                    .add(Column::Username.eq(email_or_username.to_string()))
                    .add(Column::Email.eq(email_or_username.to_string())),
            )
            .count(db);

        query.await.unwrap_or(0) > 0
    }

    pub async fn email_exists<T: ToString>(db: &DatabaseConnection, email: T) -> bool {
        let query = Entity::find()
            .filter(Column::Email.eq(email.to_string()))
            .count(db);

        query.await.unwrap_or(0) > 0
    }

    pub async fn username_exists<T: ToString>(db: &DatabaseConnection, username: T) -> bool {
        let query = Entity::find()
            .filter(Column::Username.eq(username.to_string()))
            .count(db);

        query.await.unwrap_or(0) > 0
    }

    pub async fn store(
        &self,
        db: &DatabaseConnection,
        permissions: Vec<permissions::Model>,
        roles: Vec<roles::Model>,
    ) -> Result<Self, TransactionError<DbErr>> {
        db.transaction(|db| {
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
        .await
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn update_general_information<Name, Email, Username>(
        &self,
        db: &DatabaseConnection,
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
        let name = name.to_string();
        let email = email.to_string();
        let username = username.to_string();

        db.transaction(|db| {
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
        .await
    }

    pub async fn update_password(
        &self,
        db: &DatabaseConnection,
        password: Hash,
    ) -> Result<Self, DbErr> {
        let mut model = ActiveModel::from(self.clone());

        model.password = Set(password.to_string());
        model.updated_at = Set(now());
        model.update(db).await
    }

    pub async fn soft_delete(&self, db: &DatabaseConnection) -> Result<Self, DbErr> {
        let mut model = ActiveModel::from(self.clone());

        model.deleted_at = Set(Some(now()));
        model.update(db).await
    }

    pub async fn permissions(
        &self,
        db: &DatabaseConnection,
    ) -> Result<Vec<permissions::Model>, DbErr> {
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

        query.all(db).await
    }

    pub async fn roles(&self, db: &DatabaseConnection) -> Result<Vec<roles::Model>, DbErr> {
        let query = roles::Entity::find()
            .inner_join(role_user::Entity)
            .filter(role_user::Column::UserId.eq(self.id));

        query.all(db).await
    }

    pub async fn generate_token(
        &self,
        db: &DatabaseConnection,
        expired_at: Option<NaiveDateTime>,
    ) -> Result<tokens::Model, DbErr> {
        let token = tokens::Model {
            id: Uuid::new_v4(),
            user_id: self.id,
            expired_at,
        };

        token.store(db).await
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
