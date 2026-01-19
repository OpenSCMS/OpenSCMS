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

use actix_web::HttpRequest;
use actix_web::{HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::endpoint_error_codes::{BadRequestErrorCodes, Ieee1609Dot2Dot1ErrorCodes};
use scmscommon::{AppState, GlobalConfig, errors, get_current_time_seconds};
use sea_orm::DatabaseConnection;

use crate::persistence::eca_certificates::{latest_eca_certificate_chain, latest_eca_private_key};
use crate::persistence::eca_request_management::{
    fetch_last_request_time_and_count_by_ip, store_new_eca_request, update_status_by_ids,
};

use crate::ee_registration_interaction::{check_ee_registration, update_ee_status};

/// 1609.2.1 SS 6.3.4.2 Enrollment certificate request
///
/// Request Enrollment Certificate
#[utoipa::path(
    post,
    tag = "Enrollment Certificates",
    path = "enrollment-certificate",
    context_path = "/v3/",
    request_body(
        content = [u8],
        content_type = "application/octet-stream",
        description = "EeEcaCertRequestSpdu",
    ),
    params(
        ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
    ),
    responses(
        (status = 202, body = [u8], description = "EcaEeCertResponseSpdu" ),
        (
          status = 400,
          description = "Bad request",
          headers(("Ieee-1609.2.1-Error", description = "returned if 'scmsv3Error=fine'"))
        ),
        (status = 401, description = "Unauthorized"),
        (
          status = 403,
          description = "Forbidden",
          headers(("Ieee-1609.2.1-Error", description = "returned if 'scmsv3Error=fine'"))
        ),
        (status = 405, description = "Method not allowed"),
        (status = 408, description = "Request timeout"),
        (status = 429, description = "Too many requests"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Service unavailable")
      )
  )]
