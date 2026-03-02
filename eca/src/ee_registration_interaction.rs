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

use actix_web::http::StatusCode;
use scmscommon::endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes;
use scmscommon::{GlobalConfig, errors};

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use scmscommon::core_types::{PayloadEePatchStatus, PayloadEeRegistrationResponse};

fn client_with_middleware() -> ClientWithMiddleware {
    // Retry up to eca_max_requests times
    let eca_max_requests = GlobalConfig::global().param_eca_max_reqs as u32;
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(eca_max_requests);

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

async fn get_eca_to_ra(
    req_addr: String,
    canonical_id: Option<String>,
    device_id: Option<u64>,
) -> Result<reqwest::Response, reqwest_middleware::Error> {
    let client = client_with_middleware();
    if let Some(canonical_id) = canonical_id.as_ref() {
        log::debug!("Requesting RA info by canonical ID: {}", canonical_id);
        return client
            .get(req_addr)
            .query(&[("canonicalId", canonical_id)])
            .send()
            .await;
    } else if let Some(device_id) = device_id.as_ref() {
        log::debug!("Requesting RA info by device ID: {}", device_id);
        return client
            .get(req_addr)
            .query(&[("deviceId", &device_id.to_string())])
            .send()
            .await;
    }

    client.get(req_addr).send().await
}

async fn patch_eca_to_ra(
    req_addr: String,
    payload: PayloadEePatchStatus,
) -> Result<reqwest::Response, reqwest_middleware::Error> {
    let client = client_with_middleware();
    client.patch(req_addr).json(&payload).send().await
}

fn get_full_ra_addr() -> String {
    // Ask public key from RA
    let ra_addr = GlobalConfig::global().ra_addr();
    let ra_port = GlobalConfig::global().ra_port;
    let ra_post_endpoint = &GlobalConfig::global().ra_post_ee_registration;
    format!("{}:{}{}", ra_addr, ra_port, ra_post_endpoint)
}

pub async fn check_ee_registration(
    is_successor_enroll_request: bool,
    in_canonical_id: Option<String>,
    in_device_id: Option<u64>,
) -> Result<(Vec<u8>, u64, String), errors::HandleResponseError> {
    log::debug!("Checking EE registration status");

    let req_addr = get_full_ra_addr();

    log::debug!(
        "Requesting RA (url: {:?} for ECA public key with canonical ID: {:?} and device ID: {:?}",
        req_addr,
        in_canonical_id,
        in_device_id
    );
    let response_result = get_eca_to_ra(req_addr, in_canonical_id, in_device_id).await;

    let ee_registration_info = match response_result {
        Ok(response) => {
            if !response.status().is_success() {
                log::error!(
                    "Canonical ID not registered, status code: {}",
                    response.status()
                );
                return Err(errors::HandleResponseError::new(
                    "Canonical ID not registered",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                ));
            }
            log::debug!("Received response from RA about ECA public key");

            // Extract public key from response
            response
                .json::<PayloadEeRegistrationResponse>()
                .await
                .map_err(|e| {
                    log::error!("Failed to parse RA response: {:?}", e);
                    errors::HandleResponseError::new(
                        "Failed to parse RA response",
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                    )
                })?
        }
        Err(e) => {
            log::error!("Failed to communicate with RA {:?}", e);
            return Err(errors::HandleResponseError::new(
                "Failed to communicate with RA",
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
    };

    log::debug!(
        "Received EE registration info from RA: {:?}",
        ee_registration_info
    );

    // First, check if the ee status is "Registered"
    // If status is Enrolled or Successor-Enrolled - response will be: Device already Enrolled;
    // If status is Blocked - response will be: Device is Blocked;
    // If status is Deleted - response will be: Device is Deleted;
    match ee_registration_info.status.as_str() {
        "Registered" => {
            log::debug!("Device 'Registered'");
            // if is_successor_enroll_request status should be "Enrolled" first
            if is_successor_enroll_request {
                log::debug!("Device is not enrolled yet");
                return Err(errors::HandleResponseError::new(
                    "Device is not Enrolled.",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                ));
            }
        }
        "Enrolled" => {
            if !is_successor_enroll_request {
                log::debug!("Device is already enrolled");
                return Err(errors::HandleResponseError::new(
                    "Device already Enrolled",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                ));
            } else {
                log::debug!("Device is already enrolled, proceeding with successor enrollment");
            }
        }
        "Successor-Enrolled" => {
            log::warn!("Device already Successor-Enrolled");
            return Err(errors::HandleResponseError::new(
                "Device already Successor-Enrolled",
                StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
        "Blocked" => {
            log::warn!("Device is Blocked");
            return Err(errors::HandleResponseError::new(
                "Device is Blocked",
                StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
        "Deleted" => {
            log::warn!("Device is Deleted");
            return Err(errors::HandleResponseError::new(
                "Device is Deleted",
                StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
        "Provisioning" => {
            log::debug!("Device is in Provisioning status");
            // If the device is in Provisioning status, we can proceed with the registration
            if !is_successor_enroll_request {
                log::debug!("Device is already enrolled");
                return Err(errors::HandleResponseError::new(
                    "Device already Enrolled",
                    StatusCode::BAD_REQUEST,
                    Ieee1609Dot2Dot1ErrorCodes::NotDefined,
                ));
            }
        }
        _ => {
            log::error!(
                "Unknown EE registration status: {}",
                ee_registration_info.status
            );
            return Err(errors::HandleResponseError::new(
                "Unknown EE registration status",
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            ));
        }
    }

    let canonical_public_key = hex::decode(ee_registration_info.public_key).map_err(|e| {
        log::error!("Failed to decode canonical public key: {:?}", e);
        errors::HandleResponseError::new(
            "Failed to decode canonical public key",
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    })?;

    log::debug!("Canonical Public Key: {:?}", canonical_public_key);
    log::debug!(
        "EE Registration Vehicle ID: {:?}",
        ee_registration_info.device_id
    );
    log::debug!("Canonical ID: {}", ee_registration_info.canonical_id);

    // device id string to u64
    let device_id = ee_registration_info.device_id.parse::<u64>().map_err(|e| {
        log::error!("Failed to parse device ID: {:?}", e);
        errors::HandleResponseError::new(
            "Failed to parse device ID",
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    })?;

    Ok((
        canonical_public_key,
        device_id,
        ee_registration_info.canonical_id,
    ))
}

pub async fn update_ee_status(
    canonical_id: String,
    new_status: String,
) -> Result<(), errors::HandleResponseError> {
    log::debug!("Updating EE status to: {}", new_status);

    let req_addr = get_full_ra_addr();

    let payload_ee_patch_status = PayloadEePatchStatus::new(canonical_id, new_status.clone());

    if let Err(e) = patch_eca_to_ra(req_addr, payload_ee_patch_status).await {
        log::error!("Failed to patch ECA status to RA: {:?}", e);
        return Err(errors::HandleResponseError::new(
            "Failed to patch ECA status to RA",
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ));
    }

    log::info!("Successfully updated EE status to: {}", new_status);
    Ok(())
}
