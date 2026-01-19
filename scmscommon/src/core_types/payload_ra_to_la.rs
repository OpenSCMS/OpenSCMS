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

use crate::core_types::plv_request::PlvReq;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PayloadRaToLa {
    pub request_id: String,
    pub plv_reqs: Vec<PlvReq>, // There is one PlvReq per caterpillar.
}

impl PayloadRaToLa {
    pub fn new(request_id: String, plv_reqs: Vec<PlvReq>) -> Self {
        Self {
            request_id,
            plv_reqs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_ra_to_la_new() {
        // Create some PlvReq instances
        let plv_req1 = PlvReq {
            req_id: 1,
            ra_id: 2,
            i_min: 0,
            i_max: 10,
            j_max: 5,
        };

        let plv_req2 = PlvReq {
            req_id: 2,
            ra_id: 2,
            i_min: 5,
            i_max: 15,
            j_max: 7,
        };

        // Create a new PayloadRaToLa instance using the new method
        let payload_ra_to_la = PayloadRaToLa::new(
            "sample_request_id".to_string(),
            vec![plv_req1.clone(), plv_req2.clone()],
        );

        // Assert that the fields of the created instance match the expected values
        assert_eq!(payload_ra_to_la.request_id, "sample_request_id");
        assert_eq!(payload_ra_to_la.plv_reqs, vec![plv_req1, plv_req2]);
    }
}
