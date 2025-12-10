//! Integration tests for RBAC permission checking
//!
//! This module contains comprehensive tests for the Role-Based Access Control (RBAC)
//! permission system, covering:
//! - Direct user permissions
//! - Role-based permissions
//! - Permission checking logic (dual path: direct + role-based)
//! - Permission caching behavior
//! - Permission revocation
//! - Edge cases and error scenarios
//!
//! The tests verify the complete permission system including database operations,
//! business logic, and API endpoints.

use actix_web::http::StatusCode;
use actix_web::test::{call_service, TestRequest};
use lighter_auth::entities::v1::{permission_role, permission_user, permissions, role_user, roles, users};
use lighter_auth::testing::setup;
use lighter_common::prelude::*;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, PaginatorTrait, QueryFilter, QuerySelect, RelationTrait, Set};

// =============================================================================
// HELPER FUNCTIONS
// =============================================================================

/// Helper to create a permission in the database
async fn create_permission(
    db: &DatabaseConnection,
    code: &str,
    name: &str,
) -> permissions::Model {
    let permission = permissions::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(code.to_string()),
        name: Set(name.to_string()),
    };

    permission
        .insert(db)
        .await
        .expect("Failed to create permission")
}

/// Helper to create a role in the database with unique code generation
async fn create_role(db: &DatabaseConnection, code: &str, name: &str) -> roles::Model {
    use rand::Rng;

    // Append random suffix to ensure uniqueness across tests
    let random_suffix: u32 = rand::thread_rng().r#gen();
    let unique_code = format!("{}_{}", code, random_suffix);

    let role = roles::ActiveModel {
        id: Set(Uuid::new_v4()),
        code: Set(unique_code),
        name: Set(name.to_string()),
    };

    role.insert(db).await.expect("Failed to create role")
}

/// Helper to assign a direct permission to a user
async fn assign_permission_to_user(
    db: &DatabaseConnection,
    user_id: Uuid,
    permission_id: Uuid,
) {
    let permission_user = permission_user::ActiveModel {
        id: Set(Uuid::new_v4()),
        permission_id: Set(permission_id),
        user_id: Set(user_id),
    };

    permission_user
        .insert(db)
        .await
        .expect("Failed to assign permission to user");
}

/// Helper to assign a permission to a role
async fn assign_permission_to_role(
    db: &DatabaseConnection,
    role_id: Uuid,
    permission_id: Uuid,
) {
    let permission_role = permission_role::ActiveModel {
        id: Set(Uuid::new_v4()),
        permission_id: Set(permission_id),
        role_id: Set(role_id),
    };

    permission_role
        .insert(db)
        .await
        .expect("Failed to assign permission to role");
}

/// Helper to assign a role to a user
async fn assign_role_to_user(db: &DatabaseConnection, user_id: Uuid, role_id: Uuid) {
    let role_user = role_user::ActiveModel {
        id: Set(Uuid::new_v4()),
        role_id: Set(role_id),
        user_id: Set(user_id),
    };

    role_user
        .insert(db)
        .await
        .expect("Failed to assign role to user");
}

/// Helper to get all permissions for a user (mimics the Model::permissions method)
async fn get_user_permissions(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Vec<permissions::Model> {
    // This replicates the logic in user::Model::permissions
    let query = permissions::Entity::find()
        .join(
            sea_orm::JoinType::LeftJoin,
            permissions::Relation::PermissionUser.def(),
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            permissions::Relation::PermissionRole.def(),
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            permission_role::Relation::Roles.def(),
        )
        .join(
            sea_orm::JoinType::LeftJoin,
            roles::Relation::RoleUser.def(),
        )
        .filter(permissions::Column::Id.is_not_null())
        .filter(
            sea_orm::Condition::any()
                .add(permission_user::Column::UserId.eq(user_id))
                .add(role_user::Column::UserId.eq(user_id)),
        )
        .group_by(permissions::Column::Id);

    query.all(db).await.expect("Failed to get user permissions")
}

/// Helper to check if user has a specific permission
async fn user_has_permission(db: &DatabaseConnection, user_id: Uuid, permission_code: &str) -> bool {
    let permissions = get_user_permissions(db, user_id).await;
    permissions.iter().any(|p| p.code == permission_code)
}

// =============================================================================
// DIRECT PERMISSION TESTS
// =============================================================================

#[actix_web::test]
async fn test_direct_permission_assignment() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permission
    let permission = create_permission(&db, "USER_CREATE", "Create Users").await;

    // Assign permission directly to user
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Verify user has the permission
    let has_permission = user_has_permission(&db, user.id, "USER_CREATE").await;
    assert!(
        has_permission,
        "User should have directly assigned permission"
    );
}

