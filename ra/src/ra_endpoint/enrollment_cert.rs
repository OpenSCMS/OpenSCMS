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

use crate::entities::sea_orm_active_enums::RaRequestManagementRequestType;
use crate::persistence::ra_certificates::{
    latest_ra_certificate, latest_ra_enc_private_key, latest_ra_private_key,
};
use crate::persistence::successor_enrollment_certificate_store::{
    download_enrollment_certificate, store_enrollment_request_to_be_processed,
};
use crate::ra_endpoint::common::request_checker::check_ra_incomming_request_frequency_by_app_ip;
use crate::ra_endpoint::common::{
    decode_ieee1609dot2_authorization, oscms_error_to_ra_response_error,
};
use crate::ra_endpoint::worker::tasks::run_successor_enrollment_request;

use actix_web::{HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::{
    AppState, endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes, errors,
    get_current_time_period_days, get_current_time_seconds,
};
use scmscommon::{BadRequestErrorCodes, ForbiddenErrorCodes};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

/// 1609.2.1 SS 6.3.5.4 Successor Enrollment Certificate request
///
/// Initiate generation of a successor enrollment certificate
#[utoipa::path(
  post,
  tag = "Successor Enrollment Certificates",
  path = "successor-enrollment-certificate",
  context_path = "/v3/",
  request_body(
    content = [u8],
    content_type = "application/octet-stream",
    description = "EaRaSuccessorEnrollmentCertRequestSpdu"
  ),
  params(
    ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
  ),
  responses(
    (status = 202, body = [u8], description = "RaEeEnrollmentCertAckSpdu"),
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
#[actix_web::post("/successor-enrollment-certificate")]
pub async fn request_successor_enrollment_certificate(
    data: web::Data<AppState>,
    req_body: web::Bytes,
    req: actix_web::HttpRequest,
) -> impl Responder {
    log::debug!("Received POST request for successor enrollment certificate");
    let app_ip = match req.peer_addr() {
        Some(ip) => ip.ip().to_string(),
        None => {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .append_header(("Ieee-1609.2.1-Error", "400-42"))
                .body("Failed parsing");
        }
    };

    // Time received
    let time_request_received = get_current_time_seconds();
    log::debug!("Receivede from {:?} at {}", app_ip, time_request_received);

    let result = handle_successor_post_request(
        req_body,
        time_request_received,
        app_ip,
        &data.db,
        &data.celery_app,
    )
    .await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::ACCEPTED)
            .append_header(("Content-Type", "application/octet-stream"))
            .body(x),
        Err(x) => x.http_error_response(),
    }
}

