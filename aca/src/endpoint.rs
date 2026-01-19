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

use crate::core_logic::build_certificates::{build_certificates, build_non_butterfly_certificate};
use crate::persistence::lv_to_plv_map::{find_plvs, store_lv_to_plv_map};
use actix_web::{HttpResponse, HttpResponseBuilder, Responder};
use actix_web::{http::StatusCode, web};
use scmscommon::AppState;
use scmscommon::core_types::Certificate;
use scmscommon::core_types::PayloadRaToAca;
use scmscommon::errors as scmscommon_errors;
use sea_orm::DatabaseConnection;

/// Provide a health-check endpoint for the service
///
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

#[utoipa::path(
  post,
  tag = "Authorization Certificates",
  path = "/",
  responses(
    (
      status = 200,
      body = [u8], description = "certificate",
      headers(("Content-Type", description = "octet-stream"))
    ),
  )
)]
#[actix_web::post("/")]
async fn process_ra_request(
    data: web::Data<AppState>,
    req_body: web::Json<PayloadRaToAca>,
) -> impl Responder {
    log::debug!("Start processing RA request (POST)");
    let result = handle_post_request(req_body, &data.db).await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::OK)
            .append_header(("Content-Type", "application/json"))
            .json(&x),
        Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
    }
}

pub async fn handle_post_request(
    req_body: web::Json<PayloadRaToAca>,
    db: &DatabaseConnection,
) -> Result<Vec<Certificate>, scmscommon_errors::HandleResponseError> {
    let payload_ra_to_aca = req_body.into_inner();
    log::debug!(
        "Received payload_ra_to_aca (n cocoons {}).",
        payload_ra_to_aca.cocoon_requests.len()
    );

    let exp_type = payload_ra_to_aca.exp_type;
    if exp_type == scmscommon::core_types::ExpansionType::NonButterfly {
        log::debug!("Handling Nonbutterfly request.");
        if payload_ra_to_aca.non_butterfly_request.is_none() {
            log::debug!("Verifying key is missing.");
            return Err(scmscommon_errors::HandleResponseError::new(
                "Verifying key is missing",
                StatusCode::BAD_REQUEST,
                scmscommon_errors::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
        let non_butterfly_request = payload_ra_to_aca.non_butterfly_request.unwrap();

        let result = build_non_butterfly_certificate(exp_type, non_butterfly_request, db).await;
        if result.is_err() {
            log::debug!("Error building certificates: {:?}", result);
            return Err(scmscommon_errors::HandleResponseError::new(
                "Build Certificate Error",
                StatusCode::INTERNAL_SERVER_ERROR,
                scmscommon_errors::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
        let certificate = result.unwrap();
        log::debug!("Returning 1 certificate");
        return Ok(vec![certificate]);
    }

    let result = build_certificates(payload_ra_to_aca, db).await;
    if result.is_err() {
        log::debug!("Error building certificates: {:?}", result);
        return Err(scmscommon_errors::HandleResponseError::new(
            "Build Certificate Error",
            StatusCode::INTERNAL_SERVER_ERROR,
            scmscommon_errors::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }
    let (certificates, lv_to_plv_maps) = result.unwrap();

    for lv_to_plv_map in lv_to_plv_maps {
        let result = store_lv_to_plv_map(db, lv_to_plv_map).await;
        if result.is_err() {
            log::debug!("Error storing the lv_to_plv_map: {:?}", result);
            return Err(scmscommon_errors::HandleResponseError::new(
                "Storing lv_to_plv_map error",
                StatusCode::INTERNAL_SERVER_ERROR,
                scmscommon_errors::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
    }

    log::debug!("Returning certificates: len {:?}", certificates.len());
    Ok(certificates)
}

#[utoipa::path(
  get,
  path = "/get-plvs-using-ls",
  request_body(
    content = PayloadRaToLa,
    content_type = "application/json",
    description = "Linkage Seed",
  ),
  responses(
    (status = 200, body = [u8;16], description = "Pre Linkage Seed"),
  )
)]
#[actix_web::get("/get-plvs-using-ls")]
pub async fn get_plvs_using_ls(
    data: web::Data<AppState>,
    req_body: web::Json<[u8; 16]>,
) -> impl Responder {
    let result = handle_get_plvs_using_ls(req_body, &data.db).await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::OK)
            .append_header(("Content-Type", "application/json"))
            .json(&x),
        Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
    }
}

pub async fn handle_get_plvs_using_ls(
    req_body: web::Json<[u8; 16]>,
    db: &DatabaseConnection,
) -> Result<Vec<[u8; 16]>, scmscommon_errors::HandleResponseError> {
    let lv = req_body.into_inner();
    let result = find_plvs(db, lv).await;
    if result.is_err() {
        log::debug!("Error building certificates: {:?}", result);
        return Err(scmscommon_errors::HandleResponseError::new(
            "Build Certificate Error",
            StatusCode::INTERNAL_SERVER_ERROR,
            scmscommon_errors::Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }
    let plvs = result.unwrap();
    Ok(plvs)
}
