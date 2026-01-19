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

use crate::plv_payload::PlvPayload;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// There is one PayloadLaToRa per LA.
/// Potentially inside the RA it needs to handle two PaylodLaToRa objects.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PayloadLaToRa {
    pub request_id: String, // The request_id is associated with a bundle of caterpillars, i.e. a unique request from RA.
    pub plv_payloads: Vec<PlvPayload>,
}

impl PayloadLaToRa {
    pub fn new(request_id: String, plv_payloads: Vec<PlvPayload>) -> Self {
        Self {
            request_id,
            plv_payloads,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_types::pre_linkage_value::Eplv;

    #[test]
    fn test_payload_la_to_ra_new() {
        // Create some PlvPayload instances
        let plv_payload1 = PlvPayload {
            req_id: 1,
            eplvs: vec![
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 2),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 3),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 4),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 5),
            ],
        };

        let plv_payload2 = PlvPayload {
            req_id: 2,
            eplvs: vec![
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 2),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 3),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 4),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 5),
            ],
        };

        // Create a new PayloadLaToRa instance using the new method
        let payload_la_to_ra = PayloadLaToRa::new(
            "sample_request_id".to_owned(),
            vec![plv_payload1.clone(), plv_payload2.clone()],
        );

        // Assert that the fields of the created instance match the expected values
        assert_eq!(payload_la_to_ra.request_id, "sample_request_id".to_owned());
        assert_eq!(
            payload_la_to_ra.plv_payloads,
            vec![plv_payload1, plv_payload2]
        );
    }
}
