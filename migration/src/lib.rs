pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_extensions;
mod m20230902_024725_v1_create_users;
mod m20230902_024928_v1_create_permissions;
mod m20230902_025106_v1_create_roles;
mod m20230902_025217_v1_create_permission_user;
mod m20230902_025247_v1_create_permission_role;
mod m20230902_025255_v1_create_role_user;
mod m20230902_025309_v1_create_tokens;
mod m20231216_092530_v1_user_initial_seeder;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_extensions::Migration),
            Box::new(m20230902_024725_v1_create_users::Migration),
            Box::new(m20230902_024928_v1_create_permissions::Migration),
            Box::new(m20230902_025106_v1_create_roles::Migration),
            Box::new(m20230902_025217_v1_create_permission_user::Migration),
            Box::new(m20230902_025247_v1_create_permission_role::Migration),
            Box::new(m20230902_025255_v1_create_role_user::Migration),
            Box::new(m20230902_025309_v1_create_tokens::Migration),
            Box::new(m20231216_092530_v1_user_initial_seeder::Migration),
        ]
    }
}
