use lighter_common::prelude::*;
use sea_orm::prelude::*;

use crate::entities::v1::tokens::{ActiveModel, Column, Entity, Model};
use crate::entities::v1::users;

impl Model {
    pub async fn user(db: &DatabaseConnection, id: Uuid) -> Option<users::Model> {
        let query = users::Entity::find()
            .inner_join(Entity)
            .filter(Column::Id.eq(id))
            .filter(
                Condition::any()
                    .add(Column::ExpiredAt.gt(now()))
                    .add(Column::ExpiredAt.is_null()),
            );

        match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find token by id");
                ::tracing::error!("Error: {}", e);

                None
            }
        }
    }

    pub async fn store(&self, db: &DatabaseConnection) -> Result<Self, DbErr> {
        ActiveModel::from(self.clone()).insert(db).await
    }

    pub async fn delete(&self, db: &DatabaseConnection) -> Result<(), DbErr> {
        Entity::delete_by_id(self.id).exec(db).await?;

        Ok(())
    }

    pub async fn logout(db: &DatabaseConnection, id: Uuid) -> Result<(), DbErr> {
        Entity::delete_many()
            .filter(Column::UserId.eq(id))
            .exec(db)
            .await?;

        Ok(())
    }
}
