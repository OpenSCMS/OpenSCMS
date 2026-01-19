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

use actix_web::{HttpResponse, Responder, web};
use scmscommon::AppState;

#[utoipa::path(
    post,
    path = "/",
    request_body(
      content = Vec<String>,
      content_type = "application/json",
      description = "List of PayloadRaToCam",
    ),
    responses(
      (status = 200, body = Vec<String>, description = "List of PayloadCamToRa" ),
      (status = 400, description = "Bad request"),
      (status = 401, description = "Unauthorized"),
      (status = 403, description = "Forbidden"),
      (status = 404, description = "Not found"),
      (status = 405, description = "Method not allowed"),
      (status = 408, description = "Request timeout"),
      (status = 416, description = "Request range no satisfiable"),
      (status = 429, description = "Too many requests"),
      (status = 500, description = "Internal server error"),
      (status = 503, description = "Service unavailable"),
    )
  )]
#[actix_web::post("/")]
pub async fn process_ra_request(
    _data: web::Data<AppState>,
    _req_body: web::Json<Vec<String>>,
) -> impl Responder {
    HttpResponse::InternalServerError().body("CAM Not Implemented")
}
