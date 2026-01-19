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

use aes::cipher::BlockEncryptMut;
use aes::cipher::block_padding::NoPadding;
use core::fmt;
use num_bigint::BigUint;
use num_traits::Num;
use p256::EncodedPoint;
use p256::NistP256;
use p256::SecretKey;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use p256::elliptic_curve::ProjectivePoint;
use p256::elliptic_curve::ops::Add;
use p256::elliptic_curve::sec1::FromEncodedPoint;
use p256::elliptic_curve::sec1::ToEncodedPoint;
use scmscommon::errors::ConversionError;
use scmscommon::vec_to_array_16;
use scmscommon::xor_16_bytes_array;
use sha2::digest::KeyInit;

#[derive(Debug)]
pub enum OperationType {
    Sign,
    Encrypt,
}

fn into_be_4bytes(i_index: u64) -> [u8; 4] {
    let i_index = i_index as u32;
    i_index.to_be_bytes()
}

fn into_be_16bytes(i_index: u128) -> [u8; 16] {
    i_index.to_be_bytes()
}

fn vec_to_u128_be(vec: Vec<u8>) -> u128 {
    // Ensure the vector has at most 16 bytes
    assert!(vec.len() <= 16, "Vector should have at most 16 bytes");

    // Pad the vector with zeros at the start if it has fewer than 16 bytes
    let mut padded_vec = vec.clone();
    while padded_vec.len() < 16 {
        padded_vec.insert(0, 0);
    }

    // Construct the u128 value
    let mut result: u128 = 0;
    for byte in padded_vec {
        result = (result << 8) | (byte as u128);
    }

    result
}

/// Create the 128 bit bit string xS (for the signing expansion function fS)  as
/// follows: xS = (1^32 || i || j || 0^32)
fn derive_encrypt(i_index: u64, j_index: u64) -> u128 {
    let i_bytes = into_be_4bytes(i_index);
    let j_bytes = into_be_4bytes(j_index);

    // Define zeros and ones byte arrays (32 bits each)
    let zeros: [u8; 4] = [0x00; 4];
    let ones: [u8; 4] = [0xFF; 4];

    // Create a vector to hold the concatenated result
    let mut hold_values =
        Vec::with_capacity(ones.len() + i_bytes.len() + j_bytes.len() + zeros.len());

    // Concatenate the byte arrays
    hold_values.extend_from_slice(&ones);
    hold_values.extend_from_slice(&i_bytes);
    hold_values.extend_from_slice(&j_bytes);
    hold_values.extend_from_slice(&zeros);

    vec_to_u128_be(hold_values)
}

/// Create the 128 bit bit string xS (for the signing expansion function fS)  as
/// follows: xS = (0^32 || i || j || 0^32)
fn derive_sign(i_index: u64, j_index: u64) -> u128 {
    let i_bytes = into_be_4bytes(i_index);
    let j_bytes = into_be_4bytes(j_index);

    // Define a zeros byte array (32 bits or 4 bytes)
    let zeros: [u8; 4] = [0x00; 4];

    // Create a vector to hold the concatenated result
    let mut hold_values =
        Vec::with_capacity(zeros.len() + i_bytes.len() + j_bytes.len() + zeros.len());

    // Concatenate the byte arrays
    hold_values.extend_from_slice(&zeros);
    hold_values.extend_from_slice(&i_bytes);
    hold_values.extend_from_slice(&j_bytes);
    hold_values.extend_from_slice(&zeros);

    vec_to_u128_be(hold_values)
}

