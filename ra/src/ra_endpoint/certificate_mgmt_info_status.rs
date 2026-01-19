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

use crate::persistence::ccf_store::get_latest_updated_time;
use crate::persistence::crl_store::fech_all_crls_models;
use crate::persistence::ctl_store::fetch_all_ctl_series_ids_sequence_number_and_updated_time;
use crate::persistence::ra_certificates::{
    get_ra_certificates_latest_updated_time, latest_ra_certificate, latest_ra_private_key,
};
use crate::ra_endpoint::common::oscms_error_to_ra_response_error;
use actix_web::{HttpResponseBuilder, Responder, http::StatusCode, web};
use oscms_bridge::make_certificate_mgmt_info_status_file_encoded;
use scmscommon::{AppState, GlobalConfigInfo, errors};
use sea_orm::DatabaseConnection;
/// 1609.2.1 SS6.3.5.14 Certificate management information status download; table 23
///
/// Download the RA Certificate management information status
#[utoipa::path(
    get,
    tag = "Certificate management information status download",
    path = "certificate-management-info-status",
    context_path = "/v3/",
    params(
        ("Authorization" = Option<String>, Header, description = "Bearer [access-token/AT]"),
      ),
      responses(
        (
          status = 200,
          body = [u8], description = "A certificate management information status file as defined in 8.7",
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
#[actix_web::get("/certificate-management-info-status")]
pub async fn certificate_management_info_status_download(
    data: web::Data<AppState>,
    _req: actix_web::HttpRequest,
) -> impl Responder {
    log::debug!("Received GET request for certificate management info status download");

    let result = handle_certificate_management_info_status_download(&data.db).await;
    match result {
        Ok((payload, filename)) => HttpResponseBuilder::new(StatusCode::OK)
            .append_header(("Content-Type", "application/octet-stream"))
            .append_header((
                "Content-Disposition",
                format!("attachment; filename={}", filename).as_str(),
            ))
            .body(payload),
        Err(x) => x.http_error_response(),
    }
}

async fn handle_certificate_management_info_status_download(
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, String), errors::HandleResponseError> {
    log::debug!("Handling GET request for Certificate Management Info Status donwload");

    // Loading conf
    let config_info = GlobalConfigInfo::from_global_config_or_default();

    // TODO: We dont have the MA component yet, so MA info will be empty
    let ma_psid_list = vec![];
    let ma_update_time_list = vec![];

    let crl_info = fech_all_crls_models(db).await?;
    // craca Id as Vec<u8>, So we need to turn hex string into Vec<u8>
    let crl_craca_id_list = crl_info
        .iter()
        .map(|x| hex::decode(&x.craca_id).unwrap_or_default())
        .collect::<Vec<Vec<u8>>>();
    let crl_series_id_list = crl_info
        .iter()
        .map(|x| x.crl_series as u16)
        .collect::<Vec<u16>>();
    // issue date is Datetime so we need to convert it to u32
    let crl_issue_date_list = crl_info
        .iter()
        .map(|x| x.updated_time.and_utc().timestamp() as u32)
        .collect::<Vec<u32>>();

    // Getting RA info
    let ra_hostname = config_info.ra_url;
    let ra_private_key = latest_ra_private_key(db).await?;
    let ra_certificate = latest_ra_certificate(db).await?;
    let ra_updated_time = get_ra_certificates_latest_updated_time(db).await?;

    // Getting CTL Info
    let ctl_info = fetch_all_ctl_series_ids_sequence_number_and_updated_time(db).await?;
    let ctl_sequence_number_list = ctl_info.iter().map(|x| x.1).collect::<Vec<u16>>();
    let ct_series_id_list = ctl_info
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<Vec<u8>>>();
    let ctl_update_time_list = ctl_info.iter().map(|x| x.2).collect::<Vec<u32>>();

    // Getting CCT Info
    let ccf_update_time = get_latest_updated_time(db).await?;

    make_certificate_mgmt_info_status_file_encoded(
        ra_hostname,
        ma_psid_list,
        ma_update_time_list,
        ctl_sequence_number_list,
        ct_series_id_list,
        ctl_update_time_list,
        crl_craca_id_list,
        crl_series_id_list,
        crl_issue_date_list,
        ccf_update_time,
        ra_updated_time,
        ra_private_key,
        ra_certificate,
    )
    .map_err(oscms_error_to_ra_response_error)
}
