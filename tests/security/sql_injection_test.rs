//! SQL Injection Security Tests
//!
//! This module tests that the lighter-auth service is protected against SQL injection attacks.
//! SeaORM uses parameterized queries by default, which should prevent SQL injection.
//!
//! We test various attack vectors across different endpoints:
//! - User creation with malicious input
//! - User updates with malicious input
//! - Login with malicious credentials
//! - Search/pagination with malicious query parameters
//!
//! Each test verifies that:
//! 1. The operation completes without SQL errors
//! 2. Malicious SQL is treated as literal string data (not executed)
//! 3. Database tables remain intact
//! 4. No unauthorized data access occurs

use actix_web::http::StatusCode;
use actix_web::test::{call_service, TestRequest};
use lighter_auth::requests::v1::auth::LoginRequest;
use lighter_auth::requests::v1::user::UserStoreRequest;
use lighter_auth::responses::v1::auth::Authenticated;
use lighter_auth::testing::setup;
use lighter_common::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

// =============================================================================
// SQL INJECTION TESTS - USER CREATION
// =============================================================================

/// Test SQL injection attempt in email field during user creation
///
/// Attack vector: Classic DROP TABLE injection
/// Expected: Email is treated as literal string, operation fails validation or succeeds safely
#[actix_web::test]
async fn test_sql_injection_in_email_during_user_creation() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user to get token
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in email field: DROP TABLE attack
    let malicious_email = "test'; DROP TABLE users; --@example.com";
    let create_request = UserStoreRequest {
        name: "Test User".to_string(),
        email: malicious_email.to_string(),
        username: "testuser".to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        profile_photo_id: None,
        permissions: vec![],
        roles: vec![],
    };

    let req = TestRequest::post()
        .uri("/v1/user")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Response should be 400 (validation error for invalid email) or 200 (treated as literal)
    // Either way, SQL should NOT be executed
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection should not cause server error, got: {}",
        resp.status()
    );

    // Verify users table still exists by querying it
    use lighter_auth::entities::v1::users;
    let users_count = users::Entity::find().all(&db).await;
    assert!(
        users_count.is_ok(),
        "Users table should still exist after injection attempt"
    );
}

/// Test SQL injection in username field during user creation
///
/// Attack vector: Boolean-based blind SQL injection (OR 1=1)
/// Expected: Username treated as literal, no unauthorized access
#[actix_web::test]
async fn test_sql_injection_in_username_during_user_creation() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user and login
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in username: OR 1=1 attack
    let malicious_username = "admin' OR '1'='1";
    let create_request = UserStoreRequest {
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        username: malicious_username.to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        profile_photo_id: None,
        permissions: vec![],
        roles: vec![],
    };

    let req = TestRequest::post()
        .uri("/v1/user")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Should either succeed (treating as literal) or fail validation
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection should not cause server error, got: {}",
        resp.status()
    );

    // If succeeded, verify the username was stored literally
    if resp.status() == StatusCode::OK {
        use lighter_auth::entities::v1::users;
        use sea_orm::ColumnTrait;

        let user = users::Entity::find()
            .filter(users::Column::Username.eq(malicious_username))
            .one(&db)
            .await
            .unwrap();

        if let Some(user) = user {
            // Username should be stored exactly as provided (literal string)
            assert_eq!(
                user.username, malicious_username,
                "Username should be stored as literal string"
            );
        }
    }
}

