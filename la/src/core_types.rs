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

/// In the short-term revocation the MA will ask the LA
/// which LS is associated with which PLVs. Then the LA needs to send this
/// information to back to the MA, which then includes the LS information in the CRL.
/// The req_id and request_id are used for the long term revocation.
/// The LA needs to send those to the MA, and then MA send them to the
/// RA so that it can remove the enrollment certificate of the vehicle in question.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct PlvToLsMap {
    pub plv: [u8; 16],
    pub ls: [u8; 16],
    pub i_index: u64,
    pub req_id: u64,
    pub request_id: String,
}

impl PlvToLsMap {
    pub fn new(plv: [u8; 16], ls: [u8; 16], i_index: u64, req_id: u64, request_id: String) -> Self {
        PlvToLsMap {
            plv,
            ls,
            i_index,
            req_id,
            request_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_plv_to_ls_map() {
        // Create test data
        let plv = [1; 16];
        let ls = [2; 16];
        let i_index = 123456;
        let req_id = 456;
        let request_id = "example_request_id".to_string();
        // Call the new method to create a PlvToLsMap instance
        let plv_to_ls_map = PlvToLsMap::new(plv, ls, i_index, req_id, request_id);

        // Verify that the fields of the created instance match the input data
        assert_eq!(plv_to_ls_map.plv, [1; 16]);
        assert_eq!(plv_to_ls_map.ls, [2; 16]);
        assert_eq!(plv_to_ls_map.i_index, 123456);
    }
}
