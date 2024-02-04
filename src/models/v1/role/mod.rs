use lighter_common::prelude::*;
use sea_orm::prelude::*;

use crate::entities::v1::roles::{ActiveModel, Column, Entity, Model};
use crate::responses::v1::role::Role;

impl Model {
    pub async fn find_by_id(db: &DatabaseConnection, id: Uuid) -> Result<Option<Self>, DbErr> {
        match Entity::find_by_id(id).one(db).await {
            Ok(user) => Ok(user),
            Err(e) => {
                tracing::error!("Failed to find user by id");
                tracing::error!("Error: {}", e);

                Err(e)
            }
        }
    }

    pub async fn code_exist<T: ToString>(db: &DatabaseConnection, code: T) -> bool {
        let code = code.to_string().replace(" ", "_").to_uppercase();
        let query = Entity::find().filter(Column::Code.eq(code)).count(db).await;

        match query {
            Ok(count) => count > 0,
            Err(e) => {
                tracing::error!("Failed to check if code exist");
                tracing::error!("Error: {}", e);

                false
            }
        }
    }

    pub async fn store(&self, db: &DatabaseConnection) -> Result<Model, DbErr> {
        ActiveModel::from(self.clone()).insert(db).await
    }

    pub async fn update<T: ToString>(
        &self,
        db: &DatabaseConnection,
        name: T,
    ) -> Result<Model, DbErr> {
        let mut model = ActiveModel::from(self.clone());
        model.name = Set(name.to_string());

        model.update(db).await
    }

    pub async fn delete(&self, db: &DatabaseConnection) -> Result<(), DbErr> {
        Entity::delete_by_id(self.id).exec(db).await?;

        Ok(())
    }
}

impl Into<Role> for Model {
    fn into(self) -> Role {
        Role {
            id: self.id,
            code: self.code,
            name: self.name,
        }
    }
}

impl Into<Role> for &Model {
    fn into(self) -> Role {
        Role {
            id: self.id,
            code: self.code.clone(),
            name: self.name.clone(),
        }
    }
}