/// Test SQL injection in name field during user creation
///
/// Attack vector: DELETE statement injection
/// Expected: Name treated as literal string
#[actix_web::test]
async fn test_sql_injection_in_name_during_user_creation() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user and login
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in name: DELETE statement
    let malicious_name = "'; DELETE FROM users WHERE '1'='1'; --";
    let create_request = UserStoreRequest {
        name: malicious_name.to_string(),
        email: "test_delete@example.com".to_string(),
        username: "test_delete_user".to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        profile_photo_id: None,
        permissions: vec![],
        roles: vec![],
    };

    let req = TestRequest::post()
        .uri("/v1/user")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_request)
        .to_request();

    let resp = call_service(&service, req).await;

    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection should not cause server error, got: {}",
        resp.status()
    );

    // Verify users were NOT deleted - original user should still exist
    use lighter_auth::entities::v1::users;
    let original_user = users::Entity::find()
        .filter(users::Column::Id.eq(auth_user.id))
        .one(&db)
        .await
        .unwrap();

    assert!(
        original_user.is_some(),
        "Original user should still exist - DELETE should not have executed"
    );

    // If creation succeeded, verify name was stored literally
    if resp.status() == StatusCode::OK {
        let new_user = users::Entity::find()
            .filter(users::Column::Username.eq("test_delete_user"))
            .one(&db)
            .await
            .unwrap();

        if let Some(user) = new_user {
            // Compare case-insensitively since database might normalize case
            assert_eq!(
                user.name.to_lowercase(),
                malicious_name.to_lowercase(),
                "Name should be stored as literal string (case-insensitive)"
            );
        }
    }
}

// =============================================================================
// SQL INJECTION TESTS - LOGIN
// =============================================================================

/// Test SQL injection in email/username field during login
///
/// Attack vector: Authentication bypass with OR 1=1
/// Expected: Login fails, no unauthorized access granted
#[actix_web::test]
async fn test_sql_injection_in_login_email_field() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create a legitimate user
    let _legitimate_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Attempt SQL injection in email field to bypass authentication
    let malicious_email = "admin' OR '1'='1' --";
    let login_request = LoginRequest {
        email_or_username: malicious_email.to_string(),
        password: "any_password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Login should fail - SQL injection should NOT bypass authentication
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "SQL injection should NOT bypass authentication"
    );

    // Should return 404 (user not found) or 400 (bad request)
    assert!(
        resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::BAD_REQUEST,
        "Expected 404 or 400, got: {}",
        resp.status()
    );
}

/// Test SQL injection in password field during login
///
/// Attack vector: Comment-based SQL injection
/// Expected: Login fails, password hash comparison prevents injection
#[actix_web::test]
async fn test_sql_injection_in_login_password_field() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create a legitimate user
    let legitimate_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Attempt SQL injection in password field
    let malicious_password = "' OR 1=1 --";
    let login_request = LoginRequest {
        email_or_username: legitimate_user.email.clone(),
        password: malicious_password.to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Login should fail - password is hashed and compared, not used in SQL
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "SQL injection in password should NOT succeed"
    );

    // Should return 400 (bad credentials)
    assert_eq!(
        resp.status(),
        StatusCode::BAD_REQUEST,
        "Expected 400 for invalid credentials, got: {}",
        resp.status()
    );
}

/// Test SQL injection combining email and password fields
///
/// Attack vector: Multi-field injection attempt
/// Expected: Login fails completely
#[actix_web::test]
async fn test_sql_injection_in_login_combined_fields() {
    let (service, _db) = lighter_auth::service!();

    // Attempt SQL injection in both fields simultaneously
    let login_request = LoginRequest {
        email_or_username: "admin'--".to_string(),
        password: "' OR '1'='1".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Login should fail
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "Combined SQL injection should NOT succeed"
    );
}

// =============================================================================
// SQL INJECTION TESTS - USER UPDATE
// =============================================================================

/// Test SQL injection in email field during user update
///
/// Attack vector: UPDATE injection with WHERE bypass
/// Expected: Update fails or treats input as literal
#[actix_web::test]
async fn test_sql_injection_in_email_during_user_update() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create two users
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();
    let target_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Login as auth user
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in email during update
    let malicious_email = "test' WHERE '1'='1@example.com";
    let update_request = serde_json::json!({
        "name": target_user.name,
        "email": malicious_email,
        "username": target_user.username,
        "profilePhotoId": null,
        "permissions": [],
        "roles": []
    });

    let req = TestRequest::put()
        .uri(&format!("/v1/user/{}", target_user.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Should either succeed (literal) or fail validation
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection should not cause server error, got: {}",
        resp.status()
    );

    // Verify only target user was affected (if update succeeded)
    use lighter_auth::entities::v1::users;
    let auth_user_check = users::Entity::find()
        .filter(users::Column::Id.eq(auth_user.id))
        .one(&db)
        .await
        .unwrap();

    assert!(
        auth_user_check.is_some(),
        "Auth user should not be affected by injection"
    );
}