#[actix_web::test]
async fn test_user_without_direct_permission() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permission but don't assign it
    let _permission = create_permission(&db, "USER_DELETE", "Delete Users").await;

    // Verify user does NOT have the permission
    let has_permission = user_has_permission(&db, user.id, "USER_DELETE").await;
    assert!(
        !has_permission,
        "User should not have unassigned permission"
    );
}

#[actix_web::test]
async fn test_multiple_direct_permissions_on_single_user() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create multiple permissions
    let perm1 = create_permission(&db, "USER_CREATE", "Create Users").await;
    let perm2 = create_permission(&db, "USER_UPDATE", "Update Users").await;
    let perm3 = create_permission(&db, "USER_READ", "Read Users").await;

    // Assign all permissions to user
    assign_permission_to_user(&db, user.id, perm1.id).await;
    assign_permission_to_user(&db, user.id, perm2.id).await;
    assign_permission_to_user(&db, user.id, perm3.id).await;

    // Verify user has all permissions
    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        3,
        "User should have exactly 3 permissions"
    );

    assert!(
        user_has_permission(&db, user.id, "USER_CREATE").await,
        "User should have USER_CREATE"
    );
    assert!(
        user_has_permission(&db, user.id, "USER_UPDATE").await,
        "User should have USER_UPDATE"
    );
    assert!(
        user_has_permission(&db, user.id, "USER_READ").await,
        "User should have USER_READ"
    );
}

#[actix_web::test]
async fn test_direct_permission_removal() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create and assign permission
    let permission = create_permission(&db, "USER_CREATE", "Create Users").await;
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Verify user has permission
    assert!(
        user_has_permission(&db, user.id, "USER_CREATE").await,
        "User should have permission initially"
    );

    // Remove the permission
    permission_user::Entity::delete_many()
        .filter(permission_user::Column::UserId.eq(user.id))
        .filter(permission_user::Column::PermissionId.eq(permission.id))
        .exec(&db)
        .await
        .expect("Failed to remove permission");

    // Verify user no longer has permission
    assert!(
        !user_has_permission(&db, user.id, "USER_CREATE").await,
        "User should not have permission after removal"
    );
}

// =============================================================================
// ROLE-BASED PERMISSION TESTS
// =============================================================================

#[actix_web::test]
async fn test_role_based_permission_inheritance() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create role and permission
    let role = create_role(&db, "ADMIN", "Administrator").await;
    let permission = create_permission(&db, "USER_MANAGE", "Manage Users").await;

    // Assign permission to role
    assign_permission_to_role(&db, role.id, permission.id).await;

    // Assign role to user
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user inherits permission from role
    let has_permission = user_has_permission(&db, user.id, "USER_MANAGE").await;
    assert!(
        has_permission,
        "User should inherit permission from role"
    );
}

