#[test]
pub async fn store() -> Result<(), lighter_common::prelude::Error> {
    use actix_web::http::Method;
    use actix_web::test::{TestRequest, call_service};
    use lighter_common::prelude::*;
    use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

    use crate::entities::v1::users;
    use crate::requests::v1::user::UserStoreRequest;
    use crate::testing::instance::token;

    let payload = UserStoreRequest {
        name: "John Doe".to_string(),
        email: "john.doe@local.test".to_string(),
        username: "john_doe".to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        profile_photo_id: None,
        permissions: Vec::new(),
        roles: Vec::new(),
    };

    let (service, db) = crate::service!();
    let request = TestRequest::default()
        .insert_header(("Authorization", format!("Bearer {}", token(&db).await)))
        .insert_header(("Content-Type", "application/json"))
        .uri("/v1/user")
        .method(Method::POST)
        .set_payload(serde_json::to_string(&payload).unwrap())
        .to_request();

    let response = call_service(&service, request).await;
    let status = response.status();

    assert_eq!(status, StatusCode::OK);

    let model = users::Entity::find()
        .filter(users::Column::Username.eq(payload.username.clone()))
        .one(&db)
        .await;

    assert!(model.is_ok());

    let model = model.unwrap();

    assert!(model.is_some());

    let model = model.unwrap();

    users::Entity::delete_by_id(model.id).exec(&db).await?;

    Ok(())
}
