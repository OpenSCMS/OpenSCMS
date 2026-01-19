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

use crate::persistence::eca_certificates::latest_eca_public_uncompressed;
use actix_web::{HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes;
use scmscommon::{AppState, errors};
use sea_orm::DatabaseConnection;

/// Download the ECA Public Key
#[utoipa::path(
    get,
    tag = "Public Key Downloads",
    path = "eca-public-key",
    responses(
      (status = 200, body = [u8], description = "ECA public key" ),
      (
        status = 400,
        description = "Bad request",
      ),
      (status = 401, description = "Unauthorized"),
      (
        status = 403,
        description = "Forbidden",
      ),
      (status = 405, description = "Method not allowed"),
      (status = 408, description = "Request timeout"),
      (status = 416, description = "requested range not satisfiable: retry without HTTP Range"),
      (status = 429, description = "Too many requests"),
      (status = 500, description = "Internal server error"),
      (status = 503, description = "Service unavailable")
    )
  )]
#[actix_web::get("/eca-public-key")]
pub async fn download_eca_public_key(data: web::Data<AppState>) -> impl Responder {
    log::debug!("Received GET request for ECA public key");

    // Return ECA public key
    match handle_get_eca_public_key(&data.db).await {
        Ok(x) => HttpResponseBuilder::new(StatusCode::OK).body(x),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_get_eca_public_key(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    latest_eca_public_uncompressed(db).await.map_err(|e| {
        errors::HandleResponseError::new(
            format!("Failed to fetch ECA Public Key: {}", e).as_str(),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    })
}