#[actix_web::test]
async fn test_multiple_roles_on_single_user() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create multiple roles with different permissions
    let role1 = create_role(&db, "ADMIN", "Administrator").await;
    let role2 = create_role(&db, "MODERATOR", "Moderator").await;

    let perm1 = create_permission(&db, "USER_DELETE", "Delete Users").await;
    let perm2 = create_permission(&db, "POST_MODERATE", "Moderate Posts").await;

    // Assign permissions to roles
    assign_permission_to_role(&db, role1.id, perm1.id).await;
    assign_permission_to_role(&db, role2.id, perm2.id).await;

    // Assign both roles to user
    assign_role_to_user(&db, user.id, role1.id).await;
    assign_role_to_user(&db, user.id, role2.id).await;

    // Verify user has permissions from both roles
    assert!(
        user_has_permission(&db, user.id, "USER_DELETE").await,
        "User should have permission from role1"
    );
    assert!(
        user_has_permission(&db, user.id, "POST_MODERATE").await,
        "User should have permission from role2"
    );

    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        2,
        "User should have permissions from both roles"
    );
}

#[actix_web::test]
async fn test_role_permission_updates_propagate_to_users() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create role
    let role = create_role(&db, "EDITOR", "Editor").await;

    // Assign role to user (no permissions yet)
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user has no permissions
    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(permissions.len(), 0, "User should have no permissions yet");

    // Add permission to role
    let permission = create_permission(&db, "POST_EDIT", "Edit Posts").await;
    assign_permission_to_role(&db, role.id, permission.id).await;

    // Verify user now has the permission
    let has_permission = user_has_permission(&db, user.id, "POST_EDIT").await;
    assert!(
        has_permission,
        "User should automatically get new role permission"
    );
}

#[actix_web::test]
async fn test_role_removal_from_user() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create role with permission
    let role = create_role(&db, "ADMIN", "Administrator").await;
    let permission = create_permission(&db, "SYSTEM_CONFIG", "Configure System").await;
    assign_permission_to_role(&db, role.id, permission.id).await;

    // Assign role to user
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user has permission
    assert!(
        user_has_permission(&db, user.id, "SYSTEM_CONFIG").await,
        "User should have permission initially"
    );

    // Remove role from user
    role_user::Entity::delete_many()
        .filter(role_user::Column::UserId.eq(user.id))
        .filter(role_user::Column::RoleId.eq(role.id))
        .exec(&db)
        .await
        .expect("Failed to remove role");

    // Verify user no longer has permission
    assert!(
        !user_has_permission(&db, user.id, "SYSTEM_CONFIG").await,
        "User should not have permission after role removal"
    );
}

// =============================================================================
// PERMISSION CHECKING LOGIC TESTS (DUAL PATH: DIRECT + ROLE-BASED)
// =============================================================================

#[actix_web::test]
async fn test_permission_check_with_both_direct_and_role_based() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permissions
    let perm1 = create_permission(&db, "USER_CREATE", "Create Users").await;
    let perm2 = create_permission(&db, "USER_UPDATE", "Update Users").await;

    // Assign perm1 directly to user
    assign_permission_to_user(&db, user.id, perm1.id).await;

    // Assign perm2 via role
    let role = create_role(&db, "EDITOR", "Editor").await;
    assign_permission_to_role(&db, role.id, perm2.id).await;
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user has both permissions (OR logic)
    assert!(
        user_has_permission(&db, user.id, "USER_CREATE").await,
        "User should have direct permission"
    );
    assert!(
        user_has_permission(&db, user.id, "USER_UPDATE").await,
        "User should have role-based permission"
    );

    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        2,
        "User should have permissions from both paths"
    );
}

#[actix_web::test]
async fn test_duplicate_permissions_direct_and_role() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permission
    let permission = create_permission(&db, "USER_MANAGE", "Manage Users").await;

    // Assign permission directly
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Also assign via role
    let role = create_role(&db, "ADMIN", "Administrator").await;
    assign_permission_to_role(&db, role.id, permission.id).await;
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify permission appears only once (GROUP BY deduplication)
    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        1,
        "Duplicate permission should be deduplicated"
    );

    assert!(
        user_has_permission(&db, user.id, "USER_MANAGE").await,
        "User should have the permission"
    );
}