#[actix_web::post("/enrollment-certificate")]
pub async fn request_enrollment_certificate(
    data: web::Data<AppState>,
    req_body: web::Bytes,
    req: HttpRequest,
) -> impl Responder {
    // Time received
    let time_request_received = get_current_time_seconds();
    let app_ip = req.peer_addr().map(|ip| ip.ip().to_string());

    log::debug!(
        "Received POST request from {:?} at {:?} for enrollment certificate",
        app_ip,
        time_request_received
    );

    let result = handle_post_request(req_body, app_ip, time_request_received, &data.db).await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::ACCEPTED)
            .append_header(("Content-Type", "application/octet-stream"))
            .body(x),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_post_request(
    req_body: web::Bytes,
    app_ip: Option<String>,
    time_request_received: u32,
    db: &DatabaseConnection,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    // These error codes are not covered in this implementation
    // TODO: 400-33 disallowed region. (GeographicRegion)
    // TODO: 400-80 wrong ECA.

    // Validate request frequency
    check_eca_incomming_request_frequency(app_ip, time_request_received, db).await?;

    // 1. Load input data
    let enrollment_certificate_request = req_body.to_vec();
    let eca_private_key = latest_eca_private_key(db).await?;

    // 2. Call the ECA library
    log::debug!(
        "Processing ECA enrollment certificate request: {:?}",
        enrollment_certificate_request
    );

    // 2.1 Fetch Canonical ID
    let in_canonical_id = oscms_bridge::eca_extract_canonical_id_from_enrollment_request(
        enrollment_certificate_request.clone(),
    )
    .map_err(|e| {
        log::error!("Error while getting Canonical ID: {:?}", e);

        let (message, error_code) = oscms_bridge::oscms_bridge_error_to_ieee_1609_error_codes(e);

        log::error!("OSCMS-BRIDGE Codec Error: {}", message);

        let status_code = match error_code {
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(_) => StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::Forbidden(_) => StatusCode::FORBIDDEN,
            Ieee1609Dot2Dot1ErrorCodes::Unauthorized => StatusCode::UNAUTHORIZED,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        errors::HandleResponseError::new(message.as_str(), status_code, error_code)
    })?;

    log::debug!(
        "Checking EE registration for Canonical ID: {}",
        in_canonical_id
    );

    // If canonical ID is not provided, it's an error
    if in_canonical_id.is_empty() {
        return Err(errors::HandleResponseError::new(
            "Canonical ID is required",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::InvalidEnrollmentCertificate,
            ),
        ));
    }

    let (canonical_public_key, ee_registration_vid, canonical_id) =
        check_ee_registration(false, Some(in_canonical_id), None).await?;

    let eca_certificate_chain = latest_eca_certificate_chain(db).await?;
    let eca_max_wait = GlobalConfig::global().param_eca_max_wait as u32;
    let eca_max_age = GlobalConfig::global().param_eca_max_age as u32;

    let (certificate, vehicle_id, issuer_id) =
        oscms_bridge::eca_enrollment_certificate_request_handler(
            enrollment_certificate_request,
            eca_private_key,
            canonical_public_key,
            eca_certificate_chain,
            ee_registration_vid,
            time_request_received,
            eca_max_wait,
            eca_max_age,
        )
        .map_err(|e| {
            log::error!(
                "Error processing ECA enrollment certificate request: {:?}",
                e
            );

            let (message, error_code) =
                oscms_bridge::oscms_bridge_error_to_ieee_1609_error_codes(e);

            log::error!("OSCMS-BRIDGE Codec Error: {}", message);

            let status_code = match error_code {
                Ieee1609Dot2Dot1ErrorCodes::BadRequest(_) => StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::Forbidden(_) => StatusCode::FORBIDDEN,
                Ieee1609Dot2Dot1ErrorCodes::Unauthorized => StatusCode::UNAUTHORIZED,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };

            errors::HandleResponseError::new(message.as_str(), status_code, error_code)
        })?;

    // path new status
    update_ee_status(canonical_id.clone(), "Enrolled".to_string()).await?;

    log::info!(
        "Successfully processed enrollment certificate request for vehicle ID: {}, issuer ID: {}",
        vehicle_id,
        issuer_id
    );

    Ok(certificate)
}

async fn check_eca_incomming_request_frequency(
    app_ip: Option<String>,
    request_time: u32,
    db: &DatabaseConnection,
) -> Result<(), errors::HandleResponseError> {
    if app_ip.is_none() {
        return Err(errors::HandleResponseError::new(
            "No IP address found in request",
            StatusCode::UNAUTHORIZED,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    let ip = app_ip.unwrap();
    let request_entry = fetch_last_request_time_and_count_by_ip(ip.clone(), db).await?;

    if request_entry.is_none() {
        store_new_eca_request(db, ip.clone(), request_time).await?;
        return Ok(());
    }

    let (last_request_time, period_request_count) = request_entry.unwrap();

    // 400-73 within eca-minWait.
    // Minimum time that an EE shall wait before retrying the request.
    let min_wait = GlobalConfig::global().param_eca_min_wait as u32;
    let current_time = get_current_time_seconds();

    let time_since_last_request = current_time - last_request_time;
    if time_since_last_request < min_wait {
        return Err(errors::HandleResponseError::new(
            "Within eca-minWait",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::WithinEcaMinWait),
        ));
    }

    // TODO: 400-71 outside eca-maxReqs.
    // Maximum number of requests in a 7 × 24-hour period.
    let max_requests = GlobalConfig::global().param_eca_max_reqs as i32;
    if period_request_count + 1 > max_requests {
        log::debug!(
            "Weekly request count: {} max_requests {}",
            period_request_count + 1,
            max_requests
        );
        return Err(errors::HandleResponseError::new(
            "Outside eca-maxReqs",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::OutsideEcaMaxReqs),
        ));
    }

    // Update request table
    update_status_by_ids(db, ip, request_time, period_request_count + 1).await?;
    Ok(())
}
