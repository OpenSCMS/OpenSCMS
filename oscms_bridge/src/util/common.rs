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

use crate::errors::HashIdGenerationError;
use crate::{OscmsOctetBuffer, oscms_octet_buffer_init_from_buffer};
use hex::encode;
use sha2::{Digest, Sha256};

pub fn hashed_binary(data: Vec<u8>) -> Result<[u8; 8], HashIdGenerationError> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    let result_slice = result.as_slice();
    let result_length = result.len();
    match result_slice[result_length - 8..].try_into() {
        Ok(hash) => Ok(hash),
        Err(_) => Err(HashIdGenerationError::FailedToTryInto),
    }
}

pub fn hashed_binary_hex(data: Vec<u8>) -> Result<String, HashIdGenerationError> {
    let hashed_data = hashed_binary(data)?;
    Ok(encode(hashed_data))
}

pub fn oscms_octet_buffer_to_vec(octet_buffer: &OscmsOctetBuffer) -> Vec<u8> {
    // if empty, return empty vec
    if octet_buffer.length == 0 {
        return Vec::new();
    }
    unsafe { std::slice::from_raw_parts(octet_buffer.data, octet_buffer.length).to_vec() }
}

pub fn initialize_oscms_new_octet_buffer_from_vec(mut data: Vec<u8>) -> OscmsOctetBuffer {
    let mut buffer = OscmsOctetBuffer {
        data: std::ptr::null_mut(),
        length: 0,
    };

    unsafe {
        oscms_octet_buffer_init_from_buffer(&mut buffer, data.as_mut_ptr(), data.len());
    }
    buffer
}

pub fn initialize_empty_oscms_octet_buffer() -> OscmsOctetBuffer {
    OscmsOctetBuffer {
        data: std::ptr::null_mut(),
        length: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_hashed_binary() {
        let encoded = vec![
            3, 131, 130, 1, 79, 0, 2, 135, 128, 64, 2, 35, 247, 10, 162, 0, 0, 131, 0, 0, 0, 0, 0,
            0, 0, 48, 57, 132, 0, 168, 128, 128, 130, 93, 132, 37, 152, 245, 139, 78, 36, 188, 245,
            194, 146, 55, 125, 19, 98, 72, 33, 176, 187, 105, 17, 224, 51, 185, 98, 84, 254, 250,
            94, 1, 110, 128, 128, 89, 8, 51, 13, 29, 156, 4, 75, 205, 64, 254, 213, 122, 106, 121,
            88, 0, 128, 130, 232, 123, 253, 65, 152, 137, 63, 12, 162, 241, 34, 12, 81, 126, 186,
            44, 68, 206, 239, 191, 176, 87, 209, 62, 251, 170, 17, 216, 247, 119, 135, 126, 128,
            148, 39, 105, 159, 155, 119, 141, 148, 111, 8, 206, 243, 236, 191, 108, 207, 129, 1, 1,
            128, 3, 0, 128, 69, 108, 7, 119, 220, 14, 191, 41, 0, 130, 8, 5, 68, 165, 74, 43, 82,
            64, 47, 1, 2, 3, 0, 0, 35, 247, 10, 162, 132, 255, 255, 128, 128, 131, 116, 57, 199,
            143, 221, 63, 138, 100, 228, 74, 123, 22, 255, 92, 61, 74, 214, 199, 97, 177, 177, 108,
            50, 95, 23, 0, 150, 42, 150, 59, 247, 59, 128, 128, 192, 84, 190, 25, 34, 156, 127, 25,
            211, 151, 81, 241, 98, 37, 213, 196, 121, 224, 9, 5, 187, 29, 189, 137, 51, 198, 22,
            69, 213, 26, 124, 11, 63, 183, 190, 141, 178, 44, 118, 115, 44, 170, 73, 39, 142, 92,
            76, 1, 42, 129, 245, 215, 158, 43, 21, 86, 251, 151, 169, 221, 37, 158, 13, 25, 128,
            128, 153, 243, 201, 134, 214, 65, 59, 98, 96, 173, 190, 243, 57, 55, 125, 25, 225, 135,
            82, 140, 0, 171, 107, 159, 135, 93, 22, 99, 158, 17, 238, 33, 160, 254, 30, 248, 124,
            122, 67, 22, 184, 160, 18, 35, 189, 162, 90, 216, 54, 167, 208, 27, 108, 29, 186, 122,
            251, 89, 75, 174, 54, 77, 99, 145,
        ];

        match hashed_binary_hex(encoded) {
            Ok(hashed_binary_hex) => assert_eq!(hashed_binary_hex, "f91eed332a9e802c"),
            Err(e) => {
                panic!("Error: Unknown {:?}", e);
            }
        }
    }
}
