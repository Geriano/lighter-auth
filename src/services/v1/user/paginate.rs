use lighter_common::prelude::*;
use sea_orm::prelude::*;
use sea_orm::{ColumnTrait, QueryOrder, QuerySelect};

use crate::entities::v1::users::{Column, Entity};
use crate::responses::v1::user::simple::{
    UserPaginationOrder, UserPaginationRequest, UserPaginationResponse,
};

pub async fn paginate(
    db: &DatabaseConnection,
    request: UserPaginationRequest,
) -> Result<UserPaginationResponse, Error> {
    let mut query = Entity::find().filter(Column::DeletedAt.is_null());

    if let Some(search) = request.search() {
        let search = format!("%{}%", search);

        query = query.filter(
            Condition::any()
                .add(Column::Name.like(search.clone()))
                .add(Column::Username.like(search.clone()))
                .add(Column::Email.like(search.clone())),
        );
    }

    let total = query.clone().count(db).await?;

    query = query
        .limit(request.limit())
        .offset(request.offset())
        .order_by(
            match request.order() {
                UserPaginationOrder::Name => Column::Name,
                UserPaginationOrder::Username => Column::Username,
                UserPaginationOrder::Email => Column::Email,
                UserPaginationOrder::EmailVerifiedAt => Column::EmailVerifiedAt,
                UserPaginationOrder::CreatedAt => Column::CreatedAt,
            },
            request.sort().into(),
        );

    let users = query.all(db).await?;

    Ok(UserPaginationResponse {
        total,
        page: request.page(),
        pages: total / request.limit() + 1,
        data: users.iter().map(|user| user.into()).collect(),
    })
}
