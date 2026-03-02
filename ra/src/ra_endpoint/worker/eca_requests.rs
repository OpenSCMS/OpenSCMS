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

use scmscommon::GlobalConfig;
use scmscommon::errors;

use crate::ra_endpoint::worker::common::{get_ra_to_eca, post_ra_to_eca};

pub async fn download_eca_public_key() -> Result<Vec<u8>, errors::ScmsInternalCommError> {
    // Construct request address
    let eca_addr = GlobalConfig::global().eca_addr();
    let eca_port = GlobalConfig::global().eca_port;
    let req_addr = format!("{}:{}/eca-public-key", eca_addr, eca_port);

    let response_result = get_ra_to_eca(req_addr).await;
    match response_result {
        Ok(response) => {
            log::debug!("Received response from ECA");
            // Capture PayloadLaToRa
            if response.status().is_success() {
                let content = response.bytes().await?;
                log::debug!("ECA response is ok, returning content!");
                return Ok(content.to_vec());
            }

            let status_code = response.status().as_u16();
            let err_return = errors::ScmsInternalCommError::new(
                format!(
                    "Received response from ECA but not with error status code: {}",
                    status_code
                )
                .as_str(),
                errors::InternalCommWire::RaToEca,
                status_code,
            );

            log::error!(
                "Failed to Communicate with ECA, status code: {:?}",
                err_return
            );
            Err(err_return)
        }
        Err(e) => Err(errors::ScmsInternalCommError::new(
            format!("Failed to Communicate with ECA, status code: {}", e).as_str(),
            errors::InternalCommWire::RaToEca,
            500,
        )),
    }
}

pub async fn handle_eca_enrollment_certificate_request(
    successor_enrollment_request: Vec<u8>,
    hash_id: String,
) -> Result<(String, Vec<u8>), errors::ScmsInternalCommError> {
    // Construct request address
    let eca_addr = GlobalConfig::global().eca_addr();
    let eca_port = GlobalConfig::global().eca_port;
    let req_addr = format!("{}:{}/successor-enrollment-certificate", eca_addr, eca_port);

    let response_result = post_ra_to_eca(req_addr, successor_enrollment_request, hash_id).await;
    match response_result {
        Ok(response) => {
            log::debug!("Received response from ECA");
            // Capture PayloadLaToRa
            if response.status().is_success() {
                let filename = extract_filename_from_header(&response)?;
                let content = response.bytes().await?;
                log::debug!("ECA response is ok, returning file {}!", filename);
                return Ok((filename, content.to_vec()));
            }

            let status_code = response.status().as_u16();
            let mut err_return = errors::ScmsInternalCommError::new(
                format!(
                    "Received response from ECA but not with error status code: {}",
                    status_code
                )
                .as_str(),
                errors::InternalCommWire::RaToEca,
                status_code,
            );

            if status_code == 400 || status_code == 403 {
                let detailed_error_code = extract_error_code_from_header(&response);
                err_return.set_detailed_code(detailed_error_code);
            }
            log::error!(
                "Failed to Communicate with ECA, status code: {:?}",
                err_return
            );
            Err(err_return)
        }
        Err(e) => Err(errors::ScmsInternalCommError::new(
            format!("Failed to Communicate with ECA, status code: {}", e).as_str(),
            errors::InternalCommWire::RaToEca,
            500,
        )),
    }
}

fn extract_filename_from_header(
    response: &reqwest::Response,
) -> Result<String, errors::ScmsInternalCommError> {
    if let Some(content_disposition) = response.headers().get("Content-Disposition") {
        if let Ok(header_value) = content_disposition.to_str() {
            if let Some(filename) = header_value.split(';').find_map(|param| {
                if param.trim_start().starts_with("filename=") {
                    Some(param.trim_start()[9..].trim_matches('"').to_string())
                } else {
                    None
                }
            }) {
                return Ok(filename);
            }
        }
    }
    Err(errors::ScmsInternalCommError::new(
        "Failed to extract filename from ECA response {}",
        errors::InternalCommWire::RaToEca,
        500,
    ))
}

fn extract_error_code_from_header(response: &reqwest::Response) -> Option<i32> {
    if let Some(content_disposition) = response.headers().get("Ieee-1609.2.1-Error") {
        log::debug!("Error header: {:?}", content_disposition);
        if let Ok(header_value) = content_disposition.to_str() {
            log::debug!("Error header value: {:?}", header_value);
            match header_value.split('-').nth(1) {
                Some(code) => match code.parse::<i32>() {
                    Ok(code) => {
                        log::debug!("Error code found: {:?}", code);
                        return Some(code);
                    }
                    Err(_) => {
                        log::error!("Failed to parse error code from header");
                        return None;
                    }
                },
                None => {
                    return None;
                }
            };
        }
    }
    None
}
