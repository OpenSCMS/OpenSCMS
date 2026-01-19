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

use crate::core_types::{CertificateType, CocoonRequest, ExpansionType, NonButterflyRequest};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct PayloadRaToAca {
    pub exp_type: ExpansionType,
    pub certificate_type: CertificateType,
    pub ra_id: u32,
    pub cocoon_requests: Vec<CocoonRequest>,
    // This is only used in case of Nonbutterfly request
    pub non_butterfly_request: Option<NonButterflyRequest>,
}

impl PayloadRaToAca {
    pub fn new(
        exp_type: ExpansionType,
        ra_id: u32,
        certificate_type: CertificateType,
        cocoon_requests: Vec<CocoonRequest>,
        non_butterfly_request: Option<NonButterflyRequest>,
    ) -> Self {
        Self {
            exp_type,
            certificate_type,
            ra_id,
            cocoon_requests,
            non_butterfly_request,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_payload_ra_to_aca() {
        let exp_type = ExpansionType::Original;
        let certificate_type = CertificateType::Explicit;
        let ra_id = 1;
        let cocoon_requests = vec![];
        let payload = PayloadRaToAca::new(
            exp_type,
            ra_id,
            certificate_type,
            cocoon_requests.clone(),
            None,
        );
        assert_eq!(payload.exp_type, exp_type);
        assert_eq!(payload.ra_id, ra_id);
        assert_eq!(payload.cocoon_requests, cocoon_requests);
    }
}