fn combine_f_value_with_i_and_j(
    operation_type: OperationType,
    f_value: [u8; 16],
    i_index: u64,
    j_index: u64,
) -> Result<Vec<u8>, ConversionError> {
    let x_s = match operation_type {
        OperationType::Encrypt => derive_encrypt(i_index, j_index),
        OperationType::Sign => derive_sign(i_index, j_index),
    };

    type Aes128EcbEnc = ecb::Encryptor<aes::Aes128>;
    let key = f_value;

    // https://docs.rs/ecb/latest/ecb/

    // (Symm(kS, xS+1) ⊕ (xS+1))
    let xs_1_bytes = into_be_16bytes(x_s + 1);
    let xs_1_bytes_enc = vec_to_array_16(
        Aes128EcbEnc::new(&key.into()).encrypt_padded_vec_mut::<NoPadding>(&xs_1_bytes),
    )?;
    let xs_1_xor = xor_16_bytes_array(&xs_1_bytes, &xs_1_bytes_enc);

    // (Symm(kS, xS+2) ⊕ (xS+2))
    let xs_2_bytes = into_be_16bytes(x_s + 2);
    let xs_2_bytes_enc = vec_to_array_16(
        Aes128EcbEnc::new(&key.into()).encrypt_padded_vec_mut::<NoPadding>(&xs_2_bytes),
    )?;
    let xs_2_xor = xor_16_bytes_array(&xs_2_bytes, &xs_2_bytes_enc);

    // (Symm(kS, xS+3) ⊕ (xS+3))
    let xs_3_bytes = into_be_16bytes(x_s + 3);
    let xs_3_bytes_enc = vec_to_array_16(
        Aes128EcbEnc::new(&key.into()).encrypt_padded_vec_mut::<NoPadding>(&xs_3_bytes),
    )?;
    let xs_3_xor = xor_16_bytes_array(&xs_3_bytes, &xs_3_bytes_enc);

    // yS = (Symm(kS, xS+1) ⊕ (xS+1)) || (Symm(kS, xS+2) ⊕ (xS+2)) || (Symm(kS, xS+3) ⊕ (xS+3)).
    // Create a vector with the required capacity (48 bytes in total)
    let mut y_s: Vec<u8> = Vec::with_capacity(48);

    // Append each array to the vector
    y_s.extend_from_slice(&xs_1_xor);
    y_s.extend_from_slice(&xs_2_xor);
    y_s.extend_from_slice(&xs_3_xor);

    // start part for debuging --------------------------
    let hex_string = hex::encode(&y_s);
    log::debug!("y_s hex_string: {:?}", hex_string);
    log::debug!(
        "combine_f_value_with_i_and_j: i_index, j_index, type: {:?}, {:?}, {:?}",
        i_index,
        j_index,
        operation_type
    );

    Ok(y_s)
}

fn generate_priv_key_for_expansion_function(
    y_s: Vec<u8>,
) -> Result<(SigningKey, Vec<u8>), ExpansionFunctionError> {
    let secret_bytes = y_s;

    // Convert the byte array to a BigUint
    let num = BigUint::from_bytes_be(&secret_bytes);

    // Order of the p256 curve
    let curve_order = BigUint::from_str_radix(
        "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551",
        16,
    )
    .unwrap();

    // Calculate the private key value mod the curve order
    let private_key_int = num % curve_order;

    // Convert the private key value back to bytes
    let private_key_bytes = private_key_int.to_bytes_be();

    // Ensure the private key bytes are exactly 32 bytes long
    let mut fixed_private_key_bytes = vec![0u8; 32];
    fixed_private_key_bytes[(32 - private_key_bytes.len())..].copy_from_slice(&private_key_bytes);

    // Convert Vec<u8> to [u8; 32]
    let fixed_private_key_array: [u8; 32] = fixed_private_key_bytes
        .try_into()
        .expect("Slice with incorrect length");

    // Create the SecretKey from the byte array
    let secret_key =
        SecretKey::from_bytes(&fixed_private_key_array.into()).expect("Failed to create SecretKey");

    // Create the SigningKey from the byte array
    let private_key = SigningKey::from(&secret_key);

    Ok((private_key, private_key_bytes))
}

pub fn generate_public_key(
    operation_type: OperationType,
    f_value: [u8; 16],
    i_index: u64,
    j_index: u64,
) -> Result<(VerifyingKey, Vec<u8>), ExpansionFunctionError> {
    log::debug!(
        "Start Expanding public key (generate_public_key): i_index, j_index, f_value, type: {:?}, {:?}, {:?}, {:?}",
        i_index,
        j_index,
        f_value,
        operation_type
    );
    let y_s: Vec<u8> = combine_f_value_with_i_and_j(operation_type, f_value, i_index, j_index)?;
    log::debug!("Combined fij {:?}", y_s);

    let (priv_key, private_key_info) = generate_priv_key_for_expansion_function(y_s)?;
    log::debug!("Generated private key: {:?}", priv_key);

    let encoded_point: EncodedPoint = priv_key.verifying_key().to_encoded_point(false);
    log::debug!("Derived public key from expansion: {:?}", encoded_point);

    let pub_key = VerifyingKey::from_encoded_point(&encoded_point)?;
    Ok((pub_key, private_key_info))
}

