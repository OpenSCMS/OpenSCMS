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

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PayloadCertResponse {
    pub name: String,
    pub nid: u16,
    pub validity_duration_years: u16,
    pub start: u32,
    pub expiry: u32,
    pub request_for_renewal: u32,
    pub certificate_hash: String,
    pub encoded_certificate: Vec<u8>,
    pub public_key: Vec<u8>,
    pub private_key: Vec<u8>,
    pub enc_public_key: Option<Vec<u8>>,
    pub enc_private_key: Option<Vec<u8>>,
}

impl PayloadCertResponse {
    pub fn new(
        name: String,
        nid: u16,
        validity_duration_years: u16,
        start: u32,
        expiry: u32,
        request_for_renewal: u32,
        certificate_hash: String,
        encoded_certificate: Vec<u8>,
        public_key: Vec<u8>,
        private_key: Vec<u8>,
        enc_public_key: Option<Vec<u8>>,
        enc_private_key: Option<Vec<u8>>,
    ) -> Self {
        Self {
            name,
            nid,
            validity_duration_years,
            start,
            expiry,
            request_for_renewal,
            certificate_hash,
            encoded_certificate,
            public_key,
            private_key,
            enc_public_key,
            enc_private_key,
        }
    }
}