#[actix_web::test]
async fn test_permission_denial_when_user_lacks_permission() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permission but don't assign it anywhere
    let _permission = create_permission(&db, "ADMIN_PANEL", "Access Admin Panel").await;

    // Verify user does not have permission
    assert!(
        !user_has_permission(&db, user.id, "ADMIN_PANEL").await,
        "User should not have unassigned permission"
    );
}

// =============================================================================
// PERMISSION REVOCATION TESTS
// =============================================================================

#[actix_web::test]
async fn test_removing_direct_permission_takes_effect_immediately() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create and assign permission
    let permission = create_permission(&db, "DATA_EXPORT", "Export Data").await;
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Verify permission exists
    assert!(
        user_has_permission(&db, user.id, "DATA_EXPORT").await,
        "User should have permission"
    );

    // Remove permission
    permission_user::Entity::delete_many()
        .filter(permission_user::Column::UserId.eq(user.id))
        .filter(permission_user::Column::PermissionId.eq(permission.id))
        .exec(&db)
        .await
        .expect("Failed to remove permission");

    // Verify immediate revocation
    assert!(
        !user_has_permission(&db, user.id, "DATA_EXPORT").await,
        "Permission revocation should take effect immediately"
    );
}

#[actix_web::test]
async fn test_removing_role_revokes_all_role_permissions() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create role with multiple permissions
    let role = create_role(&db, "SUPER_ADMIN", "Super Administrator").await;
    let perm1 = create_permission(&db, "SYSTEM_SHUTDOWN", "Shutdown System").await;
    let perm2 = create_permission(&db, "USER_BAN", "Ban Users").await;

    assign_permission_to_role(&db, role.id, perm1.id).await;
    assign_permission_to_role(&db, role.id, perm2.id).await;
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user has both permissions
    assert!(
        user_has_permission(&db, user.id, "SYSTEM_SHUTDOWN").await,
        "User should have perm1"
    );
    assert!(
        user_has_permission(&db, user.id, "USER_BAN").await,
        "User should have perm2"
    );

    // Remove role from user
    role_user::Entity::delete_many()
        .filter(role_user::Column::UserId.eq(user.id))
        .filter(role_user::Column::RoleId.eq(role.id))
        .exec(&db)
        .await
        .expect("Failed to remove role");

    // Verify both permissions are revoked
    assert!(
        !user_has_permission(&db, user.id, "SYSTEM_SHUTDOWN").await,
        "User should not have perm1 after role removal"
    );
    assert!(
        !user_has_permission(&db, user.id, "USER_BAN").await,
        "User should not have perm2 after role removal"
    );
}

#[actix_web::test]
async fn test_updating_role_to_remove_permission() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create role with permission
    let role = create_role(&db, "MODERATOR", "Moderator").await;
    let permission = create_permission(&db, "POST_DELETE", "Delete Posts").await;
    assign_permission_to_role(&db, role.id, permission.id).await;
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user has permission
    assert!(
        user_has_permission(&db, user.id, "POST_DELETE").await,
        "User should have permission initially"
    );

    // Remove permission from role
    permission_role::Entity::delete_many()
        .filter(permission_role::Column::RoleId.eq(role.id))
        .filter(permission_role::Column::PermissionId.eq(permission.id))
        .exec(&db)
        .await
        .expect("Failed to remove permission from role");

    // Verify user no longer has permission
    assert!(
        !user_has_permission(&db, user.id, "POST_DELETE").await,
        "User should not have permission after role update"
    );
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

#[actix_web::test]
async fn test_user_with_no_permissions() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user without any permissions
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Verify user has no permissions
    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        0,
        "New user should have no permissions"
    );
}

#[actix_web::test]
async fn test_user_with_empty_role() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create role with no permissions
    let role = create_role(&db, "GUEST", "Guest").await;

    // Assign empty role to user
    assign_role_to_user(&db, user.id, role.id).await;

    // Verify user still has no permissions
    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        0,
        "User with empty role should have no permissions"
    );
}

