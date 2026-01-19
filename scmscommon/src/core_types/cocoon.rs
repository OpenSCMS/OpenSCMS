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
use crate::core_types::pre_linkage_value::Eplv;
use crate::{deserialize_option_verifying_key, serialize_option_verifying_key};
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// There is one CocoonRequest for each Caterpillar (hence, for each EE request).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CocoonRequest {
    // fields
    pub req_id: u64,
    #[serde(
        serialize_with = "serialize_option_verifying_key",
        deserialize_with = "deserialize_option_verifying_key"
    )]
    pub pub_key_sign: Option<VerifyingKey>,
    #[serde(
        serialize_with = "serialize_option_verifying_key",
        deserialize_with = "deserialize_option_verifying_key"
    )]
    pub pub_key_encrypt: Option<VerifyingKey>,
    pub private_key_info: Option<Vec<u8>>,
    pub eplv_pair: Vec<Eplv>, // this field is called eplv_pair because we need to get an EPLV from LA1 and LA2. (in the case of 2 LAs)
    pub i_index: u64,
}

impl CocoonRequest {
    pub fn new(
        req_id: u64,
        pub_key_sign: Option<VerifyingKey>,
        pub_key_encrypt: Option<VerifyingKey>,
        private_key_info: Option<Vec<u8>>,
        eplv_pair: Vec<Eplv>,
        i_index: u64,
    ) -> Self {
        Self {
            req_id,
            pub_key_sign,
            pub_key_encrypt,
            private_key_info,
            eplv_pair,
            i_index,
        }
    }
}

/// This struct is on the level of the Certificate. It has values for
/// i_index and j_index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ClientRequestsMappingEntry {
    pub i_index: u64,
    pub j_index: u64,
}

/// This struct is on the level of the certificate request, and the caterpillar.
/// Because there is one Caterpillar for each Certificate Request.
/// Each Certificate Request will output multiple Certificates.
/// Each vehicle could have made multiple certificate requests.
/// Given that it has a vid and hash_id values which are combined unique for each
/// EE's certificate request.
/// The ClientRequestsMappingEntry on the other hand will be in the level of each
/// Certificate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct ClientRequestsMapping {
    pub requests: Vec<ClientRequestsMappingEntry>,
    pub vid: u64,
    pub hash_id: String,
    pub exp_type: ExpansionType,
    pub req_id: u64,
}

impl ClientRequestsMapping {
    pub fn new(
        requests: Vec<ClientRequestsMappingEntry>,
        vid: u64,
        hash_id: String,
        exp_type: ExpansionType,
        req_id: u64,
    ) -> Self {
        Self {
            requests,
            vid,
            hash_id,
            exp_type,
            req_id,
        }
    }
}

impl ClientRequestsMappingEntry {
    pub fn new(i_index: u64, j_index: u64) -> Self {
        Self { i_index, j_index }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use p256::ecdsa::{SigningKey, VerifyingKey};
    use rand::rngs::OsRng;

    #[test]
    fn test_cocoon_request_new() {
        let req_id: u64 = 1;
        // Generate a random secret key
        let signing_key = SigningKey::random(&mut OsRng);
        let private_key_info = Some(signing_key.to_bytes().to_vec());
        let verifying_key = VerifyingKey::from(&signing_key);
        let pub_key_sign = Some(verifying_key);
        let pub_key_encrypt = Some(verifying_key);

        // building eplv
        let enc_value = [1u8; 32];
        let ephemeral_pub_key = vec![2u8; 32];
        let nonce = [5u8; 12];
        let i_index = 123;
        let j_index = 456;

        let eplv = Eplv::new(
            enc_value,
            ephemeral_pub_key.clone(),
            nonce,
            i_index,
            j_index,
        );

        let eplv_pair = vec![eplv.clone(), eplv.clone()];
        let i_index: u64 = 10;

        // Create a new CocoonRequest using the new method
        let cocoon_request = CocoonRequest::new(
            req_id,
            pub_key_sign,
            pub_key_encrypt,
            private_key_info.clone(),
            eplv_pair.clone(),
            i_index,
        );

        // Verify that each field is correctly assigned
        assert_eq!(cocoon_request.req_id, req_id);
        assert_eq!(cocoon_request.pub_key_sign, pub_key_sign);
        assert_eq!(cocoon_request.pub_key_encrypt, pub_key_encrypt);
        assert_eq!(cocoon_request.private_key_info, private_key_info);
        assert_eq!(cocoon_request.eplv_pair, eplv_pair);
        assert_eq!(cocoon_request.i_index, i_index);
    }
}