pub fn create_cocoon_pub_key(
    f_encrypt: Option<[u8; 16]>,
    f_sign: Option<[u8; 16]>,
    i_index: u64,
    j_index: u64,
    caterpillar_pub_key_encrypt: Option<VerifyingKey>,
    caterpillar_pub_key_sign: Option<VerifyingKey>,
) -> Result<(Option<VerifyingKey>, Option<VerifyingKey>, Option<Vec<u8>>), ExpansionFunctionError> {
    log::debug!(
        "create_cocoon_pub_key: i_index, j_index: {:?}, {:?}",
        i_index,
        j_index,
    );
    log::debug!(
        "create_cocoon_pub_key: caterpillar_pub_key_encrypt, caterpillar_pub_key_sign: {:?}, {:?}",
        caterpillar_pub_key_encrypt,
        caterpillar_pub_key_sign
    );
    log::debug!(
        "create_cocoon_pub_key: f_encrypt, f_sign: {:?}, {:?}",
        f_encrypt,
        f_sign
    );

    let cocoon_pub_key_encrypt: Option<VerifyingKey> = if let Some(x) = f_encrypt {
        let caterpillar_pub_key_encrypt = caterpillar_pub_key_encrypt.unwrap();
        let (expansion_function_pub_key, _) =
            generate_public_key(OperationType::Encrypt, x, i_index, j_index)?;
        let cocoon_pub_key_encrypt =
            add_verifying_keys(&expansion_function_pub_key, &caterpillar_pub_key_encrypt)?;
        Some(cocoon_pub_key_encrypt)
    } else {
        None
    };

    let (cocoon_pub_key_sign, private_key_info) = if let Some(x) = f_sign {
        let caterpillar_pub_key_sign = caterpillar_pub_key_sign.unwrap();
        let (expansion_function_pub_key, private_key_info) =
            generate_public_key(OperationType::Sign, x, i_index, j_index)?;
        let cocoon_pub_key_sign =
            add_verifying_keys(&expansion_function_pub_key, &caterpillar_pub_key_sign)?;
        let cocoon_pub_key_sign = Some(cocoon_pub_key_sign);
        (cocoon_pub_key_sign, Some(private_key_info))
    } else {
        (None, None)
    };

    Ok((
        cocoon_pub_key_encrypt,
        cocoon_pub_key_sign,
        private_key_info,
    ))
}

fn add_verifying_keys(
    vk1: &VerifyingKey,
    vk2: &VerifyingKey,
) -> Result<VerifyingKey, ExpansionFunctionError> {
    let vk1 = vk1.to_encoded_point(false);
    let vk2 = vk2.to_encoded_point(false);

    let vk1 = ProjectivePoint::<NistP256>::from_encoded_point(&vk1).unwrap();
    let vk2 = ProjectivePoint::<NistP256>::from_encoded_point(&vk2).unwrap();

    let vk_add = vk1.add(vk2);
    let vk_add_encoded_point = vk_add.to_encoded_point(false);
    let vk_add_verifying_key = VerifyingKey::from_encoded_point(&vk_add_encoded_point)?;
    Ok(vk_add_verifying_key)
}

#[derive(Debug)]
pub struct ExpansionFunctionError {
    message: String,
}