/// 1609.2.1 SS 6.3.5.5 Successor Enrollment Certificate Download
///
/// Download generated successor enrollment certificate
#[utoipa::path(
  get,
  tag = "Successor Enrollment Certificates",
  path = "successor-enrollment-certificate",
  context_path = "/v3/",
  params(
    ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
    ("Ieee-1609.2.1-Authorization" = String, Header, description = "EeRaDownloadRequestSpdu or EeDownloadRequestPlainSpdu")
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
      headers(("Ieee-1609.2.1-Error", description = "returned if 'scmsv3Error=fine'"))
    ),
    (status = 404, description = "Not found"),
    (status = 405, description = "Method not allowed"),
    (status = 408, description = "Request timeout"),
    (status = 416, description = "Request range no satisfiable"),
    (status = 429, description = "Too many requests"),
    (status = 500, description = "Internal server error"),
    (status = 503, description = "Service unavailable")
  )
)]
#[actix_web::get("/successor-enrollment-certificate")]
pub async fn download_successor_certificate(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    log::info!("Received GET request for successor enrollment certificate");
    let app_ip = match req.peer_addr() {
        Some(ip) => ip.ip().to_string(),
        None => {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .append_header(("Ieee-1609.2.1-Error", "400-42"))
                .body("Failed parsing");
        }
    };

    // Time received
    let time_request_received = get_current_time_seconds();
    log::debug!("Receivede from {:?} at {}", app_ip, time_request_received);

    // Get header Ieee-1609.2.1-Authorization. If it's empty so we have situation 2)
    let ieee1609dot2_authorization_encoded: Option<String> = req
        .headers()
        .get("Ieee-1609.2.1-Authorization")
        .map(|x| x.to_str().unwrap().to_string());

    log::debug!(
        "Header Ieee-1609.2.1-Authorization: {:?}",
        ieee1609dot2_authorization_encoded
    );

    if ieee1609dot2_authorization_encoded.is_none() {
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
            .append_header(("Ieee-1609.2.1-Error", "400-42"))
            .body("Failed parsing");
    }

    log::debug!(
        "Received GET request for successor enrollment certificate with header:{:?}",
        ieee1609dot2_authorization_encoded
    );

    let result = handle_successor_get_request(
        ieee1609dot2_authorization_encoded.unwrap(),
        time_request_received,
        app_ip,
        &data.db,
    )
    .await;

    match result {
        // An authorization certificate download file as defined in 8.2, where the filename shall be equal to the
        // eeRaDownloadRequest.filename field of the Ieee-1609.2.1-Authorization provided in the request
        // headers.
        Ok((payload, filename)) => HttpResponseBuilder::new(StatusCode::ACCEPTED)
            .append_header(("Content-Type", "application/octet-stream"))
            .append_header((
                "Content-Disposition",
                format!("attachment; filename={}", filename).as_str(),
            ))
            .body(payload),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_successor_get_request(
    ieee1609dot2_authorization_encoded: String,
    time_request_received: u32,
    app_ip: String,
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    log::info!("Starting Handle GET request for successor enrollment certificate.");
    // TODO: Not covering - lack of details from standard
    // 400-81 wrong RA.

    // ETDES-694: We first need to handle EeRaDownloadRequestSpdu as signed encrypted data
    // and so we'll be able to fetch Enrollment Certificate and apply validations:
    // Throw /GET Error-Code 400-20 blocked enrollment certificate.
    // Throw /GET Error-Code 400-21 unregistered enrollment certificate.
    // Throw /GET Error-Code 400-62 invalid enrollment certificate.
    // Throw /GET Error-Code 400-712 outside enrollment certificate’s validityPeriod.

    // Validate request frequency
    check_ra_incomming_request_frequency_by_app_ip(
        app_ip,
        time_request_received,
        RaRequestManagementRequestType::SuccessorEnrollmentDownload,
        db,
    )
    .await?;

    // 2. Extracting HashId from SPDU
    let request_filename_from_spdu = decode_ieee1609dot2_authorization(
        ieee1609dot2_authorization_encoded,
        time_request_received,
        db,
    )
    .await?;

    log::debug!(
        "Request filename from spdu: {}",
        request_filename_from_spdu.clone(),
    );

    // 1. Fetch Certificate FileName and File by Hash ID
    let (filename, certificate_file, error_code, status_code) =
        download_enrollment_certificate(db, request_filename_from_spdu.clone()).await?;

    // 3. Check if there's content
    if certificate_file.is_empty() {
        log::error!(
            "No certificates found for hash id {}",
            request_filename_from_spdu
        );
        if let (Some(status_code_value), Some(error_code_value)) = (status_code, error_code) {
            if status_code_value == 400 {
                let code: Option<BadRequestErrorCodes> =
                    num::FromPrimitive::from_i32(error_code_value);
                return Err(errors::HandleResponseError::new(
                    "Bad Request for successor enrollment certificate request",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                        code.unwrap_or(BadRequestErrorCodes::Undefined),
                    ),
                ));
            } else if status_code_value == 403 {
                let code: Option<ForbiddenErrorCodes> =
                    num::FromPrimitive::from_i32(error_code_value);
                return Err(errors::HandleResponseError::new(
                    "Forbidden successor enrollment certificate request",
                    StatusCode::FORBIDDEN,
                    Ieee1609Dot2Dot1ErrorCodes::Forbidden(
                        code.unwrap_or(ForbiddenErrorCodes::Undefined),
                    ),
                ));
            } else {
                // Error 500 for some reason
                return Err(errors::HandleResponseError::new(
                    format!(
                        "No certificates found for filename {} or failed to process request.",
                        request_filename_from_spdu
                    )
                    .as_str(),
                    StatusCode::NOT_FOUND,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                ));
            }
        }
        return Err(errors::HandleResponseError::new(
            format!(
                "No certificates found for filename {} or failed to process request.",
                request_filename_from_spdu
            )
            .as_str(),
            StatusCode::NOT_FOUND,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    log::debug!("Found certificate content len {}", certificate_file.len());

    Ok((certificate_file, filename))
}

async fn handle_successor_post_request(
    req_body: web::Bytes,
    time_request_received: u32,
    app_ip: String,
    db: &DatabaseConnection,
    celery_app: &Option<Arc<celery::Celery>>,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    log::info!("Starting Handle POST request for successor enrollment certificate.");
    // TODO: Not covering - lack of details from standard
    // 400-81 wrong RA.
    // 400-710 outside ra-maxOverlap.

    // ETDES-687 TODO: Throw /POST Error code: 400-20 blocked enrollment certificate.
    // ETDES-686 TODO: Throw /POST Error code: 400-21 unregistered enrollment certificate.
    // As per 4.1.4.3.1, we have to check for unregistered & blocked EE enrollment certificate

    // Validate request frequency
    check_ra_incomming_request_frequency_by_app_ip(
        app_ip,
        time_request_received,
        RaRequestManagementRequestType::SuccessorEnrollmentRequest,
        db,
    )
    .await?;

    // 1. Processing the requesst by calling lib1609
    let encoded = req_body.to_vec();
    let ra_private_key = latest_ra_private_key(db).await?;
    let ra_enc_private_key = latest_ra_enc_private_key(db).await?;
    let ra_certificate = latest_ra_certificate(db).await?;

    log::debug!("Handling successor enrollment certificate request by calling lib1609",);

    let (ack_message, decoded_payload, hash_id) =
        match oscms_bridge::ra_sucessor_enrollment_certificate_request_handler(
            encoded.clone(),
            ra_private_key,
            ra_enc_private_key,
            ra_certificate,
        ) {
            Ok((ack_message, decoded_payload, hash_id)) => (ack_message, decoded_payload, hash_id),
            Err(x) => return Err(oscms_error_to_ra_response_error(x)),
        };

    log::debug!(
        "Successor enrollment certificate request processed successfully: hash id ({:?})",
        hash_id
    );

    // 2. Store request to be processed and sending task to worker
    log::debug!("Storing request to be processed.");
    match store_enrollment_request_to_be_processed(
        db,
        hash_id.clone(),
        get_current_time_period_days() as u64,
        decoded_payload,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            return Err(errors::HandleResponseError::new(
                format!("Failed to store request to be processed: {}", e).as_str(),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
    };

    log::debug!("Seding task to be processed by worker.");
    if let Some(celery_app) = celery_app {
        celery_app
            .send_task(run_successor_enrollment_request::new(hash_id.clone()))
            .await
            .map_err(|e| {
                errors::HandleResponseError::new(
                    format!("Failed to send task to celery: {}", e).as_str(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                )
            })?;
    } else {
        return Err(errors::HandleResponseError::new(
            "Celery app is not configured",
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    // 3. Return ack
    log::info!(
        "Finishing successor enrollment certificate ({:?}) request handling.",
        hash_id
    );
    Ok(ack_message)
}
