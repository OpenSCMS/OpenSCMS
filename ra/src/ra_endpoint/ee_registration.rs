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

use crate::persistence::ee_registration::{
    fetch_ee_registration_by_canonical_id, fetch_ee_registration_by_device_id,
    fetch_ee_registration_by_public_key, patch_status_ee_registration, store_new_registration,
};

use crate::entities::sea_orm_active_enums::{DeviceType, Status};

use actix_web::{HttpResponse, Responder, web};
use scmscommon::core_types::{
    PayloadEePatchStatus, PayloadEeRegistration, PayloadEeRegistrationResponse,
};
use scmscommon::{AppState, EeRegistrationDeviceType};
use scmscommon::{DeviceConfigInfo, GlobalConfigInfo};
use sha2::{Digest, Sha256};

// EE registration path dcms/device/na
#[utoipa::path(
    post,
    tag = "Device Configuration",
    path = "/dcms/device/na",
    request_body(
      content = PayloadEeRegistration,
      content_type = "application/json",
      description = "PayloadEeRegistration",
    ),
    responses(
      (status = 200, body = PayloadEeRegistrationResponse, description = "Registration json with canonical id, device id and device policy"),
      (status = 400, description = "Bad request"),
      (status = 404, description = "Not found"),
      (status = 500, description = "Internal server error"),
      (status = 503, description = "Service unavailable"),
    )
)]
#[actix_web::post("/dcms/device/na")]
pub async fn handle_ee_registration(
    data: web::Data<AppState>,
    req_body: web::Json<PayloadEeRegistration>,
) -> impl Responder {
    log::debug!("Received EE registration request {:?}", req_body);

    // First check if public key already exists
    let mut public_key = req_body.canonical_public_key.clone();
    match fetch_ee_registration_by_public_key(public_key.clone(), &data.db).await {
        Ok(model) => match model {
            Some(certificate_entity) => {
                log::debug!(
                    "EE registration already exists for public key: {:?}",
                    certificate_entity
                );
                return HttpResponse::BadRequest()
                    .body("EE registration already exists".to_string());
            }
            None => {
                log::debug!(
                    "No existing EE registration found for public key, proceeding with registration"
                );
            }
        },
        Err(e) => {
            log::error!("Error fetching EE registration by public key: {:?}", e);
            return HttpResponse::InternalServerError().body("Internal server error".to_string());
        }
    };

    // Calculate canonical id
    // Check if canonical public key is uncompressed: 0x04 + 64 bytes = 130 characters
    if public_key.len() != 130 {
        log::error!("Invalid public key length: {:?}", public_key);
        return HttpResponse::BadRequest().body("Invalid public key length".to_string());
    }

    // Check if first byte is 0x04
    if !public_key.starts_with("04") {
        log::error!("Public key is not in uncompressed format: {:?}", public_key);
        return HttpResponse::BadRequest()
            .body("Public key must be in uncompressed format".to_string());
    }

    // Canonical id is sha256 of the public key ignoring first byte
    // First remove first byte and turn that hex string to Vec<u8>
    public_key = public_key[2..].to_string(); // Remove the first byte (0x04)
    log::debug!("Public key after removing first byte: {:?}", public_key);
    // Convert the public key hex string to bytes
    let public_key_bytes = match hex::decode(&public_key) {
        Ok(bytes) => bytes,
        Err(e) => {
            log::error!("Failed to decode public key hex string: {:?}", e);
            return HttpResponse::BadRequest().body("Invalid public key format".to_string());
        }
    };

    let canonical_id_value = {
        let mut hasher = Sha256::new();
        hasher.update(&public_key_bytes);
        hasher.finalize()
    };

    let canonical_id = hex::encode(canonical_id_value);
    log::info!("Canonical ID generated: {:?}", canonical_id);

    // Generate device id
    // Device ID is a random u64 value
    let mut device_id = rand::random::<u64>();
    // Check if device_id already exists
    loop {
        let result = fetch_ee_registration_by_device_id(device_id, &data.db).await;
        if result.is_ok() && result.unwrap().is_none() {
            // Device ID is unique, break the loop
            break;
        }
        device_id = rand::random::<u64>();
    }

    log::info!("Generated device ID: {:?}", device_id);

    // Store:
    match store_new_registration(
        device_id,
        req_body.device_type.clone(),
        canonical_id.clone(),
        public_key,
        &data.db,
    )
    .await
    {
        Ok(_) => {
            log::debug!("EE registration stored successfully");

            // Prepare response
            let response = PayloadEeRegistrationResponse::new(
                canonical_id,
                device_id.to_string(),
                req_body.device_type.clone(),
                req_body.canonical_public_key.clone(),
                "Registered".to_string(),
                "na".to_string(),
                chrono::Utc::now().to_rfc3339(), // Created time
                chrono::Utc::now().to_rfc3339(), // Updated time
                DeviceConfigInfo::from_config_info(),
            );
            HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
        }
        Err(e) => {
            log::error!("Error storing EE registration: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error".to_string())
        }
    }
}

