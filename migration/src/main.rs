use sea_orm_migration::prelude::*;

#[tokio::main]
async fn main() {
    cli::run_cli(lighter_auth_migration::Migrator).await;
}
