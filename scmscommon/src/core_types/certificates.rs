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

use crate::core_types::ExpansionType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Certificate {
    pub exp_type: ExpansionType,
    pub req_id: u64,
    pub encoded_binary: Vec<u8>,
    pub certificate_validity_begin: u32,
}

impl Certificate {
    pub fn new(
        exp_type: ExpansionType,
        req_id: u64,
        encoded_binary: Vec<u8>,
        certificate_validity_begin: u32,
    ) -> Self {
        Self {
            exp_type,
            req_id,
            encoded_binary,
            certificate_validity_begin,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_new() {
        let exp_type = ExpansionType::Original;
        let request_id = 2;
        let encoded_binary = vec![1, 2, 3];
        let certificate_validity_begin = 4;
        let certificate = Certificate::new(
            exp_type,
            request_id,
            encoded_binary.clone(),
            certificate_validity_begin,
        );
        assert_eq!(certificate.exp_type, exp_type);
        assert_eq!(certificate.req_id, request_id);
        assert_eq!(certificate.encoded_binary, encoded_binary);
        assert_eq!(
            certificate.certificate_validity_begin,
            certificate_validity_begin
        );
    }
}
