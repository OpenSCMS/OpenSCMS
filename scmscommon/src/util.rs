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

use crate::errors::ConversionError;
use p256::PublicKey;
use p256::SecretKey;
use p256::elliptic_curve::sec1::ToEncodedPoint;

pub fn vec_to_array_16(vec: Vec<u8>) -> Result<[u8; 16], ConversionError> {
    // Ensure the vector has exactly 16 elements
    let mut array = [0u8; 16]; // Create a zero-initialized array
    let len = vec.len();
    // Check if the vector has exactly 16 elements
    if vec.len() != 16 {
        // If not, return a ConversionError
        return Err(ConversionError::new(&format!(
            "Expected a vector of length 16, but got length {}",
            vec.len()
        )));
    } else {
        array[..len].copy_from_slice(&vec); // Copy all elements if vec has 16 or fewer elements
    }

    Ok(array)
}

pub fn vec_to_array_32(vec: Vec<u8>) -> Result<[u8; 32], ConversionError> {
    // Ensure the vector has exactly 32 elements
    let mut array = [0u8; 32]; // Create a zero-initialized array
    let len = vec.len();

    // Check if the vector has exactly 32 elements
    if vec.len() != 32 {
        // If not, return a ConversionError
        return Err(ConversionError::new(&format!(
            "Expected a vector of length 32, but got length {}",
            vec.len()
        )));
    } else {
        array[..len].copy_from_slice(&vec); // Copy all elements if vec has 64 or fewer elements
    }

    Ok(array)
}

pub fn vec_to_array_64(vec: Vec<u8>) -> Result<[u8; 64], ConversionError> {
    // Ensure the vector has exactly 64 elements
    let mut array = [0u8; 64]; // Create a zero-initialized array
    let len = vec.len();

    // Check if the vector has exactly 64 elements
    if vec.len() != 64 {
        // If not, return a ConversionError
        return Err(ConversionError::new(&format!(
            "Expected a vector of length 64, but got length {}",
            vec.len()
        )));
    } else {
        array[..len].copy_from_slice(&vec); // Copy all elements if vec has 64 or fewer elements
    }

    Ok(array)
}

pub fn xor_16_bytes_array(a: &[u8; 16], b: &[u8; 16]) -> [u8; 16] {
    let mut result = [0u8; 16];
    for i in 0..16 {
        result[i] = a[i] ^ b[i];
    }
    result
}

pub fn convert_secret_key_into_vec_32(secret_key: SecretKey) -> Result<[u8; 32], ConversionError> {
    let vec_form = secret_key.to_bytes().to_vec();
    let array = vec_to_array_32(vec_form)?;
    Ok(array)
}

pub fn convert_public_key_into_vec_64(public_key: PublicKey) -> Result<[u8; 64], ConversionError> {
    let vec_form = public_key.to_encoded_point(true).to_bytes().to_vec();
    let array = vec_to_array_64(vec_form)?;
    Ok(array)
}
