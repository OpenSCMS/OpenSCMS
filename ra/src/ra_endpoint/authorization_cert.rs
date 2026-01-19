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
use crate::entities::sea_orm_active_enums::Status;
use crate::persistence::caterpillar::CaterpillarPersistence;
use crate::persistence::certificate_store::{
    fetch_certificates_files_by_hash_id_and_i_value, update_set_downloaded_true,
};
use crate::persistence::ee_registration::{
    fetch_ee_registration_by_device_id, patch_status_ee_registration_by_device_id,
};
use crate::persistence::ra_certificates::{
    latest_eca_certificate, latest_eca_public_key, latest_ra_certificate,
    latest_ra_enc_private_key, latest_ra_private_key,
};
use crate::persistence::x_dot_info_store::{
    fetch_x_info_file_by_hash_id_and_i_value, update_x_dot_info_set_downloaded_true,
};
use crate::ra_endpoint::common::oscms_error_to_ra_response_error;
use crate::ra_endpoint::common::zip_helpers::write_file_to_zip;
use crate::ra_endpoint::common::{
    decode_ieee1609dot2_authorization,
    request_checker::check_ra_incomming_request_frequency_by_app_ip,
    request_checker::check_ra_incomming_request_frequency_by_vid,
};
use crate::ra_endpoint::validation;
use crate::ra_endpoint::worker::tasks::eca_public_key_download_and_store;
use crate::ra_endpoint::worker::tasks::run_authorization_request_processing;
use actix_web::{HttpResponseBuilder, Responder, http::StatusCode, web};
use regex::Regex;
use scmscommon::{
    AppState, GlobalConfig, endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes, errors,
    get_current_time_period_days, get_current_time_seconds,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::ToSchema;
use zip::ZipWriter;

/// 1609.2.1 SS 6.3.5.2 Authorization Certificate Request
///
/// Initiate generation of an RA Certificate
#[utoipa::path(
  post,
  tag = "Authorization Certificates",
  path = "authorization-certificate",
  context_path = "/v3/",
  request_body(
    content = [u8],
    content_type = "application/octet-stream",
    description = "EaRaCertRequestSpdu",
  ),
  params(
    ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
  ),
  responses(
    (status = 202, body = [u8], description = "RaEeCertAckSpdu" ),
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
#[actix_web::post("/authorization-certificate")]
pub async fn request_auth_certificate(
    data: web::Data<AppState>,
    req_body: web::Bytes,
    req: actix_web::HttpRequest,
) -> impl Responder {
    log::debug!("Received POST request for authorization certificate");
    let app_ip = match req.peer_addr() {
        Some(ip) => ip.ip().to_string(),
        None => {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .append_header(("Ieee-1609.2.1-Error", "400-42"))
                .body("Failed parsing");
        }
    };
    log::debug!("Receivede from {:?}", app_ip);

    let result = handle_post_request(req_body, app_ip, &data.db, &data.celery_app).await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::ACCEPTED)
            .append_header(("Content-Type", "application/octet-stream"))
            .body(x),
        Err(x) => x.http_error_response(),
    }
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams)]
pub struct CertFilename {
    filename: Option<String>,
}