#[derive(serde::Deserialize, utoipa::ToSchema, utoipa::IntoParams, Clone)]
pub struct CanonicalIdParam {
    #[serde(alias = "canonicalId", rename(serialize = "canonicalId"))]
    canonical_id: Option<String>,
    #[serde(alias = "deviceId", rename(serialize = "deviceId"))]
    device_id: Option<u64>,
}

#[utoipa::path(
  get,
  tag = "Device Configuration",
  path = "/dcms/device/na",
  params(
    (
        "canonicalId" = Option<String>,
        Query,
        description = "Canonical id"
    ),
    (
        "deviceId" = Option<u64>,
        Query,
        description = "Device id"
    )
  ),
  responses(
    (status = 200, body = PayloadEeRegistrationResponse, description = "Registration json with canonical id, device id and device policy"),
    (status = 400, description = "Bad request"),
    (status = 404, description = "Not found"),
    (status = 500, description = "Internal server error"),
    (status = 503, description = "Service unavailable"),
  )
)]
#[actix_web::get("/dcms/device/na")]
pub async fn handle_get_ee(
    data: web::Data<AppState>,
    query_params: web::Query<CanonicalIdParam>,
) -> impl Responder {
    log::debug!(
        "Received EE registration GET request: {:?}",
        query_params.canonical_id
    );

    // Check if canonical_id is provided and if it's registered
    let canonical_id_opt = query_params.canonical_id.clone();
    let device_id_opt = query_params.device_id;

    let fetch_result: Result<
        Option<crate::entities::ee_registration::Model>,
        scmscommon::PersistenceLoadError,
    >;

    if let Some(canonical_id) = canonical_id_opt {
        log::debug!(
            "Fetching EE registration by canonical ID: {:?}",
            canonical_id
        );
        fetch_result = fetch_ee_registration_by_canonical_id(canonical_id, &data.db).await;
    } else if let Some(device_id) = device_id_opt {
        log::debug!("Fetching EE registration by device ID: {:?}", device_id);
        fetch_result = fetch_ee_registration_by_device_id(device_id, &data.db).await;
    } else {
        log::error!("No valid parameters provided for EE registration lookup");
        return HttpResponse::BadRequest().body("Invalid parameters".to_string());
    }

    // Fetch EE registration by canonical_id
    match fetch_result {
        Ok(model) => match model {
            Some(certificate_entity) => {
                log::debug!("EE registration found: {:?}", certificate_entity.device_id);

                let device_type = match certificate_entity.device_type {
                    DeviceType::Obu => EeRegistrationDeviceType::OBU,
                    DeviceType::Rsu => EeRegistrationDeviceType::RSU,
                };

                let status = match certificate_entity.status {
                    Status::Registered => "Registered".to_string(),
                    Status::Enrolled => "Enrolled".to_string(),
                    Status::SuccessorEnrolled => "Successor-Enrolled".to_string(),
                    Status::Blocked => "Blocked".to_string(),
                    Status::Provisioning => "Provisioning".to_string(),
                    Status::Deleted => "Deleted".to_string(),
                };

                // Prepare response
                let response = PayloadEeRegistrationResponse::new(
                    certificate_entity.canonical_id,
                    certificate_entity.device_id.to_string(),
                    device_type,
                    certificate_entity.public_key,
                    status,
                    "na".to_string(),
                    chrono::Utc::now().to_rfc3339(), // Created time
                    chrono::Utc::now().to_rfc3339(), // Updated time
                    DeviceConfigInfo::from_config_info(),
                );
                HttpResponse::Ok().body(serde_json::to_string(&response).unwrap())
            }
            None => {
                log::error!("No EE registration found for canonical ID or device ID.");
                HttpResponse::NotFound().body("EE registration not found".to_string())
            }
        },
        Err(e) => {
            log::error!("Error fetching EE registration by canonical ID: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error".to_string())
        }
    }
}