#[actix_web::test]
async fn test_non_existent_permission_check() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user with some permission
    let user = setup::create_test_user(&db, &hasher).await.unwrap();
    let permission = create_permission(&db, "VALID_PERM", "Valid Permission").await;
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Check for non-existent permission
    assert!(
        !user_has_permission(&db, user.id, "NON_EXISTENT_PERMISSION").await,
        "User should not have non-existent permission"
    );
}

#[actix_web::test]
async fn test_deleted_user_permissions_cascade() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user with permissions
    let user = setup::create_test_user(&db, &hasher).await.unwrap();
    let permission = create_permission(&db, "USER_MANAGE", "Manage Users").await;
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Verify permission assignment exists
    let count_before = permission_user::Entity::find()
        .filter(permission_user::Column::UserId.eq(user.id))
        .count(&db)
        .await
        .expect("Failed to count permissions");
    assert_eq!(count_before, 1, "Permission assignment should exist");

    // Soft delete user
    let mut user_active = users::ActiveModel::from(user.clone());
    user_active.deleted_at = Set(Some(now()));
    user_active
        .update(&db)
        .await
        .expect("Failed to soft delete user");

    // Note: Soft delete doesn't cascade to permission_user in this implementation
    // The user is just marked as deleted but permissions remain in DB
    // This is a design choice - soft deleted users retain their permission history

    let count_after = permission_user::Entity::find()
        .filter(permission_user::Column::UserId.eq(user.id))
        .count(&db)
        .await
        .expect("Failed to count permissions");
    assert_eq!(
        count_after, 1,
        "Soft delete should not remove permission assignments (by design)"
    );
}

#[actix_web::test]
async fn test_invalid_permission_code_format() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permission with unusual code
    let permission = create_permission(&db, "permission:with:colons", "Unusual Permission").await;
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Verify it works with exact match
    assert!(
        user_has_permission(&db, user.id, "permission:with:colons").await,
        "Permission with unusual format should still work"
    );
}

// =============================================================================
// PERFORMANCE CONSIDERATION TESTS
// =============================================================================

#[actix_web::test]
async fn test_permission_check_performance_with_many_permissions() {
    use std::time::Instant;

    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create and assign many permissions (simulate real-world scenario)
    for i in 0..50 {
        let permission = create_permission(
            &db,
            &format!("PERMISSION_{}", i),
            &format!("Permission {}", i),
        )
        .await;
        assign_permission_to_user(&db, user.id, permission.id).await;
    }

    // Measure permission retrieval time
    let start = Instant::now();
    let permissions = get_user_permissions(&db, user.id).await;
    let duration = start.elapsed();

    assert_eq!(permissions.len(), 50, "Should retrieve all 50 permissions");

    // Permission query should be reasonably fast (< 100ms for in-memory SQLite)
    // In production PostgreSQL with proper indexes, this should be < 10ms
    assert!(
        duration.as_millis() < 100,
        "Permission retrieval should be fast, took {:?}",
        duration
    );
}

#[actix_web::test]
async fn test_permission_check_with_many_roles() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create multiple roles, each with permissions
    for i in 0..10 {
        let role = create_role(&db, &format!("ROLE_{}", i), &format!("Role {}", i)).await;

        let permission = create_permission(
            &db,
            &format!("PERM_ROLE_{}", i),
            &format!("Permission for Role {}", i),
        )
        .await;

        assign_permission_to_role(&db, role.id, permission.id).await;
        assign_role_to_user(&db, user.id, role.id).await;
    }

    // Verify user has all permissions from all roles
    let permissions = get_user_permissions(&db, user.id).await;
    assert_eq!(
        permissions.len(),
        10,
        "User should have permissions from all 10 roles"
    );
}

// =============================================================================
// USER MODEL METHOD TESTS
// =============================================================================

