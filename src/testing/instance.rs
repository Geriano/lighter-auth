use lighter_auth_migration::MigratorTrait;
use lighter_common::{base58, prelude::*};
use sea_orm::{ActiveModelTrait, ColumnTrait, DbErr, EntityTrait, QueryFilter};

use crate::entities::v1::tokens;

pub async fn token(db: &DatabaseConnection) -> String {
    let user_id = Uuid::from_u128(0);
    let token = tokens::Entity::find()
        .filter(tokens::Column::UserId.eq(user_id))
        .one(db)
        .await
        .unwrap();

    base58::to_string(match token {
        Some(token) => token.id,
        None => {
            let id = Uuid::new_v4();
            let model = tokens::Model {
                id,
                user_id,
                expired_at: None,
            };

            let model = tokens::ActiveModel::from(model);
            model.insert(db).await.ok();

            id
        }
    })
}

pub async fn database() -> Result<DatabaseConnection, DbErr> {
    let db = database::env().await?;

    lighter_auth_migration::Migrator::up(&db, None).await?;

    Ok(db)
}

#[test]
async fn database_connected() {
    let db = database().await.unwrap();

    assert_eq!(db.ping().await, Ok(()));
}

#[macro_export]
macro_rules! app {
    () => {};
}

#[macro_export]
macro_rules! service {
    () => {{
        let db = crate::testing::instance::database().await.unwrap();
        let app = ::actix_web::App::new()
            .app_data(::actix_web::web::Data::new(db.clone()))
            .app_data(::actix_web::web::Data::new(
                crate::middlewares::v1::auth::Authenticated::new(),
            ))
            .configure(crate::router::route);

        let service = ::actix_web::test::init_service(app).await;

        (service, db)
    }};
}
