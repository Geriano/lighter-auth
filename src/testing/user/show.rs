#[test]
pub async fn show() -> Result<(), lighter_common::prelude::Error> {
    use actix_web::body::MessageBody;
    use actix_web::test::{TestRequest, call_service};
    use lighter_common::prelude::*;
    use sea_orm::EntityTrait;

    use crate::entities::v1::users;
    use crate::responses::v1::user::complete::UserWithPermissionAndRole;
    use crate::testing::instance::token;

    let (service, db) = crate::service!();
    let model = users::Entity::find().one(&db).await;

    assert!(model.is_ok());

    let model = model.unwrap();

    assert!(model.is_some());

    let model = model.unwrap();
    let request = TestRequest::default()
        .insert_header(("Authorization", format!("Bearer {}", token(&db).await)))
        .uri(format!("/v1/user/{}", model.id).as_str())
        .to_request();

    let response = call_service(&service, request).await;
    let status = response.status();
    let body = response.into_body().boxed().try_into_bytes().unwrap();
    let body = serde_json::from_slice::<UserWithPermissionAndRole>(&body);

    assert_eq!(status, StatusCode::OK);
    assert!(body.is_ok());

    Ok(())
}