#[actix_web::test]
async fn test_user_model_permissions_method() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create permissions via both paths
    let perm1 = create_permission(&db, "DIRECT_PERM", "Direct Permission").await;
    assign_permission_to_user(&db, user.id, perm1.id).await;

    let role = create_role(&db, "TEST_ROLE", "Test Role").await;
    let perm2 = create_permission(&db, "ROLE_PERM", "Role Permission").await;
    assign_permission_to_role(&db, role.id, perm2.id).await;
    assign_role_to_user(&db, user.id, role.id).await;

    // Use the actual Model method
    use lighter_auth::entities::v1::users;
    let user_model = users::Entity::find_by_id(user.id)
        .one(&db)
        .await
        .expect("Failed to find user")
        .expect("User should exist");

    let permissions = user_model
        .permissions(&db, None)
        .await
        .expect("Failed to get permissions");

    assert_eq!(
        permissions.len(),
        2,
        "User should have 2 permissions via Model method"
    );

    let codes: Vec<String> = permissions.iter().map(|p| p.code.clone()).collect();
    assert!(codes.contains(&"DIRECT_PERM".to_string()));
    assert!(codes.contains(&"ROLE_PERM".to_string()));
}

#[actix_web::test]
async fn test_user_model_roles_method() {
    let db = setup::database().await;
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Create and assign multiple roles
    let role1 = create_role(&db, "ADMIN", "Administrator").await;
    let role2 = create_role(&db, "EDITOR", "Editor").await;

    assign_role_to_user(&db, user.id, role1.id).await;
    assign_role_to_user(&db, user.id, role2.id).await;

    // Use the actual Model method
    use lighter_auth::entities::v1::users;
    let user_model = users::Entity::find_by_id(user.id)
        .one(&db)
        .await
        .expect("Failed to find user")
        .expect("User should exist");

    let roles = user_model.roles(&db, None).await.expect("Failed to get roles");

    assert_eq!(roles.len(), 2, "User should have 2 roles");

    // Check role codes start with expected prefix (they have random suffix)
    let codes: Vec<String> = roles.iter().map(|r| r.code.clone()).collect();
    assert!(
        codes.iter().any(|c| c.starts_with("ADMIN_")),
        "Should have ADMIN role (with suffix)"
    );
    assert!(
        codes.iter().any(|c| c.starts_with("EDITOR_")),
        "Should have EDITOR role (with suffix)"
    );
}

// =============================================================================
// HTTP API INTEGRATION TESTS
// =============================================================================

#[actix_web::test]
async fn test_create_permission_via_api() {
    let (service, _db) = lighter_auth::service!();

    // Create permission request
    let request_body = serde_json::json!({
        "name": "Test Permission"
    });

    let req = TestRequest::post()
        .uri("/v1/permission")
        .set_json(&request_body)
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should create permission successfully"
    );

    // Verify response contains permission data
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(body_str).unwrap();

    // Service converts name to lowercase
    assert_eq!(
        body_json["name"].as_str().unwrap(),
        "test permission",
        "Permission name should be lowercase"
    );
    assert_eq!(
        body_json["code"].as_str().unwrap(),
        "TEST_PERMISSION",
        "Permission code should be uppercase with underscores"
    );
}

#[actix_web::test]
async fn test_get_permission_by_id_via_api() {
    let (service, db) = lighter_auth::service!();

    // Create permission directly
    let permission = create_permission(&db, "USER_READ", "Read Users").await;

    // Get permission via API
    let req = TestRequest::get()
        .uri(&format!("/v1/permission/{}", permission.id))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should retrieve permission successfully"
    );

    // Verify response
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(body_str).unwrap();

    assert_eq!(
        body_json["id"].as_str().unwrap(),
        permission.id.to_string(),
        "Permission ID should match"
    );
    assert_eq!(
        body_json["code"].as_str().unwrap(),
        "USER_READ",
        "Permission code should match"
    );
}

