use lighter_common::prelude::*;
use sea_orm::prelude::*;
use sea_orm::{ColumnTrait, QueryOrder, QuerySelect};

use crate::entities::v1::permissions::{Column, Entity};
use crate::responses::v1::permission::{
    PermissionPaginationOrder, PermissionPaginationRequest, PermissionPaginationResponse,
};

pub async fn paginate(
    db: &DatabaseConnection,
    request: PermissionPaginationRequest,
) -> Result<PermissionPaginationResponse, Error> {
    let mut query = Entity::find();

    if let Some(search) = request.search() {
        let search = format!("%{}%", search);

        query = query.filter(
            Condition::any()
                .add(Column::Code.like(search.clone()))
                .add(Column::Name.like(search.clone())),
        );
    }

    let total = query.clone().count(db).await?;

    query = query
        .limit(request.limit())
        .offset(request.offset())
        .order_by(
            match request.order() {
                PermissionPaginationOrder::Code => Column::Code,
                PermissionPaginationOrder::Name => Column::Name,
                PermissionPaginationOrder::CreatedAt => Column::Code,
            },
            request.sort(),
        );

    let permissions = query.all(db).await?;

    Ok(PermissionPaginationResponse {
        total,
        page: request.page(),
        pages: total / request.limit() + 1,
        data: permissions
            .iter()
            .map(|permission| permission.into())
            .collect(),
    })
}
