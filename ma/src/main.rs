// Copyright (c) 2025 LG Electronics, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use actix_web::{App, HttpResponse, HttpServer, Responder, middleware, web};
use scmscommon::{AppState, load_global_config, load_mysql_config, setup_db};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use utoipauto::utoipauto;
mod endpoint;

/// Provide a health-check endpoint for the service
/// Simply responds with an HTTP Status of 200
#[utoipa::path(
    get,
    path = "/healthcheck",
    responses((status = 200, description = "Success", body = String))
)]
#[actix_web::get("/healthcheck")]
async fn healthcheck(data: web::Data<AppState>) -> impl Responder {
    let server_name = data.app_name.clone();
    HttpResponse::Ok().body(format!("{server_name} is healthy\n"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // Load the configuration objects we are interested in
    let global_config = load_global_config();
    log::debug!("{:#?}", global_config);
    load_mysql_config();

    let app_state = web::Data::new(AppState {
        app_name: "MA server".to_string(),
        db: setup_db().await.unwrap(),
        celery_app: None,
    });

    // Setup database access
    // setup_tables(&app_state.db).await.unwrap();
    // TODO: when the tables are implemented we uncomment this part

    #[utoipauto(paths = "./ma/src,./scmscommon/src from scmscommon")]
    #[derive(OpenApi)]
    #[openapi(info(title = "Misbehavior Authority (MA)"))]
    struct OpenApiDoc;
    let openapi = OpenApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(app_state.clone())
            .service(healthcheck)
            .service(endpoint::run_revocation)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}").url("/api-docs/openapi.json", openapi.clone()),
            )
    })
    .bind(("0.0.0.0", global_config.la_port))
    .unwrap()
    .run()
    .await
}
