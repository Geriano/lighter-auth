use lighter_common::prelude::*;
use sea_orm_migration::prelude::*;

use crate::{
    m20230902_024725_v1_create_users::{User, TABLE as USER_TABLE},
    m20230902_024928_v1_create_permissions::{Permission, TABLE as PERMISSION_TABLE},
    m20230902_025106_v1_create_roles::{Role, TABLE as ROLE_TABLE},
    m20230902_025247_v1_create_permission_role::{PermissionRole, TABLE as PERMISSION_ROLE_TABLE},
    m20230902_025255_v1_create_role_user::{RoleUser, TABLE as ROLE_USER_TABLE},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

const PERMISSIONS: [&str; 4] = ["user", "permission", "role", "home"];
const ABILITIES: [&str; 5] = ["manage", "create", "read", "update", "delete"];
const ROLES: [&str; 2] = ["superuser", "admin"];

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let user_id = Uuid::from_u128(0);

        manager
            .exec_stmt(
                Query::insert()
                    .into_table(USER_TABLE)
                    .columns(vec![
                        User::Id,
                        User::Name,
                        User::Email,
                        User::Username,
                        User::Password,
                    ])
                    .values_panic(vec![
                        user_id.into(),
                        "superuser".into(),
                        "root@local".into(),
                        "root".into(),
                        Hash::make(user_id, "password").to_string().into(),
                    ])
                    .to_owned(),
            )
            .await?;

        let mut permissions = vec![];
        let mut query = Query::insert()
            .into_table(PERMISSION_TABLE)
            .columns(vec![Permission::Id, Permission::Code, Permission::Name])
            .to_owned();

        for permission in PERMISSIONS {
            for ability in ABILITIES {
                let id = Uuid::new_v4();
                let name = format!("{} {}", ability, permission);
                let code = name.to_uppercase().replace(" ", "_");

                permissions.push(id);

                query = query
                    .values_panic(vec![id.into(), code.into(), name.into()])
                    .to_owned();
            }
        }

        manager.exec_stmt(query).await?;

        let mut roles = vec![];
        let mut query = Query::insert()
            .into_table(ROLE_TABLE)
            .columns(vec![Role::Id, Role::Code, Role::Name])
            .to_owned();

        for role in ROLES {
            let id = Uuid::new_v4();
            let code = role.to_uppercase();
            let name = role.to_lowercase();

            roles.push(id);

            query = query
                .values_panic(vec![id.into(), code.into(), name.into()])
                .to_owned();
        }

        manager.exec_stmt(query).await?;

        let mut role_user = Query::insert()
            .into_table(ROLE_USER_TABLE)
            .columns(vec![RoleUser::Id, RoleUser::UserId, RoleUser::RoleId])
            .to_owned();
        let mut permission_role = Query::insert()
            .into_table(PERMISSION_ROLE_TABLE)
            .columns(vec![
                PermissionRole::Id,
                PermissionRole::PermissionId,
                PermissionRole::RoleId,
            ])
            .to_owned();

        for &role in &roles {
            role_user = role_user
                .values_panic(vec![Uuid::new_v4().into(), user_id.into(), role.into()])
                .to_owned();

            for &permission in &permissions {
                permission_role = permission_role
                    .values_panic(vec![Uuid::new_v4().into(), permission.into(), role.into()])
                    .to_owned();
            }
        }

        manager.exec_stmt(role_user).await?;
        manager.exec_stmt(permission_role).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .exec_stmt(
                Query::delete()
                    .from_table(USER_TABLE)
                    .and_where(Expr::col(User::Username).eq("root"))
                    .to_owned(),
            )
            .await?;

        for permission in PERMISSIONS {
            for ability in ABILITIES {
                let permission = format!("{} {}", ability, permission);
                let code = permission.to_uppercase().replace(" ", "_");

                manager
                    .exec_stmt(
                        Query::delete()
                            .from_table(PERMISSION_TABLE)
                            .and_where(Expr::col(Permission::Code).eq(code))
                            .to_owned(),
                    )
                    .await?;
            }
        }

        for role in ROLES {
            let code = role.to_uppercase();

            manager
                .exec_stmt(
                    Query::delete()
                        .from_table(ROLE_TABLE)
                        .and_where(Expr::col(Role::Code).eq(code))
                        .to_owned(),
                )
                .await?;
        }

        Ok(())
    }
}
