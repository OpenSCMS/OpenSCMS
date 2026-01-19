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

use actix_web::{HttpResponse, HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::AppState;
use scmscommon::core_types::PayloadRaToLa;

#[utoipa::path(
  post,
  path = "/",
  request_body(
    content = Vec<u8>,
    content_type = "application/json",
    description = "Request coming from EE.",
  ),
  responses(
    (status = 200, body = PayloadLaToRa, description = "PayloadLaToRa" ),
  )
)]
#[actix_web::post("/")]
pub async fn run_revocation(
    _data: web::Data<AppState>,
    _req_body: web::Json<PayloadRaToLa>,
) -> impl Responder {
    // let result = handle_post_request(req_body, &data.db).await;
    // TODO: handle post request based on the LA's endpoint, adapt to the specification of the MA implementation
    let result: Result<i32, ()> = Ok(42);
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::OK)
            .append_header(("Content-Type", "application/json"))
            .json(x),
        // Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
        Err(_) => HttpResponse::InternalServerError().body("This is a temporary error!"),
    }
}