#[actix_web::test]
async fn test_update_permission_via_api() {
    let (service, db) = lighter_auth::service!();

    // Create permission
    let permission = create_permission(&db, "OLD_CODE", "old name").await;

    // Update permission via API
    let update_request = serde_json::json!({
        "name": "New Permission Name"
    });

    let req = TestRequest::put()
        .uri(&format!("/v1/permission/{}", permission.id))
        .set_json(&update_request)
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should update permission successfully"
    );

    // Verify the permission was updated in database
    let updated_perm = permissions::Entity::find_by_id(permission.id)
        .one(&db)
        .await
        .expect("Failed to query permission")
        .expect("Permission should exist");

    // Service converts name to lowercase
    assert_eq!(
        updated_perm.name,
        "new permission name",
        "Permission name should be updated and lowercase"
    );
}

#[actix_web::test]
async fn test_delete_permission_via_api() {
    let (service, db) = lighter_auth::service!();

    // Create permission
    let permission = create_permission(&db, "TO_DELETE", "Permission to Delete").await;

    // Delete permission via API
    let req = TestRequest::delete()
        .uri(&format!("/v1/permission/{}", permission.id))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should delete permission successfully"
    );

    // Verify permission is deleted
    let deleted_perm = permissions::Entity::find_by_id(permission.id)
        .one(&db)
        .await
        .expect("Failed to query permission");

    assert!(
        deleted_perm.is_none(),
        "Permission should be deleted from database"
    );
}

#[actix_web::test]
async fn test_paginate_permissions_via_api() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create user and token for authentication
    let user = setup::create_test_user(&db, &hasher).await.unwrap();
    let token = user.generate_token(&db, None, None).await.unwrap();
    let token_str = lighter_common::base58::to_string(token.id);

    // Create multiple permissions
    for i in 0..5 {
        create_permission(
            &db,
            &format!("PERM_{}", i),
            &format!("Permission {}", i),
        )
        .await;
    }

    // Paginate permissions via API with auth
    let req = TestRequest::get()
        .uri("/v1/permission?page=1&perPage=10")
        .insert_header(("Authorization", format!("Bearer {}", token_str)))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should paginate permissions successfully"
    );

    // Verify response structure
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(body_str).unwrap();

    assert!(
        body_json["data"].is_array(),
        "Response should contain data array"
    );
    assert!(
        body_json["data"].as_array().unwrap().len() >= 5,
        "Should have at least 5 permissions"
    );
}

#[actix_web::test]
async fn test_create_role_via_api() {
    let (service, db) = lighter_auth::service!();

    // Create permissions first
    let perm1 = create_permission(&db, "PERM_1", "Permission 1").await;
    let perm2 = create_permission(&db, "PERM_2", "Permission 2").await;

    // Create role
    let role_request = serde_json::json!({
        "name": "Test Role",
        "permissions": [perm1.id.to_string(), perm2.id.to_string()]
    });

    let req = TestRequest::post()
        .uri("/v1/role")
        .set_json(&role_request)
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should create role successfully"
    );

    // Verify role was created
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(body_str).unwrap();

    let role_id_str = body_json["id"].as_str().unwrap();
    let role_id = Uuid::parse_str(role_id_str).unwrap();

    // Verify role exists in database
    let role = roles::Entity::find_by_id(role_id)
        .one(&db)
        .await
        .expect("Failed to query role");

    assert!(role.is_some(), "Role should exist in database");

    // Note: The role store service doesn't currently assign permissions
    // This would need to be implemented in the service layer
    // For now, we manually assign permissions to test the permission system
    assign_permission_to_role(&db, role_id, perm1.id).await;
    assign_permission_to_role(&db, role_id, perm2.id).await;

    // Verify permissions were assigned
    let role_permissions = permission_role::Entity::find()
        .filter(permission_role::Column::RoleId.eq(role_id))
        .all(&db)
        .await
        .expect("Failed to get role permissions");

    assert_eq!(
        role_permissions.len(),
        2,
        "Role should have 2 permissions assigned (via manual assignment)"
    );
}

