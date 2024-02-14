#[test]
pub async fn update_general_information() -> Result<(), lighter_common::prelude::Error> {
    use actix_web::body::MessageBody;
    use actix_web::http::Method;
    use actix_web::test::{call_service, TestRequest};
    use lighter_common::prelude::*;
    use sea_orm::EntityTrait;

    use crate::entities::v1::users;
    use crate::requests::v1::user::UserUpdateGeneralInformationRequest;
    use crate::testing::instance::token;

    let (service, db) = crate::service!();
    let id = Uuid::from_u128(0);
    let user = users::Entity::find_by_id(id).one(&db).await?.unwrap();
    let permissions = user.permissions(&db).await?;
    let roles = user.roles(&db).await?;
    let payload = UserUpdateGeneralInformationRequest {
        name: "unit test".to_string(),
        email: user.email.clone(),
        username: user.username.clone(),
        profile_photo_id: user.profile_photo_id,
        permissions: permissions.iter().map(|p| p.id).collect(),
        roles: roles.iter().map(|r| r.id).collect(),
    };

    let request = TestRequest::default()
        .insert_header(("Authorization", format!("Bearer {}", token(&db).await)))
        .insert_header(("Content-Type", "application/json"))
        .method(Method::PUT)
        .uri(format!("/v1/user/{}", user.id).as_str())
        .set_payload(serde_json::to_string(&payload).unwrap())
        .to_request();

    let response = call_service(&service, request).await;
    let status = response.status();
    let body = response.into_body().boxed().try_into_bytes().unwrap();

    assert_eq!(status, StatusCode::OK, "{:?}", body);

    let user = users::Entity::find_by_id(id).one(&db).await?.unwrap();

    assert_eq!(user.name, payload.name);

    Ok(())
}