#[utoipa::path(
    patch,
    tag = "Device Configuration",
    path = "/dcms/device/na",
    request_body(
        content = PayloadEePatchStatus,
        content_type = "application/json",
        description = "PayloadEePatchStatus",
    ),
    responses(
      (status = 200, description = "Device status updated successfully"),
      (status = 400, description = "Bad request"),
      (status = 404, description = "Not found"),
      (status = 500, description = "Internal server error"),
      (status = 503, description = "Service unavailable"),
    )
  )]
#[actix_web::patch("/dcms/device/na")]
pub async fn handle_patch_ee(
    data: web::Data<AppState>,
    req_body: web::Json<PayloadEePatchStatus>,
) -> impl Responder {
    log::info!(
        "Received EE registration PATCH request: {:?}",
        req_body.canonical_id
    );

    // Check if canonical_id is provided and exists
    let canonical_id = req_body.canonical_id.clone();
    if canonical_id.is_empty() {
        log::error!("Canonical ID is required but not provided");
        return HttpResponse::BadRequest().body("Canonical ID is required".to_string());
    }

    // Check if status is provided
    if req_body.status.is_empty() {
        log::error!("Status is required but not provided");
        return HttpResponse::BadRequest().body("Status is required".to_string());
    }

    // Fetch EE registration by canonical_id
    match fetch_ee_registration_by_canonical_id(canonical_id.clone(), &data.db).await {
        Ok(model) => match model {
            Some(certificate_entity) => {
                log::debug!("EE registration found: {:?}", certificate_entity.device_id);

                // Update the status if provided
                let updated_status = match req_body.status.to_lowercase().as_str() {
                    "registered" => Status::Registered,
                    "enrolled" => Status::Enrolled,
                    "successor-enrolled" => Status::SuccessorEnrolled,
                    "blocked" => Status::Blocked,
                    "provisioning" => Status::Provisioning,
                    "deleted" => Status::Deleted,
                    _ => certificate_entity.status, // Keep existing status if not provided
                };

                // Save new status
                match patch_status_ee_registration(canonical_id.clone(), updated_status, &data.db)
                    .await
                {
                    Ok(_) => {
                        log::debug!("EE registration status updated successfully");
                        HttpResponse::Ok().body("Device status updated".to_string())
                    }
                    Err(e) => {
                        log::error!("Error updating EE registration status: {:?}", e);
                        HttpResponse::InternalServerError()
                            .body("Internal server error".to_string())
                    }
                }
            }
            None => {
                log::error!(
                    "No EE registration found for canonical ID: {:?}",
                    canonical_id
                );
                HttpResponse::NotFound().body("EE registration not found".to_string())
            }
        },
        Err(e) => {
            log::error!("Error fetching EE registration by canonical ID: {:?}", e);
            HttpResponse::InternalServerError().body("Internal server error".to_string())
        }
    }
}

// Download policy /dcms/policy/na para global policy file
// Download ee policy /dcms/policy/na?canonicalId=<canonicalID> para ee policy file
#[utoipa::path(
    get,
    tag = "Device Configuration",
    path = "/dcms/policy/na",
    params(
        (
            "canonicalId" = Option<String>,
            Query,
            description = "Canonical ID of the EE registration"
        )
    ),
    responses(
        (status = 200, description = "Policy file downloaded successfully"),
        (status = 400, description = "Bad request"),
        (status = 404, description = "Not found"),
        (status = 500, description = "Internal server error"),
        (status = 503, description = "Service unavailable"),
    )
)]
#[actix_web::get("/dcms/policy/na")]
pub async fn handle_get_policy(
    _data: web::Data<AppState>,
    query_params: web::Query<CanonicalIdParam>,
) -> impl Responder {
    log::debug!(
        "Received GET request for policy with canonical ID: {:?}",
        query_params.canonical_id
    );

    // Check if canonical_id is provided
    let canonical_id_opt = query_params.canonical_id.clone();
    if canonical_id_opt.is_none() {
        log::debug!("Canonical ID not provided, returning global policy file");

        // Here you would typically fetch the global policy file
        let config_to_send = GlobalConfigInfo::from_global_config_or_default();
        return HttpResponse::Ok().body(serde_json::to_string(&config_to_send).unwrap());
    }

    log::debug!(
        "Fetching policy for EE registration with canonical ID: {:?}",
        canonical_id_opt
    );

    let config_to_send = DeviceConfigInfo::from_config_info();
    HttpResponse::Ok().body(serde_json::to_string(&config_to_send).unwrap())
}
