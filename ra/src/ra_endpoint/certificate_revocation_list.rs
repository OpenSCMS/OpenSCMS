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

use crate::persistence::composite_crl_store::{
    fetch_composite_crl_by_ctl_series_id, store_composite_crl,
};
use crate::persistence::crl_store::{
    fetch_all_crls, fetch_crl_by_crl_series_and_or_cracaid, store_crl,
};
use crate::persistence::ctl_store::fetch_ctl_file_by_ctl_series_id;
use crate::persistence::ra_certificates::{latest_ra_certificate, latest_ra_private_key};
use crate::ra_endpoint::common::oscms_error_to_ra_response_error;
use crate::ra_endpoint::common::validate_certificate_id::validate_certificate_id;
use actix_web::{HttpResponse, HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::{AppState, errors};
use sea_orm::DatabaseConnection;
use serde::Deserialize;
use utoipa::ToSchema;
use validator::Validate;

/// 1609.2.1 SS 6.3.5.8 Composite CRL including CTL download; table 17
#[utoipa::path(
  get,
  tag = "Composite CRL including CTL download",
  path = "composite-crl-ctl",
  context_path = "/v3/",
  params(
    ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
    (
      "ctlSeriesId" = String,
      Query,
      description = "Id of the required CTL Series. 16 character hex string",
      min_length = 16,
      max_length=16,
      example = "0123456789ABCDEF",
      pattern = "^[a-fA-F0-9]{16}$",
    ),
  ),
  responses(
    (status = 200, body = [u8], description = "Composite CRL file" ),
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
#[actix_web::get("/composite-crl-ctl")]
pub async fn composite_crl_ctl(
    data: web::Data<AppState>,
    query_params: web::Query<CompositeCrlCtlQueryParam>,
) -> impl Responder {
    log::debug!(
        "Received GET request for Composite CRL with CTL {:#?}",
        query_params
    );

    match query_params.validate() {
        Ok(_) => {}
        Err(x) => return HttpResponse::BadRequest().json(x),
    }

    let result = handle_composite_crl_ctl(&data.db, query_params).await;
    match result {
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

#[derive(Deserialize, ToSchema, utoipa::IntoParams, Validate, Debug)]
pub struct CompositeCrlCtlQueryParam {
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

async fn handle_composite_crl_ctl(
    db: &DatabaseConnection,
    query_params: web::Query<CompositeCrlCtlQueryParam>,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    let ctl_series_id = &query_params.ctl_series_id;
    log::debug!(
        "Handling GET request for download of Composite CRL with CTL ctl_series_id {}",
        ctl_series_id,
    );

    // Fetch Composite CRL for that ctl_series_id
    let (composite_crl_ctl_encoded, filename) =
        fetch_composite_crl_by_ctl_series_id(ctl_series_id.clone(), db).await?;

    // If empty so we need to build it
    if composite_crl_ctl_encoded.is_empty() {
        let (new_composite_crl_ctl_encoded, filename) =
            build_and_store_composite_crl(ctl_series_id.clone(), db).await?;
        return Ok((new_composite_crl_ctl_encoded, filename));
    }

    Ok((composite_crl_ctl_encoded, filename))
}

pub async fn build_and_store_composite_crl(
    ctl_series_id: String,
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    // Build Composite CRL encoded
    log::debug!(
        "Building Composite CRL file: ctl_series_id {}",
        ctl_series_id
    );

    // Fetch all available CRL
    let mut crl_list = fetch_all_crls(db).await?;

    // if no CRLs are available, we need to create it
    if crl_list.is_empty() {
        let new_crl = build_and_store_crl(db).await?;
        crl_list.push(new_crl);
    }

    // Fetch ctl by ctl_series_id
    let (mut ctl_files, _) =
        fetch_ctl_file_by_ctl_series_id(ctl_series_id.to_string(), None, db).await?;

    // If empty, so it's an error situation
    if ctl_files.is_empty() {
        return Err(errors::HandleResponseError::new(
            "CTL file not found",
            StatusCode::INTERNAL_SERVER_ERROR,
            scmscommon::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    if ctl_files.len() > 1 {
        return Err(errors::HandleResponseError::new(
            "Multiple CTL files found",
            StatusCode::INTERNAL_SERVER_ERROR,
            scmscommon::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    let ctl_file = ctl_files.pop().unwrap().0;

    // First composite CRL, so version 1
    let crl_version = 1;

    let (composite_crl_encoded, filename) =
        oscms_bridge::make_composite_crl_encoded(ctl_file, crl_list, crl_version)
            .map_err(oscms_error_to_ra_response_error)?;

    // Store the new Composite CRL
    store_composite_crl(
        db,
        ctl_series_id,
        filename.clone(),
        composite_crl_encoded.clone(),
    )
    .await?;

    Ok((composite_crl_encoded, filename))
}

/// 1609.2.1 SS 6.3.5.10 Individual CRL download; table 19
///
/// Download the RA Certificate
#[utoipa::path(
  get,
  tag = "Individual CRL download",
  path = "crl",
  context_path = "/v3/",
  params(
    ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
    (
      "craca" = String,
      Query,
      description = "Id of the required CRL Series. 16 character hex string",
      min_length = 16,
      max_length=16,
      example = "0123456789ABCDEF",
      pattern = "^[a-fA-F0-9]{16}$",
    ),
    (
      "crlSeries" = String,
      Query,
      description = "Id of the required CRL Series. 16 character hex string",
      min_length = 16,
      max_length=16,
      example = "0123456789ABCDEF",
      pattern = "^[a-fA-F0-9]{16}$",
    ),
  ),
  responses(
    (status = 200, body = [u8], description = "Individual CRL file" ),
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
#[actix_web::get("/crl")]
pub async fn individual_crl_download(
    data: web::Data<AppState>,
    query_params: web::Query<IndividualCrlQueryParam>,
) -> impl Responder {
    log::debug!(
        "Received GET request for individual CRL download {:#?}",
        query_params
    );

    match query_params.validate() {
        Ok(_) => {}
        Err(x) => return HttpResponse::BadRequest().json(x),
    }

    let result = handle_individual_crl_download(&data.db, query_params).await;
    match result {
        Ok(payload) => HttpResponseBuilder::new(StatusCode::ACCEPTED)
            .append_header(("Content-Type", "application/octet-stream"))
            .body(payload),
        Err(x) => x.http_error_response(),
    }
}

#[derive(Deserialize, ToSchema, utoipa::IntoParams, Validate, Debug)]
pub struct IndividualCrlQueryParam {
    #[validate(
        length(equal = 16, message = "craca must be exactly 16 characters"),
        custom(
            function = "validate_certificate_id",
            message = "craca must be a valid hexadecimal value"
        )
    )]
    #[serde(rename = "craca")]
    craca: String,

    #[serde(rename = "crlSeries")]
    crl_series: u16,
}

async fn handle_individual_crl_download(
    db: &DatabaseConnection,
    query_params: web::Query<IndividualCrlQueryParam>,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    let craca = &query_params.craca;
    let crl_series = &query_params.crl_series;

    log::debug!(
        "Handling GET request for Individual CRL download: craca {} crl_series {}",
        craca,
        crl_series
    );

    // Fetch CRL for that craca and crl_series
    let individual_crl_encoded =
        fetch_crl_by_crl_series_and_or_cracaid(Some(*crl_series as u32), Some(craca.clone()), db)
            .await?;

    // If empty so we need to build it
    if individual_crl_encoded.is_empty() {
        log::debug!("Individual CRL not found, building new one");
        let new_individual_crl_encoded = build_and_store_crl(db).await?;
        return Ok(new_individual_crl_encoded);
    }

    log::debug!("Returning Individual CRL");
    Ok(individual_crl_encoded)
}

async fn build_and_store_crl(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, errors::HandleResponseError> {
    log::debug!("Building New CRL file");

    // TODO: Turn this parameters
    let crl_series: u16 = 1;
    let crl_psid: u64 = 1;

    let ra_private_key = latest_ra_private_key(db).await?;
    let ra_certificate = latest_ra_certificate(db).await?;

    let (new_crl, cracaid) = oscms_bridge::make_secured_crl_encoded(
        crl_series,
        crl_psid,
        ra_private_key,
        ra_certificate,
        vec![],
    )
    .map_err(oscms_error_to_ra_response_error)?;

    // Store the new CRL
    store_crl(db, crl_series as i32, cracaid, new_crl.clone()).await?;

    Ok(new_crl)
}
