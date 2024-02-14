#[test]
pub async fn paginate() -> Result<(), lighter_common::prelude::Error> {
    use actix_web::body::MessageBody;
    use actix_web::test::{call_service, TestRequest};
    use lighter_common::prelude::*;

    use crate::responses::v1::user::simple::UserPaginationResponse;
    use crate::testing::instance::token;

    let (service, db) = crate::service!();
    let request = TestRequest::default()
        .insert_header(("Authorization", format!("Bearer {}", token(&db).await)))
        .uri("/v1/user")
        .to_request();

    let response = call_service(&service, request).await;
    let status = response.status();
    let body = response.into_body().boxed().try_into_bytes().unwrap();

    assert_eq!(status, StatusCode::OK, "{:?}", body);

    let body = serde_json::from_slice::<UserPaginationResponse>(&body);

    assert!(body.is_ok());

    Ok(())
}