/// 1609.2.1 SS 6.3.5.3 Authorization Certificate Downoad
///
/// Download a generated RA Certificate
#[utoipa::path(
  get,
  tag = "Authorization Certificates",
  path = "authorization-certificate",
  context_path = "/v3/",
  params(
    (
      "filename" = Option<String>,
      Query,
      description = "download filename"
    ),
    ("Authorization", Header, description = "Bearer [access-token/AT]"),
    ("Ieee-1609.2.1-Authorization" = String, Header, description = "EeRaDownloadRequestSpdu or EeDownloadRequestPlainSpdu"),
  ),
  responses(
    (
      status = 200,
      body = [u8], description = "Zip file content",
      headers(("Content-Disposition", description = "attachment filename"))
    ),
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
    (status = 404, description = "Not found"),
    (status = 405, description = "Method not allowed"),
    (status = 408, description = "Request timeout"),
    (status = 416, description = "Request range no satisfiable"),
    (status = 429, description = "Too many requests"),
    (status = 500, description = "Internal server error"),
    (status = 503, description = "Service unavailable"),
  )
)]
#[actix_web::get("/authorization-certificate")]
pub async fn download_auth_certificate(
    data: web::Data<AppState>,
    query_params: web::Query<CertFilename>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    log::info!("Received GET request for authorization certificate");
    // Time received
    let time_request_received = get_current_time_seconds();

    // This endpoint accepts tow different ways:
    // 1) Authorization certificate download without filename in URL
    // 2) Authorization certificate download with filename in URL
    //      If scmsV3-eeAuth = enrollment: Ieee-1609.2.1-Authorization:
    //      EeRaDownloadRequestSpdu. The request is invalid if the
    //      eeRaDownlaodRequest.filename field is not equal to the query parameter
    //      filename provided in the request path.
    //      get header ieee1609dot2_authorization_encoded

    // Get filename from query param. If it's empty so we have situation 1)
    // Authorization certificate download without filename in URL.
    let query_param_filename = if query_params.filename.is_some() {
        query_params.filename.clone()
    } else {
        None
    };

    log::debug!("Query param filename: {:?}", query_param_filename);

    // Get header Ieee-1609.2.1-Authorization. If it's empty so we have situation 2)
    let ieee1609dot2_authorization_encoded: Option<String> = req
        .headers()
        .get("Ieee-1609.2.1-Authorization")
        .map(|x| x.to_str().unwrap().to_string());

    log::debug!(
        "Header Ieee-1609.2.1-Authorization: {:?}",
        ieee1609dot2_authorization_encoded
    );

    if query_param_filename.is_none() && ieee1609dot2_authorization_encoded.is_none() {
        return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
            .append_header(("Ieee-1609.2.1-Error", "400-42"))
            .body("Failed parsing");
    }

    let app_ip = match req.peer_addr() {
        Some(ip) => ip.ip().to_string(),
        None => {
            return HttpResponseBuilder::new(StatusCode::BAD_REQUEST)
                .append_header(("Ieee-1609.2.1-Error", "400-42"))
                .body("Failed parsing");
        }
    };

    log::debug!(
        "Received GET request for authorization certificate from {:?} with filename query param {:?} and header:{:?}",
        app_ip,
        query_param_filename,
        ieee1609dot2_authorization_encoded
    );

    // At this point we can have:
    // 1) query_param_filename not empty and ieee1609dot2_authorization_encoded not empty.
    //     So It's corresponds to situation 2) Authorization certificate download with filename in URL.
    //     And The request is invalid if the eeRaDownlaodRequest.filename field is not equal to the query parameter
    //     filename provided in the request path.
    // 2) query_param_filename not empty and ieee1609dot2_authorization_encoded empty.
    //    So It's corresponds to situation 2) Authorization certificate download with filename in URL.
    //    And scmsV3-eeAuth = none. Nothing to check
    // 3) query_param_filename empty and ieee1609dot2_authorization_encoded not empty.
    //    So It's corresponds to situation 1) Authorization certificate download without filename in URL.
    //    And scmsV3-eeAuth = enrollment.
    let result = handle_get_request(
        ieee1609dot2_authorization_encoded,
        query_param_filename,
        app_ip,
        &data.db,
        time_request_received,
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

async fn handle_post_request(
    req_body: web::Bytes,
    app_ip: String,
    db: &DatabaseConnection,
    celery_app: &Option<Arc<celery::Celery>>,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    log::info!("Starting Handle POST request for authorization certificate.");
    // TODO: Not covering - lack of details from standard
    // 400-81 wrong RA.
    // 400-710 outside ra-maxOverlap.

    // Process Request
    let time_request_received = get_current_time_seconds();
    let encoded = req_body.to_vec();
    let eca_public_key_fetch_result = latest_eca_public_key(db).await;
    let mut eca_public = [0; 64];
    match eca_public_key_fetch_result {
        Ok(key) => eca_public.copy_from_slice(&key),
        Err(_) => {
            eca_public = eca_public_key_download_and_store(db).await.map_err(|e| {
                errors::HandleResponseError::new(
                    &format!("Failed to get eca public key: {}", e.message),
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                )
            })?;
        }
    }

    log::debug!(
        "Decoding certificate request: time_request_received {}",
        time_request_received
    );

    let ra_private = latest_ra_enc_private_key(db).await?;
    let eca_certificate = latest_eca_certificate(db).await?;
    let ra_certificate = latest_ra_certificate(db).await?;
    let (caterpillar, non_butterfly, (vid, generation_time, certificate_validity_start)) =
        oscms_bridge::decode_certificate_request_spdu(
            encoded.clone(),
            eca_public,
            ra_private,
            ra_certificate,
            eca_certificate,
        )
        .map_err(oscms_error_to_ra_response_error)?;

    if caterpillar.is_none() && non_butterfly.is_none() {
        return Err(errors::HandleResponseError::new(
            "Failed to decode certificate request",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
        ));
    }
    log::debug!(
        "Decoded certificate request successfully: generation_time {:?} certificate_validity_start {:?}",
        generation_time,
        certificate_validity_start
    );

    // Composing ACK righ now
    let ack_response = certificate_request_ack_response(encoded, db).await?;
    log::debug!("ACK response composed successfully");

    // Validate request frequency
    check_ra_incomming_request_frequency_by_vid(
        app_ip,
        vid,
        time_request_received,
        RaRequestManagementRequestType::AuthorizationRequest,
        db,
    )
    .await?;
    log::debug!("Request accepted withing frequency and count!");

    // Validate decoded certificate request
    let current_time = get_current_time_seconds();
    validation::validate_generation_time(
        current_time,
        generation_time as u32,
        time_request_received,
        true,
    )?;
    log::debug!("Request generation time validated!");

    // Fetch EE registration by device ID
    let ee_registration_entry = fetch_ee_registration_by_device_id(vid, db).await?;

    let is_ee_registered = ee_registration_entry.is_some();
    let is_ee_blocked = {
        if let Some(ee_registration) = ee_registration_entry {
            ee_registration.status == Status::Blocked
        } else {
            false
        }
    };

    log::debug!(
        "EE registration found: is_ee_registered: {}, is_ee_blocked: {}",
        is_ee_registered,
        is_ee_blocked
    );

    // Throw /POST Error code: 400-21 unregistered enrollment certificate.
    //    The assumption in this document is that the enrollment certificate is used as the primary identifier of an EE
    //    to the SCMS, in particular to the RA. To prevent an EE from using its enrollment certificate to request
    //    certificates via multiple RAs and so receiving more certificates than it is entitled to, the assumption is that
    //    RAs will only process requests from EEs whose enrollment certificates are registered with that RA. If access
    //    to the RA is authorized using OAuth access tokens (ATs) as specified in 6.2, it is assumed that the OAuth
    //    authorization server (OAS) also has access to a list of registered EEs and will only issue ATs to EEs on that
    //    list. The list may use the enrollment certificate or any other appropriate identifier to identify registered EEs.
    //    This document does not specify interfaces to be used to register the EE or the enrollment certificate with an
    //    RA or other SCMS component. An enrollment certificate that is not registered with an RA is referred to as
    //    an unregistered enrollment certificate.
    if !is_ee_registered {
        return Err(errors::HandleResponseError::new(
            "Unregistered enrollment certificate",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                errors::BadRequestErrorCodes::UnregisteredEnrollmentCertificate,
            ),
        ));
    }

    // As per 4.1.4.3.1, we have to check for unregistered & blocked EE enrollment certificate
    // Throw /POST Error code: 400-20 blocked enrollment certificate.
    //    If an EE is determined by the MA to be misbehaving as described in 4.1.1, or otherwise to need certificate
    //    privileges to be withdrawn, the MA may instruct the RA to “block” that EE by marking the EE’s enrollment
    //    certificate as blocked. If an EE is blocked, the RA shall deny certificate requests and certificate download
    //    requests for authorization and successor enrollment certificates. The RA may also restrict the EE from other
    //    activities. In this case, the EE’s enrollment certificate is referred to as a blocked enrollment certificate.
    if is_ee_blocked {
        return Err(errors::HandleResponseError::new(
            "Blocked enrollment certificate",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                errors::BadRequestErrorCodes::BlockedEnrollmentCertificate,
            ),
        ));
    }

    // /POST Throw Error- 400-78: Outside ra-maxGenDelay generationTime
    // Maximum time between generationTime in the
    // EeRaCertRequest and the requested start time of the
    // certificates (requested using the validityPeriod field in
    // the tbsCert field).
    // The time difference between EeRaCertRequest.generationTime and
    // EeRaCertRequest.tbsCert.validityPeriod.start shall not be greater than the
    // ra-maxGenDelay.
    let ra_max_gen_delay = GlobalConfig::global().param_ra_max_gen_delay as u32;
    if generation_time - certificate_validity_start > ra_max_gen_delay {
        log::debug!(
            "Failed to check generation time: Outside ra-maxGenDelay generationTime: {:?} certificate_validity_start: {:?} ra_max_gen_delay: {:?}",
            generation_time,
            certificate_validity_start,
            ra_max_gen_delay
        );
        return Err(errors::HandleResponseError::new(
            "Outside ra-maxGenDelay generationTime",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                errors::BadRequestErrorCodes::OutsideRaMaxGenDelayGenerationTime,
            ),
        ));
    }

    let mut exp_type = scmscommon::ExpansionType::NonButterfly;
    let mut non_butterfly_request = None;
    if caterpillar.is_some() {
        // Store caterpillar
        let caterpillar_instance = caterpillar.ok_or_else(|| {
            errors::HandleResponseError::new(
                "Caterpillar instance is None",
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })?;

        caterpillar_instance.store(db).await.map_err(|e| {
            errors::HandleResponseError::new(
                &format!("Failed to store caterpillar: {}", e),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })?;
        exp_type = *caterpillar_instance.get_exp_type();
        log::debug!("Caterpillar {} stored successfully", exp_type);
    } else {
        log::debug!("Processing non butterfly request");
        non_butterfly_request = Some(non_butterfly.unwrap());
    }

    if let Some(celery_app) = celery_app {
        celery_app
            .send_task(run_authorization_request_processing::new(
                exp_type,
                non_butterfly_request,
            ))
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

    // Update status to provisioning
    patch_status_ee_registration_by_device_id(vid, Status::Provisioning, db).await?;

    // Return ack
    log::info!("Caterpillar {} processed successfully", exp_type);
    Ok(ack_response)
}

async fn certificate_request_ack_response(
    payload_encoded: Vec<u8>,
    db: &DatabaseConnection,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    log::debug!("Preparing certificate request ack response");
    let delta_next_dl_time = GlobalConfig::global().param_cert_next_dl_time as u32;
    let ra_private = latest_ra_private_key(db).await?;
    let ra_certificate = latest_ra_certificate(db).await?;

    let encoded_ack = match oscms_bridge::encode_certificate_ack(
        payload_encoded,
        delta_next_dl_time,
        ra_private,
        ra_certificate,
    ) {
        Ok(encoded_ack) => encoded_ack,
        Err(x) => return Err(oscms_error_to_ra_response_error(x)),
    };

    log::info!("Returning request ack response!");
    Ok(encoded_ack)
}

async fn handle_get_request(
    ieee1609dot2_authorization_encoded: Option<String>,
    query_param_filename: Option<String>,
    app_ip: String,
    db: &DatabaseConnection,
    time_request_received: u32,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    log::info!("Starting Handle GET request for authorization certificate.");
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
        RaRequestManagementRequestType::AuthorizationDownload,
        db,
    )
    .await?;

    // Extracting request hash and period valid
    let (request_hash_id, i_value) = parse_and_verify_request_hash_and_i_value(
        ieee1609dot2_authorization_encoded,
        query_param_filename.clone(),
        time_request_received,
        db,
    )
    .await?;

    // Fetching certificates and build zip file
    fetch_certificates_and_build_zip(db, request_hash_id, i_value).await
}

async fn parse_and_verify_request_hash_and_i_value(
    ieee1609dot2_authorization_encoded: Option<String>,
    query_param_filename: Option<String>,
    time_request_received: u32,
    db: &DatabaseConnection,
) -> Result<(String, u64), errors::HandleResponseError> {
    // From filename
    let mut request_hash_id_from_param: Option<String> = None;
    let mut request_i_value_from_param: Option<u64> = None;
    if let Some(query_filename) = query_param_filename {
        let (request_hash_id, i_value) =
            extract_hash_id_from_ieee1609dot2_filename(query_filename.clone())?;
        request_hash_id_from_param = Some(request_hash_id);
        request_i_value_from_param = Some(i_value);
        log::debug!(
            "Request hash_id from param: {} and i_value: {}",
            request_hash_id_from_param.clone().unwrap_or_default(),
            request_i_value_from_param.unwrap_or_default()
        );
    }

    // From Spdu
    let mut request_hash_id_from_spdu: Option<String> = None;
    let mut request_i_value_from_spdu: Option<u64> = None;
    if let Some(encoded) = ieee1609dot2_authorization_encoded {
        let raw_request_hash_id_from_spdu =
            decode_ieee1609dot2_authorization(encoded, time_request_received, db).await?;

        let (request_hash_id, i_value) =
            extract_hash_id_from_ieee1609dot2_filename(raw_request_hash_id_from_spdu)?;

        log::debug!(
            "Extracted hash_id from spdu: {} and i_value: {}",
            request_hash_id,
            i_value
        );

        request_hash_id_from_spdu = Some(request_hash_id);
        request_i_value_from_spdu = Some(if i_value == 0 {
            get_current_time_period_days() as u64
        } else {
            i_value
        });

        log::debug!(
            "Request hash_id from spdu: {} and i_value: {}",
            request_hash_id_from_spdu.clone().unwrap_or_default(),
            request_i_value_from_spdu.unwrap_or_default()
        );
    }

    // Check when both are present
    if let (Some(hash_id_param), Some(hash_id_spdu)) = (
        request_hash_id_from_param.clone(),
        request_hash_id_from_spdu.clone(),
    ) {
        if hash_id_param != hash_id_spdu {
            log::error!("Filename mismatch: {} != {}", hash_id_param, hash_id_spdu);
            return Err(errors::HandleResponseError::new(
                "Filename mismatch",
                StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
            ));
        }
    }

    // Check when none are present
    if request_hash_id_from_param.is_none() && request_hash_id_from_spdu.is_none() {
        log::error!("Filename not found");
        return Err(errors::HandleResponseError::new(
            "Filename not found",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
        ));
    }

    // When filename is present, use it as priority
    if let (Some(hash_id_param), Some(i_value_param)) =
        (request_hash_id_from_param, request_i_value_from_param)
    {
        log::debug!(
            "Using request hash_id {} and i_value: {} from param",
            hash_id_param,
            i_value_param
        );
        return Ok((hash_id_param, i_value_param));
    }

    if let (Some(hash_id_spdu), Some(i_value_spdu)) =
        (request_hash_id_from_spdu, request_i_value_from_spdu)
    {
        log::debug!(
            "Using request hash_id {} and i_value: {} from spdu",
            hash_id_spdu,
            i_value_spdu
        );
        return Ok((hash_id_spdu, i_value_spdu));
    }

    unreachable!("One of the branches should have returned a value");
}

fn extract_hash_id_from_ieee1609dot2_filename(
    filename: String,
) -> Result<(String, u64), errors::HandleResponseError> {
    log::debug!("Extracting hash_id from query param filename: {}", filename);

    let parts: Vec<&str> = filename.split("_").collect();
    if parts.len() == 1 {
        log::debug!("Request filename from non-butterfly spdu: {}", filename);
        // non-butterfly: <hash_id>.zip

        let filename_parts: Vec<&str> = filename.split(".").collect();
        if filename_parts.len() != 2 {
            // Throw /GET Errpr-Code 400-42 failed parsing
            return Err(errors::HandleResponseError::new(
                "Failed to parse request hash_id from spdu",
                StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
            ));
        }
        // ignoring .zip
        let hash_id = filename_parts[0].to_string();
        let i_value = get_current_time_period_days() as u64;
        log::debug!(
            "Extracted hash_id from spdu: {} and i_value: {}",
            hash_id,
            i_value
        );

        return Ok((hash_id, i_value));
    } else if parts.len() != 2 {
        // Throw /GET Errpr-Code 400-42 failed parsing
        return Err(errors::HandleResponseError::new(
            "Failed to parse request hash_id from spdu",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
        ));
    }

    // butterfly: <hash_id>_<i_value>.zip
    log::debug!("Request filename from butterfly spdu: {}", filename);

    // Define the regular expression to match the expected pattern
    let re = Regex::new(r"(?P<hash_id>[0-9a-fA-F]+)_(?P<i_value>[0-9a-fA-F]+)\.zip").unwrap();

    if let Some(captures) = re.captures(filename.as_str()) {
        // Extract the hash_id
        let hash_id = match captures.name("hash_id") {
            Some(hash_id) => hash_id.as_str(),
            None => {
                log::debug!("Unable to extract hash_id from {}", filename);
                // Throw /GET Errpr-Code 400-42 failed parsing
                return Err(errors::HandleResponseError::new(
                    "Invalid hash id from filename",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                        errors::BadRequestErrorCodes::FailedParsing,
                    ),
                ));
            }
        };
        // Extract and parse the i_value as u64
        let i_value_str = match captures.name("i_value") {
            Some(i_value) => i_value.as_str(),
            None => {
                log::debug!("Unable to extract i_value_str from {}", filename);
                // Throw /GET Errpr-Code 400-42 failed parsing
                return Err(errors::HandleResponseError::new(
                    "i-value from filename not found",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                        errors::BadRequestErrorCodes::FailedParsing,
                    ),
                ));
            }
        };
        // Parse the i_value string as a u64, handling both decimal and hexadecimal formats
        if i_value_str.chars().any(|c| c.is_alphabetic()) {
            match u64::from_str_radix(i_value_str, 16).ok() {
                Some(i_value) => {
                    let mut i_value_ret = i_value;
                    if i_value == 0 {
                        i_value_ret = get_current_time_period_days() as u64;
                    }
                    return Ok((hash_id.to_owned(), i_value_ret));
                }
                None => {
                    log::debug!("Unable to cast i_value_str to u64 {}", i_value_str);
                    // Throw /GET Errpr-Code 400-42 failed parsing
                    return Err(errors::HandleResponseError::new(
                        "Invalid hex i-value from filename",
                        StatusCode::BAD_REQUEST,
                        Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                            errors::BadRequestErrorCodes::FailedParsing,
                        ),
                    ));
                }
            }
        } else {
            match i_value_str.parse::<u64>().ok() {
                Some(i_value) => {
                    let mut i_value_ret = i_value;
                    if i_value == 0 {
                        i_value_ret = get_current_time_period_days() as u64;
                    }
                    return Ok((hash_id.to_owned(), i_value_ret));
                }
                None => {
                    log::debug!("Unable to cast i_value_str to u64 {}", i_value_str);
                    // Throw /GET Errpr-Code 400-42 failed parsing
                    return Err(errors::HandleResponseError::new(
                        "Invalid dec i-value from filename",
                        StatusCode::BAD_REQUEST,
                        Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                            errors::BadRequestErrorCodes::FailedParsing,
                        ),
                    ));
                }
            }
        };
    }
    // Throw /GET Errpr-Code 400-42 failed parsing
    Err(errors::HandleResponseError::new(
        "Unable to extract hash_id and i_value from filename",
        StatusCode::BAD_REQUEST,
        Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
    ))
}

