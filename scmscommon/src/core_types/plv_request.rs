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
pub struct PlvReq {
    pub req_id: u64,
    pub ra_id: u64,
    pub i_min: u64,
    pub i_max: u64,
    pub j_max: u64,
}

impl PlvReq {
    pub fn new(req_id: u64, ra_id: u64, i_min: u64, i_max: u64, j_max: u64) -> Self {
        PlvReq {
            req_id,
            ra_id,
            i_min,
            i_max,
            j_max,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_plv_req() {
        // Create a new PlvReq instance using the new method
        let plv_req = PlvReq::new(123, 1, 0, 10, 5);

        // Check that the fields of the created PlvReq instance match the provided values
        assert_eq!(plv_req.req_id, 123);
        assert_eq!(plv_req.ra_id, 1);
        assert_eq!(plv_req.i_min, 0);
        assert_eq!(plv_req.i_max, 10);
        assert_eq!(plv_req.j_max, 5);
    }
}