impl ExpansionFunctionError {
    pub fn new(message: &str) -> Self {
        ExpansionFunctionError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for ExpansionFunctionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<p256::ecdsa::Error> for ExpansionFunctionError {
    fn from(err: p256::ecdsa::Error) -> Self {
        ExpansionFunctionError::new(&format!("p256::ecdsa::Error: {}", err))
    }
}

impl From<ConversionError> for ExpansionFunctionError {
    fn from(err: ConversionError) -> Self {
        ExpansionFunctionError::new(&format!("ConversionError: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_public_key() {
        let operation_type = OperationType::Encrypt;
        let f_value = [
            94, 123, 165, 221, 24, 127, 114, 157, 42, 9, 183, 106, 116, 227, 33, 204,
        ];
        let i_index = 7590;
        let j_index = 0;

        let result = generate_public_key(operation_type, f_value, i_index, j_index);
        assert!(result.is_ok(), "Expected Ok, but got Err: {:?}", result);
        let (expanded_pub_key, _) = result.unwrap();

        // Get bytes
        let expanded_pub_key_compressed_point = expanded_pub_key.to_encoded_point(true);
        let expanded_pub_key_bytes = expanded_pub_key_compressed_point.as_bytes();
        // To Hex
        let expanded_pub_key_hex = hex::encode(expanded_pub_key_bytes);
        let expected_hex = "0331131301dd498695af1a01741676e89d47db65e4cb06d8cc87ae6c55261aa8fb";

        assert_eq!(expanded_pub_key_hex, expected_hex);

        // Test adding
        let pub_key_encryp: Vec<u8> = vec![
            4, 209, 149, 60, 128, 119, 102, 179, 251, 58, 140, 112, 33, 35, 117, 142, 104, 62, 232,
            26, 210, 241, 231, 242, 241, 121, 224, 36, 5, 222, 17, 230, 170, 61, 184, 26, 232, 72,
            157, 4, 4, 95, 102, 167, 38, 9, 130, 112, 214, 51, 187, 220, 36, 126, 216, 60, 186,
            119, 217, 195, 115, 55, 155, 252, 169,
        ];
        let e_verifying_key =
            VerifyingKey::from_encoded_point(&EncodedPoint::from_bytes(&pub_key_encryp).unwrap())
                .unwrap();

        assert_eq!(
            e_verifying_key.to_encoded_point(false).as_bytes(),
            pub_key_encryp
        );

        // Add
        let result = add_verifying_keys(&expanded_pub_key, &e_verifying_key);
        assert!(result.is_ok());

        let added_pub_key = result.unwrap();
        let added_pub_key_compressed_point = added_pub_key.to_encoded_point(true);
        let added_pub_key_bytes = added_pub_key_compressed_point.as_bytes();
        let added_pub_key_hex = hex::encode(added_pub_key_bytes);
        let expected_added_hex =
            "031cb03fd5fb8faf7a23cd339da6923c4e44b6a7f6b135acf657808b8ff68c7920";

        assert_eq!(added_pub_key_hex, expected_added_hex);
    }

    #[test]
    fn test_generate_public_key_sign() {
        let operation_type = OperationType::Sign;
        let f_value = [
            73, 187, 227, 126, 215, 155, 29, 95, 247, 47, 157, 176, 187, 45, 21, 72,
        ];
        let i_index = 7596;
        let j_index = 0;

        let result = generate_public_key(operation_type, f_value, i_index, j_index);
        assert!(result.is_ok(), "Expected Ok, but got Err: {:?}", result);
        let (expanded_pub_key, _) = result.unwrap();

        // Get bytes
        let expanded_pub_key_compressed_point = expanded_pub_key.to_encoded_point(true);
        let expanded_pub_key_bytes = expanded_pub_key_compressed_point.as_bytes();
        // To Hex
        let expanded_pub_key_hex = hex::encode(expanded_pub_key_bytes);
        let expected_hex = "03d3cbcbf48ed450757b355bfb91177d9339fa41a5c46f1571f59fe42976028b63";

        assert_eq!(expanded_pub_key_hex, expected_hex);

        // Test adding
        let pub_key_sign: Vec<u8> = vec![
            4, 175, 150, 245, 125, 155, 231, 254, 91, 69, 195, 171, 50, 7, 43, 140, 19, 106, 236,
            131, 177, 24, 190, 14, 0, 212, 165, 121, 162, 58, 128, 170, 124, 241, 59, 214, 53, 199,
            186, 241, 246, 186, 143, 17, 200, 119, 193, 134, 119, 209, 127, 158, 69, 231, 234, 45,
            237, 3, 36, 60, 224, 98, 187, 26, 153,
        ];
        let s_verifying_key =
            VerifyingKey::from_encoded_point(&EncodedPoint::from_bytes(&pub_key_sign).unwrap())
                .unwrap();

        assert_eq!(
            s_verifying_key.to_encoded_point(false).as_bytes(),
            pub_key_sign
        );

        // Add
        let result = add_verifying_keys(&expanded_pub_key, &s_verifying_key);
        assert!(result.is_ok());

        let added_pub_key = result.unwrap();
        let added_pub_key_compressed_point = added_pub_key.to_encoded_point(true);
        let added_pub_key_bytes = added_pub_key_compressed_point.as_bytes();
        let added_pub_key_hex = hex::encode(added_pub_key_bytes);
        let expected_added_hex =
            "02f6f6061761eb6c00303572bd654315cedaf9ebf275e522135c3ca9c55b34e955";

        assert_eq!(added_pub_key_hex, expected_added_hex);
    }

    #[test]
    fn test_combine_f_value_with_i_and_j_encrypt() {
        // Define test inputs
        let operation_type = OperationType::Encrypt;
        let f_value = [
            94, 123, 165, 221, 24, 127, 114, 157, 42, 9, 183, 106, 116, 227, 33, 204,
        ];
        let i_index = 7590;
        let j_index = 0;

        // Call the function
        let result = combine_f_value_with_i_and_j(operation_type, f_value, i_index, j_index);

        // Check the result
        assert!(result.is_ok());
        let y_s = result.unwrap();
        assert_eq!(y_s.len(), 48);

        // Compare Hex
        let hex_string = hex::encode(&y_s);
        let expected_hex_string = "e30a331dee3a609c31d0866b2012b2544b6421f7a32f2ffc4002e2759a5eee46411840a2c5cddfa8ec7794387bd8de68";
        assert_eq!(hex_string, expected_hex_string);
    }
}