async fn fetch_certificates_and_build_zip(
    db: &DatabaseConnection,
    request_hash_id: String,
    i_value: u64,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    log::debug!(
        "Fetching certificates and building zip for hash_id: {} and i_value: {}",
        request_hash_id,
        i_value
    );
    // 1. Fetch Certificate Files by Hash ID and current time period
    let (certificate_files, is_butterfly_request) =
        fetch_certificates_files_by_hash_id_and_i_value(db, request_hash_id.clone(), i_value)
            .await?;

    log::debug!("Found {} certificates", certificate_files.len());

    // 2. Check if there are any certificates
    if certificate_files.is_empty() {
        return Err(errors::HandleResponseError::new(
            "No certificates found",
            StatusCode::NOT_FOUND,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    let mut x_dot_info_file_id = 0;
    let mut x_dot_info_file = Vec::new();

    if is_butterfly_request {
        // 1.1 Fetch x.info file by hashId and current time period
        (x_dot_info_file_id, x_dot_info_file) =
            fetch_x_info_file_by_hash_id_and_i_value(db, request_hash_id, i_value).await?;

        log::debug!("Found x.info file: size {}", x_dot_info_file.len());
    }

    // 3. Create an in-memory zip archive
    let hash_id = &certificate_files[0].hash_id.clone();
    let mut output_zip_name = format!("{}.zip", hash_id.to_uppercase());
    if is_butterfly_request {
        let i_index = certificate_files[0].index_i;
        let dir_name = format!("{}-{:x}", hash_id, i_index);
        output_zip_name = format!("{}.zip", dir_name);
    }

    let mut certificates_ids = Vec::new();
    let mut zip_buffer = Vec::new();
    {
        let mut zip_writer = ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));
        log::debug!("Started zip file: {}", output_zip_name);

        if is_butterfly_request {
            // 3.0 Add x.info file to in-memory zip archive
            let file_path = format!("{:x}.info", i_value);
            write_file_to_zip(file_path, x_dot_info_file, &mut zip_writer)?;
        }

        // 3.1. Add all certificates to the in-memory zip archive
        for certificate_file in certificate_files {
            let mut file_path = hash_id.to_uppercase();
            if is_butterfly_request {
                file_path = format!(
                    "{:x}_{:x}",
                    certificate_file.index_i, certificate_file.index_j
                );
            }
            write_file_to_zip(
                file_path,
                certificate_file.certificate_binary,
                &mut zip_writer,
            )?;
            certificates_ids.push(certificate_file.id);
        }

        zip_writer.finish().map_err(|e| {
            errors::HandleResponseError::new(
                format!("Failed to finish zip: {}", e).as_str(),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })?;
    }

    // Update certificates and x.info downloaded to true
    if is_butterfly_request {
        update_x_dot_info_set_downloaded_true(db, x_dot_info_file_id).await?;
    }
    update_set_downloaded_true(db, certificates_ids.clone()).await?;

    log::debug!("Zip file {} created successfully", output_zip_name);
    Ok((zip_buffer, output_zip_name))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::certificate_store::Model as CertificateStoreModel;
    use crate::entities::x_dot_info_store::Model as XDotInfoStoreModel;
    use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase, MockExecResult};
    use std::io::{Cursor, Read};
    use zip::read::ZipArchive;

    #[tokio::test]
    async fn test_parse_and_verify_request_hash_and_i_value() {
        let db = mock_db();
        // 1. From filename only
        let query_param_filename: Option<String> = Some("df64afcb3a3edcf3_3.zip".to_string());
        let ieee1609dot2_authorization_encoded: Option<String> = None;
        let result = parse_and_verify_request_hash_and_i_value(
            ieee1609dot2_authorization_encoded,
            query_param_filename.clone(),
            0,
            &db,
        )
        .await;
        assert!(result.is_ok());
        let (request_hash_id, i_value) = result.unwrap();
        assert_eq!(request_hash_id, "df64afcb3a3edcf3");
        assert_eq!(i_value, 3);
    }

    #[test]
    fn test_extract_hash_id_from_ieee1609dot2_filename() {
        // current time period days
        let current_time_seconds = get_current_time_seconds() as u64;
        let current_time_period_days: u64 = current_time_seconds / (60 * 60 * 24);

        // 1. Valid filename
        let filename = "df64afcb3a3edcf3_3.zip".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_ok());
        let (hash_id, i_value) = result.unwrap();
        assert_eq!(hash_id, "df64afcb3a3edcf3");
        assert_eq!(i_value, 3);

        // 2. Valid filename with hex i_value
        let filename = "df64afcb3a3edcf3_3f.zip".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_ok());
        let (hash_id, i_value) = result.unwrap();
        assert_eq!(hash_id, "df64afcb3a3edcf3");
        assert_eq!(i_value, 63);

        // 3. Invalid filename
        let filename = "df64afcb3a3edcf3_3".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_err());

        // 4. non-butterfly
        let filename = "df64afcb3a3edcf3.zip".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_ok());
        let (hash_id, i_value) = result.unwrap();
        assert_eq!(hash_id.to_lowercase(), "df64afcb3a3edcf3");
        assert_eq!(i_value, current_time_period_days);

        // 5. Invalid hex i_value
        let filename = "df64afcb3a3edcf3_3g.zip".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_err());

        // 6. Invalid dec i_value
        let filename = "df64afcb3a3edcf3_g.zip".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_err());

        // 7. 00 ivalue
        let filename = "df64afcb3a3edcf3_00.zip".to_string();
        let result = extract_hash_id_from_ieee1609dot2_filename(filename);
        assert!(result.is_ok());
        let (hash_id, i_value) = result.unwrap();
        assert_eq!(hash_id, "df64afcb3a3edcf3");
        assert_eq!(i_value, current_time_period_days);
    }

    fn mock_db() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([[
                CertificateStoreModel {
                    id: 1,
                    hash_id: "81ddacb72a977c2f".to_string(),
                    index_i: 3,
                    index_j: 3,
                    file: vec![3; 16],
                    downloaded: 0,
                },
                CertificateStoreModel {
                    id: 2,
                    hash_id: "81ddacb72a977c2f".to_string(),
                    index_i: 3,
                    index_j: 4,
                    file: vec![4; 16],
                    downloaded: 0,
                },
            ]])
            .append_query_results([[XDotInfoStoreModel {
                id: 1,
                hash_id: "81ddacb72a977c2f".to_string(),
                current_i: 3,
                file: vec![7; 16],
                downloaded: 0,
            }]])
            .append_exec_results([MockExecResult {
                last_insert_id: 1,
                rows_affected: 1,
            }])
            .append_exec_results([
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 2,
                    rows_affected: 1,
                },
            ])
            .into_connection()
    }

    #[tokio::test]
    async fn test_fetch_certificates_and_build_zip() {
        // let mock database
        let db = mock_db();
        match fetch_certificates_and_build_zip(&db, "81ddacb72a977c2f".to_string(), 3).await {
            Ok((zip_buffer, output_zip_name)) => {
                assert_eq!(output_zip_name, "81ddacb72a977c2f-3.zip");
                // Load the zip archive from the buffer
                let cursor = Cursor::new(zip_buffer);
                let mut zip = ZipArchive::new(cursor).expect("Failed to read zip archive");

                // Verify the number of files in the zip archive
                assert_eq!(zip.len(), 3);

                // Verify each file's contents
                for i in 0..zip.len() {
                    let mut file = zip
                        .by_index(i)
                        .expect("Failed to access file in zip archive");

                    // Check file names
                    match i {
                        0 => {
                            assert_eq!(file.name(), "3.info");
                        }
                        1 => {
                            assert_eq!(file.name(), "3_3");
                        }
                        2 => {
                            assert_eq!(file.name(), "3_4");
                        }
                        _ => panic!("Unexpected file in zip archive"),
                    }

                    // Check file contents
                    let mut contents = Vec::new();
                    file.read_to_end(&mut contents)
                        .expect("Failed to read file in zip archive");
                    match i {
                        0 => {
                            assert_eq!(contents, vec![7; 16]);
                        }
                        1 => {
                            assert_eq!(contents, vec![3; 16]);
                        }
                        2 => {
                            assert_eq!(contents, vec![4; 16]);
                        }
                        _ => panic!("Unexpected file contents in zip archive"),
                    }
                }
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }
    }
}
