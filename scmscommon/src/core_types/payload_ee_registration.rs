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

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::DeviceConfigInfo;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum EeRegistrationRequestType {
    NewEnrollment,
    UpdateSuccessorEnrollment,
    UpdateBlockStatus,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum EeRegistrationDeviceType {
    #[serde(alias = "obu", rename(serialize = "obu"))]
    OBU,
    #[serde(alias = "rsu", rename(serialize = "rsu"))]
    RSU,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PayloadEePatchStatus {
    #[serde(alias = "canonicalId", rename(serialize = "canonicalId"))]
    pub canonical_id: String,
    #[serde(alias = "status", rename(serialize = "status"))]
    pub status: String,
}

impl PayloadEePatchStatus {
    pub fn new(canonical_id: String, status: String) -> Self {
        Self {
            canonical_id,
            status,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PayloadEeRegistration {
    #[serde(alias = "deviceType", rename(serialize = "deviceType"))]
    pub device_type: EeRegistrationDeviceType,
    #[serde(alias = "publicKey", rename(serialize = "publicKey"))]
    pub canonical_public_key: String,
}

impl PayloadEeRegistration {
    pub fn new(device_type: EeRegistrationDeviceType, canonical_public_key: String) -> Self {
        Self {
            device_type,
            canonical_public_key,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PayloadEeRegistrationResponse {
    #[serde(alias = "canonicalId", rename(serialize = "canonicalId"))]
    pub canonical_id: String,
    #[serde(alias = "deviceId", rename(serialize = "deviceId"))]
    pub device_id: String,
    #[serde(alias = "deviceType", rename(serialize = "deviceType"))]
    pub device_type: EeRegistrationDeviceType,
    #[serde(alias = "publicKey", rename(serialize = "publicKey"))]
    pub public_key: String,
    pub status: String,
    pub version: String,
    #[serde(alias = "createdTime", rename(serialize = "createdTime"))]
    pub created_time: String,
    #[serde(alias = "updatedTime", rename(serialize = "updatedTime"))]
    pub updated_time: String,
    #[serde(alias = "devicePolicy", rename(serialize = "devicePolicy"))]
    pub device_policy: DeviceConfigInfo,
}

impl PayloadEeRegistrationResponse {
    pub fn new(
        canonical_id: String,
        device_id: String,
        device_type: EeRegistrationDeviceType,
        public_key: String,
        status: String,
        version: String,
        created_time: String,
        updated_time: String,
        device_policy: DeviceConfigInfo,
    ) -> Self {
        Self {
            canonical_id,
            device_id,
            device_type,
            public_key,
            status,
            version,
            created_time,
            updated_time,
            device_policy,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_ee_registration_new() {
        // Create a new PayloadEeRegistration instance using the new method
        let payload_ee_registration = PayloadEeRegistration::new(
            EeRegistrationDeviceType::OBU,
            "public_key_example".to_string(),
        );

        // Assert that the fields of the created instance match the expected values
        assert_eq!(
            payload_ee_registration.device_type,
            EeRegistrationDeviceType::OBU
        );
        assert_eq!(
            payload_ee_registration.canonical_public_key,
            "public_key_example"
        );
    }
}
