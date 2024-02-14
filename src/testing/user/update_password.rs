#[test]
pub async fn update_password() -> Result<(), lighter_common::prelude::Error> {
    use actix_web::body::MessageBody;
    use actix_web::http::Method;
    use actix_web::test::{call_service, TestRequest};
    use lighter_auth_migration::Expr;
    use lighter_common::prelude::*;
    use sea_orm::EntityTrait;
    use sea_orm::{ColumnTrait, QueryFilter};

    use crate::entities::v1::users;
    use crate::requests::v1::user::UserUpdatePasswordRequest;
    use crate::testing::instance::token;

    let (service, db) = crate::service!();
    let id = Uuid::from_u128(0);
    let user = users::Entity::find_by_id(id).one(&db).await?.unwrap();
    let password = user.password.clone();
    let new_password = "new password".to_string();
    let request = TestRequest::default()
        .insert_header(("Authorization", format!("Bearer {}", token(&db).await)))
        .insert_header(("Content-Type", "application/json"))
        .set_json(&UserUpdatePasswordRequest {
            current_password: "password".to_string(),
            new_password: new_password.to_string(),
            password_confirmation: new_password.to_string(),
        })
        .method(Method::PUT)
        .uri(format!("/v1/user/{}/password", user.id).as_str())
        .to_request();

    let response = call_service(&service, request).await;
    let status = response.status();
    let body = response.into_body().boxed().try_into_bytes().unwrap();

    assert_eq!(status, StatusCode::OK, "{:?}", body);

    let user = users::Entity::find_by_id(id).one(&db).await?.unwrap();
    let hash = Hash::from(&user.password);

    assert!(hash.verify(user.id, new_password));

    users::Entity::update_many()
        .filter(users::Column::Id.eq(id))
        .col_expr(users::Column::Password, Expr::value(password))
        .exec(&db)
        .await?;

    Ok(())
}
