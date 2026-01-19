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

use crate::core_logic::process_plv_request;
use crate::persistence::plv_to_ls_map;
use actix_web::{HttpResponse, HttpResponseBuilder, Responder, http::StatusCode, web};
use scmscommon::AppState;
use scmscommon::core_types::{PayloadLaToRa, PayloadRaToLa};
use scmscommon::endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes;
use scmscommon::{HandleResponseError, errors};
use sea_orm::DatabaseConnection;

#[utoipa::path(
  post,
  path = "/",
  request_body(
    content = PayloadRaToLa,
    content_type = "application/json",
    description = "PayloadRaToLa",
  ),
  responses(
    (status = 200, body = PayloadLaToRa, description = "PayloadLaToRa" ),
  )
)]
#[actix_web::post("/")]
pub async fn process_ra_request(
    data: web::Data<AppState>,
    req_body: web::Json<PayloadRaToLa>,
) -> impl Responder {
    let result = handle_post_request(req_body, &data.db).await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::OK)
            .append_header(("Content-Type", "application/json"))
            .json(&x),
        Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
    }
}

#[utoipa::path(
  get,
  path = "/plv-to-ls-map",
  request_body(
    content = Vec<[u8;16]>,
    content_type = "application/json",
    description = "list of PLVs",
  ),
  responses(
    (status = 200, body = [u8;16], description = "Linkage Seed" ),
  )
)]
#[actix_web::get("/plv-to-ls-map")]
pub async fn get_plv_to_ls_map(
    data: web::Data<AppState>,
    req_body: web::Json<Vec<[u8; 16]>>,
) -> impl Responder {
    // The MA will send a request to LA asking for LS given a certain list of PLVs.
    // The MA will send a list of PLVs Vec<[u8;16]> and the LA will send back a LS,
    // [u8;16] if it finds the LS associated with the PLV.
    // Otherwise the LA will not send anything back in the body.
    // What if the LA doesn’t find the necessary values for MA?
    // The MA would not be able to revoke the vehicle.
    // So it is essential that the LA have the information that the MA needs.

    let result = handle_get_plv_to_ls_map(req_body, &data.db).await;
    match result {
        Ok(x) => HttpResponseBuilder::new(StatusCode::OK)
            .append_header(("Content-Type", "application/json"))
            .json(x),
        Err(x) => HttpResponse::InternalServerError().body(x.to_string()),
    }
}

async fn handle_post_request(
    req_body: web::Json<PayloadRaToLa>,
    db: &DatabaseConnection,
) -> Result<PayloadLaToRa, errors::HandleResponseError> {
    log::debug!("Received payload_ra_to_la");
    let payload_ra_to_la = req_body.into_inner();
    log::debug!(
        "handle_post_request payload_ra_to_la: {:?}",
        payload_ra_to_la
    );

    let result = process_plv_request(payload_ra_to_la, db).await;
    if result.is_err() {
        log::debug!("Error processing PLV request: {:?}", result);
        return Err(HandleResponseError::new(
            "ProcessPlvRequestError",
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }
    let (payload_la_to_ra, plv_to_ls_maps) = result.unwrap();
    log::debug!(
        "handle_post_request payload_la_to_ra: {:?}",
        payload_la_to_ra
    );

    log::debug!("Storing PLV to LS maps");
    for plv_to_ls_map in &plv_to_ls_maps {
        plv_to_ls_map.store(db).await?;
    }

    log::debug!(
        "Returning payload_la_to_ra: len {:?}",
        payload_la_to_ra.plv_payloads.len()
    );
    Ok(payload_la_to_ra)
}

/// The MA will send a list of PLVs for the LA to find in its database.
/// If the LA finds something it will check which LS is associated and send it
/// back to the MA.
async fn handle_get_plv_to_ls_map(
    req_body: web::Json<Vec<[u8; 16]>>,
    db: &DatabaseConnection,
) -> Result<[u8; 16], errors::HandleResponseError> {
    log::debug!("Starting to look for LS given PLVs");
    let list_plvs = req_body.into_inner();
    let mut results = Vec::new();
    for plv in list_plvs.clone() {
        println!("{:?}", plv);
        let plv_value = plv.to_vec();
        let plv_result = plv_to_ls_map::find_ls(db, plv_value).await;
        results.push(plv_result);
    }

    for vec_value in results.into_iter().flatten() {
        if let Ok(array_value) = vec_value.try_into() {
            return Ok(array_value); // Return the first successful [u8; 16]
        }
    }
    log::error!("list_plvs: {:?}", list_plvs);
    Err(HandleResponseError::new(
        "Could not find an LS given the PLVs",
        StatusCode::INTERNAL_SERVER_ERROR,
        Ieee1609Dot2Dot1ErrorCodes::NotDefined,
    ))
}
