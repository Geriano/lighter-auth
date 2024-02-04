use lighter_common::prelude::*;
use sea_orm::prelude::*;
use sea_orm::{ColumnTrait, QueryOrder, QuerySelect};

use crate::entities::v1::roles::{Column, Entity};
use crate::responses::v1::role::{
    RolePaginationOrder, RolePaginationRequest, RolePaginationResponse,
};

pub async fn paginate(
    db: &DatabaseConnection,
    request: RolePaginationRequest,
) -> Result<RolePaginationResponse, Error> {
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
                RolePaginationOrder::Code => Column::Code,
                RolePaginationOrder::Name => Column::Name,
                RolePaginationOrder::CreatedAt => Column::Code,
            },
            request.sort().into(),
        );

    let roles = query.all(db).await?;

    Ok(RolePaginationResponse {
        total,
        page: request.page(),
        pages: total / request.limit() + 1,
        data: roles.iter().map(|role| role.into()).collect(),
    })
}
