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

use crate::core_types::pre_linkage_value::Eplv;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// This struct is used to just bundle the req_id with a list of EPLVs.
/// The req_id identifies just one caterpillar.
/// Hence each caterpillar is associated with one list of EPLVs (per LA).
/// If we have two LAs, then the RA needs to deal with two list of EPLVs for each caterpillar.
/// And each caterpillar is associated with one EE request.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PlvPayload {
    pub req_id: u64,
    pub eplvs: Vec<Eplv>,
}

impl PlvPayload {
    pub fn new(req_id: u64, eplvs: Vec<Eplv>) -> Self {
        PlvPayload { req_id, eplvs }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plv_payload_serialization() {
        // Arrange
        let enc_value = [1u8; 32];
        let ephemeral_pub_key = vec![2u8; 32];
        let nonce = [5u8; 12];
        let i_index = 123;
        let j_index = 456;

        // Act
        let eplv1 = Eplv::new(
            enc_value,
            ephemeral_pub_key.clone(),
            nonce,
            i_index,
            j_index,
        );
        let eplv2 = Eplv::new(
            enc_value,
            ephemeral_pub_key.clone(),
            nonce,
            i_index,
            j_index,
        );

        // Create PlvPayload struct
        let plv_payload = PlvPayload::new(123, vec![eplv1, eplv2]);

        // Serialize the PlvPayload struct into JSON
        let json = serde_json::to_string(&plv_payload).unwrap();

        // Deserialize JSON back into PlvPayload struct
        let deserialized_plv_payload: PlvPayload = serde_json::from_str(&json).unwrap();

        // Check if the deserialized PlvPayload matches the original PlvPayload
        assert_eq!(plv_payload, deserialized_plv_payload);
    }
}
