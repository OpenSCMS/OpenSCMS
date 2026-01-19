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

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Clone, ToSchema)]
pub struct Plv {
    pub value: [u8; 16],
    pub i_index: u64,
    pub j_index: u64,
}

impl Plv {
    pub fn new(value: [u8; 16], i_index: u64, j_index: u64) -> Self {
        Plv {
            value,
            i_index,
            j_index,
        }
    }
}

/// This struct represents the encrypted value version of the Plv.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, Eq, ToSchema)]
pub struct Eplv {
    pub enc_value: [u8; 32],
    pub public_key: Vec<u8>,
    pub nonce: [u8; 12],
    pub i_index: u64,
    pub j_index: u64,
}

impl Eplv {
    pub fn new(
        enc_value: [u8; 32],
        public_key: Vec<u8>,
        nonce: [u8; 12],
        i_index: u64,
        j_index: u64,
    ) -> Self {
        Self {
            enc_value,
            public_key,
            nonce,
            i_index,
            j_index,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plv_serialization() {
        // Create a Plv struct
        let plv = Plv::new([1; 16], 42, 99);

        // Serialize the Plv struct into JSON
        let json = serde_json::to_string(&plv).unwrap();

        // Deserialize JSON back into Plv struct
        let deserialized_plv: Plv = serde_json::from_str(&json).unwrap();

        // Check if the deserialized Plv matches the original Plv
        assert_eq!(plv.value, deserialized_plv.value);
        assert_eq!(plv.i_index, deserialized_plv.i_index);
        assert_eq!(plv.j_index, deserialized_plv.j_index);
    }

    #[test]
    fn test_new_eplv() {
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

        // Assert
        assert_eq!(eplv.enc_value, enc_value);
        assert_eq!(eplv.public_key, ephemeral_pub_key);
        assert_eq!(eplv.nonce, nonce);
        assert_eq!(eplv.i_index, i_index);
        assert_eq!(eplv.j_index, j_index);
    }
}
