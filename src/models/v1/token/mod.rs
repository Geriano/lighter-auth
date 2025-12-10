use lighter_common::prelude::*;
use sea_orm::prelude::*;
use std::time::Instant;

use crate::entities::v1::tokens::{ActiveModel, Column, Entity, Model};
use crate::entities::v1::users;
use crate::metrics::AppMetrics;

impl Model {
    pub async fn user(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        id: Uuid,
    ) -> Option<users::Model> {
        let start = Instant::now();

        let query = users::Entity::find()
            .inner_join(Entity)
            .filter(Column::Id.eq(id))
            .filter(
                Condition::any()
                    .add(Column::ExpiredAt.gt(now()))
                    .add(Column::ExpiredAt.is_null()),
            );

        let result = match query.one(db).await {
            Ok(user) => user,
            Err(e) => {
                ::tracing::error!("Failed to find token by id");
                ::tracing::error!("Error: {}", e);

                None
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("token_user", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn store(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
    ) -> Result<Self, DbErr> {
        let start = Instant::now();
        let result = ActiveModel::from(self.clone()).insert(db).await;

        if let Some(m) = metrics {
            m.record_db_query("token_store", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn delete(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
    ) -> Result<(), DbErr> {
        let start = Instant::now();

        let result = Entity::delete_by_id(self.id).exec(db).await;

        if let Some(m) = metrics {
            m.record_db_query("token_delete", start.elapsed().as_secs_f64());
        }

        result?;
        Ok(())
    }

    pub async fn logout(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        id: Uuid,
    ) -> Result<(), DbErr> {
        let start = Instant::now();

        let result = Entity::delete_many()
            .filter(Column::UserId.eq(id))
            .exec(db)
            .await;

        if let Some(m) = metrics {
            m.record_db_query("token_logout", start.elapsed().as_secs_f64());
        }

        result?;
        Ok(())
    }
}
