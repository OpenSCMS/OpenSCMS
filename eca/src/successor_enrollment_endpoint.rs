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

use actix_web::{HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes;
use scmscommon::{AppState, errors, get_current_time_seconds};
use sea_orm::DatabaseConnection;

use crate::persistence::eca_certificates::{
    latest_eca_certificate_chain, latest_eca_private_key, latest_eca_public_uncompressed,
};

use crate::ee_registration_interaction::{check_ee_registration, update_ee_status};

/// Enpoint not specified by 1609.2 standard
///
/// Called by RA to request C-OER encoded EcaEeCertResponseSpdu
#[utoipa::path(
    post,
    tag = "Successor Enrollment Certificates",
    path = "successor-enrollment-certificate",
    request_body(
        content = [u8],
        content_type = "application/octet-stream",
        description = "EeRaSuccessorEnrollmentCertRequestSpdu",
    ),
    responses(
        (
            status = 202, body = [u8], description = "C-OER encoded EcaEeCertResponseSpdu",
            headers(("Content-Disposition", description = "attachment filename"))
        ),
        (
          status = 400,
          description = "Bad request",
          headers(("Ieee-1609.2.1-Error", description = "Indicates the issue: 400-<error code>"))
        ),
        (status = 401, description = "Unauthorized"),
        (
          status = 403,
          description = "Forbidden",
          headers(("Ieee-1609.2.1-Error", description = "Indicates the issue: 403-<error code>"))
        ),
        (status = 405, description = "Method not allowed"),
        (status = 408, description = "Request timeout"),
        (status = 429, description = "Too many requests"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Service unavailable")
      )
  )]
#[actix_web::post("/successor-enrollment-certificate")]
pub async fn process_successor_enrollment_request(
    data: web::Data<AppState>,
    req_body: web::Bytes,
    req: actix_web::HttpRequest,
) -> impl Responder {
    log::debug!("Received POST request for successor enrollment certificate");
    // Time received
    let time_request_received = get_current_time_seconds();

    // Get header hashId. If it's empty so we have a problem
    let request_hash_id: String = match req.headers().get("request_hash_id") {
        Some(x) => x.to_str().unwrap().to_string(),
        None => {
            let status_code = StatusCode::INTERNAL_SERVER_ERROR;
            let error_code = Ieee1609Dot2Dot1ErrorCodes::NotDefined;
            let error = errors::HandleResponseError::new(
                "No request_hash_id found",
                status_code,
                error_code,
            );
            return error.http_error_response();
        }
    };

    let result = handle_successor_enrollment_request(
        req_body,
        request_hash_id,
        time_request_received,
        &data.db,
    )
    .await;
    match result {
        // The file contents are a C-OER encoded EcaEeCertResponseSpdu as defined in 8.3
        Ok((filename, payload)) => HttpResponseBuilder::new(StatusCode::ACCEPTED)
            .append_header(("Content-Type", "application/octet-stream"))
            .append_header((
                "Content-Disposition",
                format!("attachment; filename={}", filename).as_str(),
            ))
            .body(payload),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_successor_enrollment_request(
    req_body: web::Bytes,
    request_hash_id: String,
    time_request_received: u32,
    db: &DatabaseConnection,
) -> Result<(String, Vec<u8>), errors::HandleResponseError> {
    log::info!("Starting Handle GET request for successor enrollment certificate.");

    // 1. Load input data
    let successor_enrollment_request = req_body.to_vec();
    let eca_private_key = latest_eca_private_key(db).await?;
    let eca_public_uncompressed = latest_eca_public_uncompressed(db).await?;
    let eca_certificate_chain = latest_eca_certificate_chain(db).await?;

    // 2. Call the ECA library
    let (filename, file, (vehicle_id, issuer_id)) =
        oscms_bridge::eca_sucessor_enrollment_certificate_request_handler(
            successor_enrollment_request,
            request_hash_id,
            eca_private_key,
            eca_public_uncompressed,
            eca_certificate_chain,
            time_request_received,
        )
        .map_err(|e| {
            log::error!(
                "Error processing ECA successor enrollment certificate request: {:?}",
                e
            );

            let (message, error_code) =
                oscms_bridge::oscms_bridge_error_to_ieee_1609_error_codes(e);

            log::error!("OSCMS-BRIDGE Codec Error: {}", message);

            let status_code = match error_code {
                Ieee1609Dot2Dot1ErrorCodes::BadRequest(_) => StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::Forbidden(_) => StatusCode::FORBIDDEN,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            errors::HandleResponseError::new(message.as_str(), status_code, error_code)
        })?;

    // 3. Validate if the vehicle ID is enrolled and update status
    let (_, _, canonical_id) = check_ee_registration(true, None, Some(vehicle_id)).await?;

    // 4. Set new status to "Successor-Enrolled"
    update_ee_status(canonical_id, "Successor-Enrolled".to_string()).await?;

    log::info!(
        "Successfully processed successor enrollment request for vehicle ID: {}, issuer ID: {}",
        vehicle_id,
        issuer_id
    );
    Ok((filename, file))
}
