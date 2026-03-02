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
use scmscommon::{
    AppState,
    endpoint_error_codes::{BadRequestErrorCodes, Ieee1609Dot2Dot1ErrorCodes},
    errors,
};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;
use zip::ZipWriter;

use crate::persistence::ccf_store::fetch_ccf_by_ctl_series_id;
use crate::persistence::ctl_store::fetch_ctl_file_by_ctl_series_id;
use crate::persistence::ra_certificates::{fetch_latest_file_by_cert_id, latest_ra_certificate};

use crate::ra_endpoint::common::validate_certificate_id::validate_certificate_id;
use crate::ra_endpoint::common::zip_helpers::write_file_to_zip;
// use oscms_bridge::make_certificate_chain_file_encoded;

///
/// 1609.2.1 SS 6.3.5.12 Download RA Certificate
///
/// Download the RA Certificate
#[utoipa::path(
  get,
  tag = "Certificate Downloads",
  path = "ra-certificate",
  context_path = "/v3/",
  params(
    ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
  ),
  responses(
    (status = 200, body = [u8], description = "Certificate raCert" ),
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
    (status = 416, description = "requested range not satisfiable: retry without HTTP Range"),
    (status = 429, description = "Too many requests"),
    (status = 500, description = "Internal server error"),
    (status = 503, description = "Service unavailable")
  )
)]
#[actix_web::get("/ra-certificate")]
pub async fn download_ra_certificate(
    data: web::Data<AppState>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    log::debug!("Received GET request for RA certificate");

    // Get Authorization token from header
    let _authorization_token = req
        .headers()
        .get("Authorization")
        .map(|x| x.to_str().unwrap().to_string());

    // TODO: Validate Authorization token
    // match validate_authorization_bearer_token(authorization_token).await {
    //   Ok(x) => HttpResponseBuilder::new(StatusCode::ACCEPTED).body(x),
    //   Err(x) => x.http_error_response(),
    // }

    // Return Ra certificate
    match handle_get_ra_certificate(&data.db).await {
        Ok(x) => HttpResponseBuilder::new(StatusCode::ACCEPTED).body(x),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_get_ra_certificate(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    latest_ra_certificate(db).await.map_err(|e| {
        errors::HandleResponseError::new(
            format!("Failed to fetch RA Certificate: {}", e).as_str(),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    })
}

/// Certificate Id - 16 digit hexadecimal value
#[derive(Deserialize, ToSchema, utoipa::IntoParams, Validate)]
pub struct CertificateId {
    #[validate(
        length(equal = 16, message = "certId must be exactly 16 characters"),
        custom(
            function = "validate_certificate_id",
            message = "certId must be a valid hexadecimal value"
        )
    )]
    #[serde(rename = "certId")]
    certificate_id: String,
}

/// 1609.2.1 SS 6.3.5.9 Download Individual CA Certificate
///
/// Download the CA certificate specified by the certId query parameter.
#[utoipa::path(
  get,
  tag = "Certificate Downloads",
  path = "ca-certificate",
  context_path = "/v3/",
  params(
    (
      "certId" = String,
      Query,
      description = "Certificate Id. 16 character hex string",
      min_length = 16,
      max_length=16,
      example = "0123456789ABCDEF",
      pattern = "^[a-fA-F0-9]{16}$",
    ),
    ("Authorization", Header, description = "Bearer [access-token/AT]"),
  ),
  responses(
    (
        status = 200,
        body = [u8], description = "C-OER encoded CA Certificate file",
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
#[actix_web::get("/ca-certificate")]
pub async fn download_ca_certificate(
    data: web::Data<AppState>,
    query_params: web::Query<CertificateId>,
) -> impl Responder {
    log::debug!("Received GET request for CA certificate");

    match query_params.validate() {
        Ok(_) => {}
        Err(x) => return HttpResponse::BadRequest().json(x),
    }

    log::debug!(
        "Downloading CA certificate with certId {}",
        query_params.certificate_id.clone()
    );

    // Return Ca certificate
    match handle_get_ca_certificate(&data.db, query_params.certificate_id.clone()).await {
        Ok(x) => HttpResponseBuilder::new(StatusCode::ACCEPTED).body(x),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_get_ca_certificate(
    db: &DatabaseConnection,
    cert_id: String,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    fetch_latest_file_by_cert_id(cert_id.clone(), db)
        .await
        .map_err(|e| {
            errors::HandleResponseError::new(
                format!(
                    "Failed to fetch CA Certificate (cert_id = {}): {}",
                    cert_id, e
                )
                .as_str(),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams, Validate, Debug)]
pub struct CtlSeriesQueryParams {
    #[validate(
        length(equal = 16, message = "CTL Series Id must be exactly 16 characters"),
        custom(
            function = "validate_certificate_id",
            message = "CTL Series Id must be a valid hexadecimal value"
        )
    )]
    #[serde(rename = "ctlSeriesId")]
    ctl_series_id: String,
    #[serde(rename = "ctlSequenceNumber")]
    ctl_sequence_number: Option<u16>,
}

/// 1609.2.1 SS 6.3.5.11 Download Certificate Trust List (CTL)
///
/// Download the specified CTL
#[utoipa::path(
  get,
  tag = "Certificate Downloads",
  path = "ctl",
  context_path = "/v3/",
  params(
    (
      "ctlSeriesId" = String,
      Query,
      description = "Id ofr the required CTL Series. 16 character hex string",
      min_length = 16,
      max_length=16,
      example = "0123456789ABCDEF",
      pattern = "^[a-fA-F0-9]{16}$",
    ),
    (
      "ctlSequenceNumber" = Option<u16>,
      Query,
      description = "Optional sequence number of the requested CTL",
      minimum = 0,
      maximum = 65535
    ),
    ("Authorization", Header, description = "Bearer [access-token/AT]"),
  ),
  responses(
    (
      status = 200,
      body = [u8], description = "It is a zip file The content is a single
      directory containing individual CTL files. The content of each individual
      CTL file is a C-OER encoded MultiSignedCtlSpdu",
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
#[actix_web::get("/ctl")]
pub async fn download_certificate_trust_list(
    data: web::Data<AppState>,
    query_params: web::Query<CtlSeriesQueryParams>,
    _req: actix_web::HttpRequest,
) -> impl Responder {
    log::debug!("Received GET request for CTL Series {:#?}", query_params);

    match query_params.validate() {
        Ok(_) => {}
        Err(x) => return HttpResponse::BadRequest().json(x),
    }

    let result = handle_get_ctl(query_params, &data.db).await;
    match result {
        // The response depends on whether or not the query parameter ctlSequenceNumber was provided
        // in the request:
        // If the ctlSequenceNumber was provided in the request, a CTL file as defined in 8.6,
        // where the multiSignedCtl.tbsCtl.FullIeeeTbsCtl.ctlSeriesId shall
        // be equal to the query parameter ctlSeriesId and
        // multiSignedCtl.tbsCtl.FullIeeeTbsCtl.sequenceNumber shall be
        // equal to the query parameter ctlSequenceNumber provided in the request path.
        // If the ctlSequenceNumber was not provided in the request, a CTL file as defined in 8.6,
        // where the multiSignedCtl.tbsCtl.FullIeeeTbsCtl.ctlSeriesId shall
        // be equal to the query parameter ctlSeriesId provided in the request path.
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

async fn handle_get_ctl(
    query_params: web::Query<CtlSeriesQueryParams>,
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    let ctl_series_id = &query_params.ctl_series_id;
    let ctl_sequence_number = query_params.ctl_sequence_number;

    log::debug!(
        "Handling GET request for CTL Series {} with sequence number {:?}",
        ctl_series_id,
        ctl_sequence_number
    );

    fetch_ctl_and_build_zip(db, ctl_series_id.to_string(), ctl_sequence_number).await
}

async fn fetch_ctl_and_build_zip(
    db: &DatabaseConnection,
    ctl_series_id: String,
    ctl_sequence_number: Option<u16>,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    let (ctl_files, min_sequence_number) =
        fetch_ctl_file_by_ctl_series_id(ctl_series_id.to_string(), ctl_sequence_number, db).await?;

    log::debug!("Fetched CTL files: {:?}", ctl_files.len());
    if ctl_files.is_empty() {
        return Err(errors::HandleResponseError::new(
            "No CTL files found for the specified CTL Series Id and sequence number",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::FailedParsing),
        ));
    }

    // Create zip file
    create_ctl_zip(ctl_files, ctl_series_id.clone(), min_sequence_number)
}

fn create_ctl_zip(
    ctl_files: Vec<(Vec<u8>, String)>,
    ctl_series_id: String,
    min_sequence_number: i32,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    log::debug!("Creating CTL zip file");
    // 8.6 CTL files
    // The CTL file is used to respond to a CTL download request as specified in 6.3.5.11. It is a zip file without
    // compression that follows the PKWare Zip File Format specification version 6.3.6. The content is a single
    // directory containing individual CTL files. The content of each individual CTL file is a C-OER encoded
    // MultiSignedCtlSpdu. Each CTL file name follows the convention ctl_ctlSeriesId_sequenceNumber.oer.
    // The zip file name is ctl_ctlSeriesId_sequenceNumber.zip, where sequenceNumber is the lowest sequence
    // number value among the included CTLs. The zip file contains all the CTLs for that CtlSeriesId value,
    // from the one with the sequence number indicated in the filename to the most recent one known to the
    // generator of the file.
    let output_zip_name = format!("ctl_{}_{}.zip", ctl_series_id, min_sequence_number);
    let mut zip_buffer = Vec::new();
    {
        let mut zip_writer = ZipWriter::new(std::io::Cursor::new(&mut zip_buffer));
        log::debug!("Started zip file: {}", output_zip_name);

        // Add all ctl files to the in-memory zip archive
        for ctl_file in ctl_files {
            write_file_to_zip(ctl_file.1, ctl_file.0, &mut zip_writer)?;
        }

        zip_writer.finish().map_err(|e| {
            errors::HandleResponseError::new(
                format!("Failed to finish zip: {}", e).as_str(),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })?;
    }

    log::debug!("Zip file {} created successfully", output_zip_name);
    Ok((zip_buffer, output_zip_name))
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams, Validate, Debug)]
pub struct CcfCtlQueryParams {
    #[validate(
        length(equal = 16, message = "CTL Series Id must be exactly 16 characters"),
        custom(
            function = "validate_certificate_id",
            message = "CTL Series Id must be a valid hexadecimal value"
        )
    )]
    #[serde(rename = "ctlSeriesId")]
    ctl_series_id: String,
}

/// 1609.2.1 SS 6.3.5.7 Download CCF including CTL
///
/// Download the CCF
#[utoipa::path(
  get,
  tag = "Certificate Downloads",
  path = "ccf-ctl",
  context_path = "/v3/",
  params(
    (
      "ctlSeriesId" = String,
      Query,
      description = "Id of the required CTL Series. 16 character hex string",
      min_length = 16,
      max_length=16,
      example = "0123456789ABCDEF",
      pattern = "^[a-fA-F0-9]{16}$",
    ),
    ("Authorization", Header, description = "Bearer [access-token/AT]"),
  ),
  responses(
    (
      status = 200,
      body = [u8], description = "File containing a CertificateChainSpdu",
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
#[actix_web::get("/ccf-ctl")]
pub async fn download_certificate_chain_file(
    data: web::Data<AppState>,
    query_params: web::Query<CcfCtlQueryParams>,
) -> impl Responder {
    log::debug!(
        "Received GET request for CCF-CTL Series {:#?}",
        query_params
    );

    match query_params.validate() {
        Ok(_) => {}
        Err(x) => return HttpResponse::BadRequest().json(x),
    }

    let result = handle_get_ccf(&data.db, &query_params.ctl_series_id).await;
    match result {
        // The RA maintains a certificate chain file to be provided to its client end entities. The version of the file is
        // updated whenever the file is updated and is indicated in the filename. The contents of the file are a C-OER
        // encoded CertificateChainSpdu.
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

pub async fn handle_get_ccf(
    db: &DatabaseConnection,
    ctl_series_id: &String,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    log::debug!(
        "Handling GET request for CCF download ctl_series_id {}",
        ctl_series_id,
    );

    // Try fetch CCF from storage
    log::debug!("Fetching CCF file");
    let (ccf_encoded, version) = fetch_ccf_by_ctl_series_id(ctl_series_id.to_string(), db).await?;
    if ccf_encoded.is_empty() {
        return Err(errors::HandleResponseError::new(
            "No CCF file found",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::FailedParsing),
        ));
    }

    let filename = format!("ccf_{}.oer", version);
    log::debug!("Found CCF file: {} version {}", filename, version);
    Ok((ccf_encoded, filename))
}
