use lighter_common::prelude::*;
use sea_orm::prelude::*;
use std::time::Instant;

use crate::entities::v1::roles::{ActiveModel, Column, Entity, Model};
use crate::metrics::AppMetrics;
use crate::responses::v1::role::Role;

impl Model {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        id: Uuid,
    ) -> Result<Option<Self>, DbErr> {
        let start = Instant::now();

        let result = match Entity::find_by_id(id).one(db).await {
            Ok(user) => Ok(user),
            Err(e) => {
                ::tracing::error!("Failed to find user by id");
                ::tracing::error!("Error: {}", e);

                Err(e)
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("role_find_by_id", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn code_exist<T: ToString>(
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        code: T,
    ) -> bool {
        let start = Instant::now();
        let code = code.to_string().replace(" ", "_").to_uppercase();
        let query = Entity::find().filter(Column::Code.eq(code)).count(db).await;

        let result = match query {
            Ok(count) => count > 0,
            Err(e) => {
                ::tracing::error!("Failed to check if code exist");
                ::tracing::error!("Error: {}", e);

                false
            }
        };

        if let Some(m) = metrics {
            m.record_db_query("role_code_exist", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn store(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
    ) -> Result<Model, DbErr> {
        let start = Instant::now();
        let result = ActiveModel::from(self.clone()).insert(db).await;

        if let Some(m) = metrics {
            m.record_db_query("role_store", start.elapsed().as_secs_f64());
        }

        result
    }

    pub async fn update<T: ToString>(
        &self,
        db: &DatabaseConnection,
        metrics: Option<&AppMetrics>,
        name: T,
    ) -> Result<Model, DbErr> {
        let start = Instant::now();
        let mut model = ActiveModel::from(self.clone());
        model.name = Set(name.to_string());

        let result = model.update(db).await;

        if let Some(m) = metrics {
            m.record_db_query("role_update", start.elapsed().as_secs_f64());
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
            m.record_db_query("role_delete", start.elapsed().as_secs_f64());
        }

        result?;
        Ok(())
    }
}

impl From<Model> for Role {
    fn from(val: Model) -> Self {
        Role {
            id: val.id,
            code: val.code,
            name: val.name,
        }
    }
}

impl From<&Model> for Role {
    fn from(val: &Model) -> Self {
        Role {
            id: val.id,
            code: val.code.clone(),
            name: val.name.clone(),
        }
    }
}
