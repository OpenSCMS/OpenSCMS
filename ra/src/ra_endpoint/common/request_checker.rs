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
use crate::persistence::ra_request_management::{
    fetch_last_request_time_and_count_by_app_ip, fetch_last_request_time_and_count_by_vid,
    store_new_ra_request, update_status_by_app_ip_and_type, update_status_by_vid_and_type,
};
use actix_web::http::StatusCode;
use scmscommon::endpoint_error_codes::{BadRequestErrorCodes, Ieee1609Dot2Dot1ErrorCodes};
use scmscommon::{GlobalConfig, errors};
use sea_orm::DatabaseConnection;

async fn check_outside_ra_max_reqs(
    app_ip: String,
    vid: Option<u64>,
    request_time: u32,
    request_type: RaRequestManagementRequestType,
    db: &DatabaseConnection,
    request_entry: Option<(u32, i32)>,
) -> Result<i32, errors::HandleResponseError> {
    if request_entry.is_none() {
        store_new_ra_request(db, app_ip, vid, request_time, request_type).await?;
        return Ok(0);
    }

    let (_, period_request_count) = request_entry.unwrap();

    // Throw /POST Error- 400-79 outside ra-maxReqs.
    // Maximum number of requests in a 7 × 24-hour period for a
    // particular set of certificate permissions (PSID/SSP)
    // authorized by the same enrollment certificate. The requests
    // within a given period are determined based on the
    // generationTime field in the request, not on the reception
    // time at the RA.
    let max_requests = GlobalConfig::global().param_ra_max_reqs as i32;
    let next_period_request_count = period_request_count + 1;
    if next_period_request_count > max_requests {
        log::debug!(
            "Weekly request count: {} max_requests {}",
            next_period_request_count,
            max_requests
        );

        // Setting error code based on request type
        let (message, error_code_type) = match request_type {
            RaRequestManagementRequestType::AuthorizationRequest
            | RaRequestManagementRequestType::SuccessorEnrollmentRequest => (
                "Outside ra-maxReqs".to_string(),
                BadRequestErrorCodes::OutsideRaMaxReqs,
            ),
            RaRequestManagementRequestType::AuthorizationDownload
            | RaRequestManagementRequestType::SuccessorEnrollmentDownload => (
                "Outside download-maxReqs".to_string(),
                BadRequestErrorCodes::OutsideDownloadMaxReqs,
            ),
        };

        return Err(errors::HandleResponseError::new(
            message.as_str(),
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(error_code_type),
        ));
    }

    Ok(next_period_request_count)
}

pub async fn check_ra_incoming_request_frequency_by_vid(
    app_ip: String,
    vid: u64,
    request_time: u32,
    request_type: RaRequestManagementRequestType,
    db: &DatabaseConnection,
) -> Result<(), errors::HandleResponseError> {
    log::debug!(
        "Checking RA request frequency for request type {:?} and vid {} at {}",
        request_type,
        vid,
        request_time
    );

    let request_entry =
        fetch_last_request_time_and_count_by_vid(vid, request_type.clone(), db).await?;

    // Check 400-79 outside ra-maxReqs.
    let next_period_request_count = check_outside_ra_max_reqs(
        app_ip,
        Some(vid),
        request_time,
        request_type.clone(),
        db,
        request_entry,
    )
    .await?;
    if next_period_request_count == 0 {
        log::debug!("First time of vid {} requesting.", vid);
        return Ok(());
    }

    // Update request table
    update_status_by_vid_and_type(
        db,
        vid,
        request_time,
        next_period_request_count,
        request_type,
    )
    .await?;

    Ok(())
}

pub async fn check_ra_incoming_request_frequency_by_app_ip(
    app_ip: String,
    request_time: u32,
    request_type: RaRequestManagementRequestType,
    db: &DatabaseConnection,
) -> Result<(), errors::HandleResponseError> {
    log::debug!(
        "Checking RA request frequency for request type {:?} and app_ip {} at {}",
        request_type,
        app_ip,
        request_time
    );

    let request_entry =
        fetch_last_request_time_and_count_by_app_ip(app_ip.clone(), request_type.clone(), db)
            .await?;

    // Check 400-79 outside ra-maxReqs.
    let next_period_request_count = check_outside_ra_max_reqs(
        app_ip.clone(),
        None,
        request_time,
        request_type.clone(),
        db,
        request_entry,
    )
    .await?;
    if next_period_request_count == 0 {
        log::debug!("First time of app_ip {} requesting.", app_ip);
        return Ok(());
    }

    // Update request table
    update_status_by_app_ip_and_type(
        db,
        app_ip,
        request_time,
        next_period_request_count,
        request_type,
    )
    .await?;

    Ok(())
}
