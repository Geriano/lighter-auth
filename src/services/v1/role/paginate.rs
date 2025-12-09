use anyhow::Context;
use lighter_common::prelude::*;
use sea_orm::prelude::*;
use sea_orm::{ColumnTrait, QueryOrder, QuerySelect};

use crate::entities::v1::roles::{Column, Entity};
use crate::responses::v1::role::{
    RolePaginationOrder, RolePaginationRequest, RolePaginationResponse,
};

#[::tracing::instrument(skip(db, request), fields(page = %request.page(), limit = %request.limit()))]
pub async fn paginate(
    db: &DatabaseConnection,
    request: RolePaginationRequest,
) -> anyhow::Result<RolePaginationResponse> {
    ::tracing::info!("Fetching paginated roles");

    let mut query = Entity::find();

    if let Some(search) = request.search() {
        let search = format!("%{}%", search);

        query = query.filter(
            Condition::any()
                .add(Column::Code.like(search.clone()))
                .add(Column::Name.like(search.clone())),
        );
    }

    let total = query
        .clone()
        .count(db)
        .await
        .context("Failed to count roles")?;

    query = query
        .limit(request.limit())
        .offset(request.offset())
        .order_by(
            match request.order() {
                RolePaginationOrder::Code => Column::Code,
                RolePaginationOrder::Name => Column::Name,
                RolePaginationOrder::CreatedAt => Column::Code,
            },
            request.sort(),
        );

    let roles = query
        .all(db)
        .await
        .context("Failed to fetch roles from database")?;

    ::tracing::info!(count = roles.len(), total = total, "Roles fetched successfully");

    Ok(RolePaginationResponse {
        total,
        page: request.page(),
        pages: total / request.limit() + 1,
        data: roles.iter().map(|role| role.into()).collect(),
    })
}