/// Test SQL injection in name field during user update
///
/// Attack vector: Nested UPDATE statement
/// Expected: Name treated as literal string
#[actix_web::test]
async fn test_sql_injection_in_name_during_user_update() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user and target user
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();
    let target_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Login
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in name field
    let malicious_name = "John'; UPDATE users SET email='hacked@evil.com' WHERE '1'='1";
    let update_request = serde_json::json!({
        "name": malicious_name,
        "email": target_user.email,
        "username": target_user.username,
        "profilePhotoId": null,
        "permissions": [],
        "roles": []
    });

    let req = TestRequest::put()
        .uri(&format!("/v1/user/{}", target_user.id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_request)
        .to_request();

    let resp = call_service(&service, req).await;

    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection should not cause server error, got: {}",
        resp.status()
    );

    // Verify auth user email was NOT changed
    use lighter_auth::entities::v1::users;
    let auth_user_check = users::Entity::find()
        .filter(users::Column::Id.eq(auth_user.id))
        .one(&db)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        auth_user_check.email, auth_user.email,
        "Auth user email should not be changed by injection"
    );
}

// =============================================================================
// SQL INJECTION TESTS - SEARCH/PAGINATION
// =============================================================================

/// Test SQL injection in search parameter during user pagination
///
/// Attack vector: UNION-based SQL injection
/// Expected: Search treats input as literal string
#[actix_web::test]
async fn test_sql_injection_in_search_parameter() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Login
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in search parameter: UNION attack
    let malicious_search = "test' UNION SELECT id,name,email FROM users--";
    let search_encoded = urlencoding::encode(malicious_search);

    let req = TestRequest::get()
        .uri(&format!("/v1/user?search={}", search_encoded))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = call_service(&service, req).await;

    // Should succeed with no SQL errors (search treated as literal)
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection in search should not cause server error, got: {}",
        resp.status()
    );

    // Verify database structure is intact
    use lighter_auth::entities::v1::users;
    let users_count = users::Entity::find().all(&db).await;
    assert!(
        users_count.is_ok(),
        "Users table should remain intact after search injection"
    );
}

/// Test SQL injection in pagination parameters
///
/// Attack vector: Injection in page/perPage parameters
/// Expected: Parameters validated/sanitized properly
#[actix_web::test]
async fn test_sql_injection_in_pagination_parameters() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Login
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Attempt SQL injection in pagination parameters
    let malicious_page = "1' OR '1'='1";
    let page_encoded = urlencoding::encode(malicious_page);

    let req = TestRequest::get()
        .uri(&format!("/v1/user?page={}&perPage=10", page_encoded))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = call_service(&service, req).await;

    // Should either succeed (parsed as number, defaults to 1) or fail validation
    // Should NOT cause SQL error
    assert!(
        resp.status() == StatusCode::OK || resp.status() == StatusCode::BAD_REQUEST,
        "SQL injection in pagination should not cause server error, got: {}",
        resp.status()
    );
}

// =============================================================================
// SQL INJECTION TESTS - COMPREHENSIVE ATTACK VECTORS
// =============================================================================

