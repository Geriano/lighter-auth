use lighter_common::prelude::*;
use utoipa::OpenApi;
use utoipa_swagger_ui::{SwaggerUi, Url};

use crate::api::Definition;
use crate::controllers;
use crate::metrics::AppMetrics;
use crate::middlewares::v1::auth::Authenticated;

pub fn route(app: &mut ServiceConfig) {
    app.app_data(Data::new(Authenticated::new()));
    app.app_data(Data::new(AppMetrics::new()));
    app.service(index);
    // User
    app.service(controllers::v1::user::paginate);
    app.service(controllers::v1::user::store);
    app.service(controllers::v1::user::show);
    app.service(controllers::v1::user::update_general_information);
    app.service(controllers::v1::user::update_password);
    app.service(controllers::v1::user::delete);
    // Permission
    app.service(controllers::v1::permission::paginate);
    app.service(controllers::v1::permission::store);
    app.service(controllers::v1::permission::show);
    app.service(controllers::v1::permission::update);
    app.service(controllers::v1::permission::delete);
    // Role
    app.service(controllers::v1::role::paginate);
    app.service(controllers::v1::role::store);
    app.service(controllers::v1::role::show);
    app.service(controllers::v1::role::update);
    app.service(controllers::v1::role::delete);
    // Auth
    app.service(controllers::v1::auth::login);
    app.service(controllers::v1::auth::authenticated);
    app.service(controllers::v1::auth::logout);

    // Health check endpoints
    app.service(controllers::health::health);
    app.service(controllers::health::health_db);
    app.service(controllers::health::ready);
    app.service(controllers::health::live);

    // Metrics endpoint
    app.service(controllers::metrics::metrics);

    // must at the end!
    app.service(web::redirect("/docs", "/docs/"));
    app.service(SwaggerUi::new("/docs/{_:.*}").urls(vec![(
        Url::new("Authentication", "/api.json"),
        Definition::openapi(),
    )]));
}

#[get("/")]
pub async fn index() -> &'static str {
    "Hello World"
}
