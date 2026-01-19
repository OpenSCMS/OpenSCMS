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
use base64::{Engine as _, engine::general_purpose};
use scmscommon::{
    GlobalConfig, ScmsErrorLevel, endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes, errors,
    get_current_time_seconds,
};
use sea_orm::DatabaseConnection;

use crate::persistence::ra_certificates::{
    latest_eca_certificate, latest_eca_public_key, latest_ra_certificate, latest_ra_enc_private_key,
};
use crate::ra_endpoint::validation;
use crate::ra_endpoint::worker::tasks::eca_public_key_download_and_store;

pub fn oscms_error_to_ra_response_error(
    oscms_bridge_error: oscms_bridge::OscmsBridgeError,
) -> errors::HandleResponseError {
    let (message, error_code) =
        oscms_bridge::oscms_bridge_error_to_ieee_1609_error_codes(oscms_bridge_error);

    log::error!("Oscms-bridge Error: {}", message);

    let global_config = GlobalConfig::global();
    let param_scmsv3_error = global_config.param_scmsv3_error;

    if param_scmsv3_error == ScmsErrorLevel::Coarse {
        return errors::HandleResponseError::new(
            "Internal server error",
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        );
    }

    let status_code = match error_code {
        Ieee1609Dot2Dot1ErrorCodes::BadRequest(_) => StatusCode::BAD_REQUEST,
        Ieee1609Dot2Dot1ErrorCodes::Forbidden(_) => StatusCode::FORBIDDEN,
        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    errors::HandleResponseError::new(message.as_str(), status_code, error_code)
}

pub async fn decode_ieee1609dot2_authorization(
    ieee1609dot2_authorization_encoded: String,
    time_request_received: u32,
    db: &DatabaseConnection,
) -> Result<String, errors::HandleResponseError> {
    log::debug!(
        "Decoding ieee1609dot2_authorization {}",
        ieee1609dot2_authorization_encoded
    );

    if ieee1609dot2_authorization_encoded.is_empty() {
        return Err(errors::HandleResponseError::new(
            "Failed Parsing",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
        ));
    }

    let encoded = general_purpose::STANDARD
        .decode(&ieee1609dot2_authorization_encoded)
        .map_err(|_| {
            errors::HandleResponseError::new(
                "Failed Parsing",
                StatusCode::BAD_REQUEST,
                Ieee1609Dot2Dot1ErrorCodes::BadRequest(errors::BadRequestErrorCodes::FailedParsing),
            )
        })?;

    // Call decoder
    let mut eca_public = [0; 64];
    let eca_public_key_fetch_result = latest_eca_public_key(db).await;
    if eca_public_key_fetch_result.is_err() {
        eca_public = eca_public_key_download_and_store(db).await.map_err(|e| {
            errors::HandleResponseError::new(
                format!("Failed to get eca public key: {}", e.message).as_str(),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })?;
    } else if let Ok(key) = eca_public_key_fetch_result {
        eca_public.copy_from_slice(&key);
    }

    let ra_private = latest_ra_enc_private_key(db).await?;
    let eca_certificate = latest_eca_certificate(db).await?;
    let ra_certificate = latest_ra_certificate(db).await?;
    let (generation_time, filename) = match oscms_bridge::decode_download_request_spdu(
        encoded,
        eca_public,
        ra_private,
        ra_certificate,
        eca_certificate,
    ) {
        Ok((generation_time, filename)) => (generation_time, filename),
        Err(x) => return Err(oscms_error_to_ra_response_error(x)),
    };

    log::debug!(
        "Got generation_time {} and filename {} from header input.",
        generation_time,
        filename
    );

    // Validate generation time
    let current_time = get_current_time_seconds();
    validation::validate_generation_time(
        current_time,
        generation_time,
        time_request_received,
        false,
    )?;

    Ok(filename)
}