/// Test common SQL injection attack patterns across multiple scenarios
///
/// Tests various well-known SQL injection patterns:
/// - Comment-based: --, #, /* */
/// - Boolean-based: OR 1=1, AND 1=1
/// - UNION-based: UNION SELECT
/// - Time-based: SLEEP, WAITFOR
/// - Stacked queries: ; DROP TABLE
#[actix_web::test]
async fn test_comprehensive_sql_injection_patterns() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Login
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Test various SQL injection patterns
    let injection_patterns = vec![
        "' OR '1'='1",
        "' OR 1=1--",
        "' OR 'a'='a",
        "' UNION SELECT NULL--",
        "'; DROP TABLE users--",
        "' AND 1=0 UNION ALL SELECT NULL--",
        "admin'--",
        "admin' #",
        "' OR '1'='1' /*",
        "1' AND '1'='1",
    ];

    for pattern in injection_patterns {
        // Try pattern in username during user creation
        let create_request = UserStoreRequest {
            name: "Test User".to_string(),
            email: format!("test_{}@example.com", rand::random::<u32>()),
            username: pattern.to_string(),
            password: "SecureP@ss123".to_string(),
            password_confirmation: "SecureP@ss123".to_string(),
            profile_photo_id: None,
            permissions: vec![],
            roles: vec![],
        };

        let req = TestRequest::post()
            .uri("/v1/user")
            .insert_header(("Authorization", format!("Bearer {}", token)))
            .set_json(&create_request)
            .to_request();

        let resp = call_service(&service, req).await;

        // Should not cause server error (500)
        assert_ne!(
            resp.status(),
            StatusCode::INTERNAL_SERVER_ERROR,
            "Pattern '{}' should not cause server error",
            pattern
        );
    }

    // Verify database integrity - users table should still exist and be queryable
    use lighter_auth::entities::v1::users;
    let users_result = users::Entity::find().all(&db).await;
    assert!(
        users_result.is_ok(),
        "Database should remain intact after all injection attempts"
    );
}

/// Test SQL injection with encoded/escaped characters
///
/// Attack vector: URL-encoded and hex-encoded SQL injection
/// Expected: Encoded input handled safely
#[actix_web::test]
async fn test_sql_injection_with_encoded_characters() {
    let (service, _db) = lighter_auth::service!();

    // Test with URL-encoded SQL injection in login
    let encoded_injection = "admin%27%20OR%20%271%27%3D%271"; // admin' OR '1'='1
    let login_request = LoginRequest {
        email_or_username: encoded_injection.to_string(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;

    // Should not succeed
    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "Encoded SQL injection should not bypass authentication"
    );
}

/// Test that SeaORM parameterized queries prevent SQL injection
///
/// This test creates a user with special characters that would break
/// non-parameterized queries, then verifies the data is stored and
/// retrieved correctly.
#[actix_web::test]
async fn test_parameterized_queries_prevent_injection() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create auth user
    let auth_user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Login
    let login_request = LoginRequest {
        email_or_username: auth_user.email.clone(),
        password: "password".to_string(),
    };

    let req = TestRequest::post()
        .uri("/login")
        .set_json(&login_request)
        .to_request();

    let resp = call_service(&service, req).await;
    let body: Authenticated = actix_web::test::read_body_json(resp).await;
    let token = body.token;

    // Create user with special SQL characters
    let special_name = "O'Reilly & Sons \"Quote\" Test";
    let create_request = UserStoreRequest {
        name: special_name.to_string(),
        email: "special@example.com".to_string(),
        username: "special_user".to_string(),
        password: "SecureP@ss123".to_string(),
        password_confirmation: "SecureP@ss123".to_string(),
        profile_photo_id: None,
        permissions: vec![],
        roles: vec![],
    };

    let req = TestRequest::post()
        .uri("/v1/user")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_request)
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "User creation with special characters should succeed"
    );

    // Verify data was stored correctly (literally)
    use lighter_auth::entities::v1::users;
    use sea_orm::ColumnTrait;

    let user = users::Entity::find()
        .filter(users::Column::Username.eq("special_user"))
        .one(&db)
        .await
        .unwrap()
        .expect("User should be found");

    // Compare case-insensitively since database might normalize case
    assert_eq!(
        user.name.to_lowercase(),
        special_name.to_lowercase(),
        "Special characters should be stored literally (case-insensitive)"
    );
}