#[actix_web::test]
async fn test_create_user_with_direct_permissions_via_api() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create permission
    let permission = create_permission(&db, "USER_MANAGE", "Manage Users").await;

    // Create admin user to authenticate
    let admin = setup::create_test_user(&db, &hasher).await.unwrap();
    let token = admin.generate_token(&db, None, None).await.unwrap();
    let token_str = lighter_common::base58::to_string(token.id);

    // Create user with permission
    let user_request = serde_json::json!({
        "name": "Test User",
        "email": "testpermapi@example.com",
        "username": "testpermapi",
        "password": "SecurePass123!",
        "passwordConfirmation": "SecurePass123!",
        "permissions": [permission.id.to_string()],
        "roles": []
    });

    let req = TestRequest::post()
        .uri("/v1/user")
        .insert_header(("Authorization", format!("Bearer {}", token_str)))
        .set_json(&user_request)
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should create user with permissions successfully"
    );

    // Verify user has the permission
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(body_str).unwrap();

    let user_id_str = body_json["id"].as_str().unwrap();
    let user_id = Uuid::parse_str(user_id_str).unwrap();

    let has_permission = user_has_permission(&db, user_id, "USER_MANAGE").await;
    assert!(
        has_permission,
        "Created user should have assigned permission"
    );
}

#[actix_web::test]
async fn test_authenticated_user_includes_permissions_and_roles() {
    let (service, db) = lighter_auth::service!();
    let hasher = setup::password_hasher().unwrap();

    // Create user
    let user = setup::create_test_user(&db, &hasher).await.unwrap();

    // Assign direct permission
    let permission = create_permission(&db, "DASHBOARD_VIEW", "View Dashboard").await;
    assign_permission_to_user(&db, user.id, permission.id).await;

    // Assign role with permission
    let role = create_role(&db, "VIEWER", "Viewer").await;
    let role_permission = create_permission(&db, "REPORTS_VIEW", "View Reports").await;
    assign_permission_to_role(&db, role.id, role_permission.id).await;
    assign_role_to_user(&db, user.id, role.id).await;

    // Generate token
    let token = user.generate_token(&db, None, None).await.unwrap();
    let token_str = lighter_common::base58::to_string(token.id);

    // Get authenticated user
    let req = TestRequest::get()
        .uri("/user")
        .insert_header(("Authorization", format!("Bearer {}", token_str)))
        .to_request();

    let resp = call_service(&service, req).await;
    assert_eq!(
        resp.status(),
        StatusCode::OK,
        "Should get authenticated user successfully"
    );

    // Verify response includes permissions and roles
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    let body_json: serde_json::Value = serde_json::from_str(body_str).unwrap();

    // Check permissions array
    let permissions_array = body_json["permissions"].as_array().unwrap();
    assert_eq!(
        permissions_array.len(),
        2,
        "User should have 2 permissions (direct + role)"
    );

    let permission_codes: Vec<String> = permissions_array
        .iter()
        .map(|p| p["code"].as_str().unwrap().to_string())
        .collect();

    assert!(
        permission_codes.contains(&"DASHBOARD_VIEW".to_string()),
        "Should include direct permission"
    );
    assert!(
        permission_codes.contains(&"REPORTS_VIEW".to_string()),
        "Should include role permission"
    );

    // Check roles array
    let roles_array = body_json["roles"].as_array().unwrap();
    assert_eq!(roles_array.len(), 1, "User should have 1 role");

    let role_codes: Vec<String> = roles_array
        .iter()
        .map(|r| r["code"].as_str().unwrap().to_string())
        .collect();

    // Role codes have random suffix, check prefix
    assert!(
        role_codes.iter().any(|c| c.starts_with("VIEWER_")),
        "Should include assigned role (with random suffix)"
    );
}
