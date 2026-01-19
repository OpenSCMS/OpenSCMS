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

use crate::core_types::PlvToLsMap;
use crate::laconfig::LaConfig;
use aes::Aes128;
use aes::cipher::BlockEncryptMut;
use aes::cipher::KeyInit;
use aes::cipher::generic_array::GenericArray;
use scmscommon::core_types::linkage_seed::Ls;
use scmscommon::core_types::plv_request::PlvReq;
use scmscommon::core_types::pre_linkage_value::Plv;
use scmscommon::xor_16_bytes_array;

/// This function receives a list of LS, and it returns a list of PLVs.
/// 4 periods (i.e. i_max - i_min + 1 = 4) and we have j_max=10 then this function
/// will generate 40 PLVs in a single list.
/// request_id is unique for each bundle of caterpillars that the RA sent.
/// req_id is unique to each caterpillar in the bundle. That is, each PlvReq has one req_id.
#[allow(dead_code)]
pub fn gen_pre_linkage_values(
    ls_list: Vec<Ls>,
    plv_req: PlvReq,
    request_id: String,
) -> (Vec<Plv>, Vec<PlvToLsMap>) {
    let req_id = plv_req.req_id;
    let mut plvs: Vec<Plv> = Vec::new(); // Vector to hold the Plv values
    let mut plv_to_ls_map_list: Vec<PlvToLsMap> = Vec::new(); // This vector is later used for revocation
    for ls in ls_list {
        for j_index in 0..plv_req.j_max {
            let value = gen_plv_value(j_index, ls.clone());
            let plv = Plv::new(value, ls.i_index, j_index);
            plvs.push(plv.clone());
            let plv_to_ls_map =
                PlvToLsMap::new(plv.value, ls.value, ls.i_index, req_id, request_id.clone());
            plv_to_ls_map_list.push(plv_to_ls_map);
        }
    }

    (plvs, plv_to_ls_map_list)
}

#[allow(dead_code)]
fn gen_plv_value(j_index: u64, ls: Ls) -> [u8; 16] {
    let la_id_bytes = LaConfig::global().la_id.to_le_bytes();
    let j_index_bytes = j_index.to_le_bytes();

    let mut concat_array = [0u8; 16];
    concat_array[..8].copy_from_slice(&la_id_bytes);
    concat_array[8..].copy_from_slice(&j_index_bytes);

    // Encrypt the concatenated array using AES ECB with ls.value as the key
    let ciphertext = encrypt_aes_ecb(&concat_array, &ls.value);
    // The justification for doing the XOR below:
    // From IEEE Std 1609.2™-2022 - D.7 Test vectors for Linkage Values lv(i,j)
    // We need to do a XOR: plv1(0,j) = AES output XOR AES input (128 bits)
    // It uses Davies-Meyer mode
    xor_16_bytes_array(&ciphertext, &concat_array)
}

#[allow(dead_code)]
fn encrypt_aes_ecb(plaintext: &[u8; 16], key: &[u8; 16]) -> [u8; 16] {
    // Create AES cipher with the provided key
    let mut cipher = Aes128::new(GenericArray::from_slice(key));

    // Encrypt the plaintext using AES ECB mode
    let mut ciphertext = *plaintext; // Start with the plaintext

    // Encrypt the plaintext block using ECB mode
    cipher.encrypt_block_mut(GenericArray::from_mut_slice(&mut ciphertext));

    ciphertext
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helper::setup_la_conf;

    #[test]
    fn test_xor_16_bytes_array() {
        // Define two input arrays
        let a: [u8; 16] = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        let b: [u8; 16] = [
            0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10, 0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45,
            0x23, 0x01,
        ];

        // Define the expected result
        let expected_result: [u8; 16] = [
            0xEC, 0xE8, 0xEC, 0xE0, 0xEC, 0xE8, 0xEC, 0xE0, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE, 0xEE,
            0xEE, 0xEE,
        ];

        // Calculate the actual result
        let result = xor_16_bytes_array(&a, &b);

        // Assert that the actual result matches the expected result
        assert_eq!(result, expected_result);
    }

    #[test]
    fn test_encrypt_aes_ecb() {
        // Define a plaintext and key for encryption
        let plaintext: [u8; 16] = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
            0xCD, 0xEF,
        ];
        let key: [u8; 16] = [
            0x1F, 0x3C, 0x5A, 0x78, 0x9E, 0xBA, 0xDF, 0x01, 0x10, 0x24, 0x4C, 0x6E, 0x80, 0x9D,
            0xAE, 0xBF,
        ];

        // Encrypt the plaintext using AES ECB mode
        let ciphertext = encrypt_aes_ecb(&plaintext, &key);

        // Manually encrypt the plaintext using external AES encryption tool
        // Ensure that the computed ciphertext matches the expected ciphertext
        let expected_ciphertext: [u8; 16] = [
            0x0A, 0x66, 0x4F, 0x6F, 0x45, 0xAC, 0x1B, 0x88, 0xB5, 0x50, 0xCD, 0xE5, 0xD4, 0xF4,
            0x43, 0x21,
        ];
        assert_eq!(ciphertext, expected_ciphertext);
    }

    #[test]
    fn test_gen_plv_value() {
        setup_la_conf();
        // Define a test value for j_index and ls
        let j_index = 4;
        let ls = Ls::new(789, [0x11; 16]); // Example ls value

        // Call the gen_plv_value function
        let result = gen_plv_value(j_index, ls);

        // Define the expected result
        let expected_result: [u8; 16] = [
            0xE4, 0x00, 0x25, 0x9E, 0x3A, 0x33, 0x55, 0xBA, 0xE5, 0xAA, 0xCE, 0xD7, 0x93, 0xAC,
            0xF7, 0x42,
        ];

        // Assert that the actual result matches the expected result
        assert_eq!(result, expected_result);
    }
}
