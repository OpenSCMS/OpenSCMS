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

use crate::errors::DecryptError;
use crate::errors::DeriveSharedSecretError;
use crate::errors::GenCertificateError;
use crate::persistence::aca_certificates::latest_aca_private_key;
use aes_gcm::Aes256Gcm;
use aes_gcm::KeyInit;
use aes_gcm::aead;
use aes_gcm::aead::Aead;
use aes_gcm::aead::generic_array::GenericArray;
use openssl::bn::{BigNum, BigNumContext};
use openssl::derive::Deriver;
use openssl::ec::{EcGroup, EcKey, EcPoint};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use p256::ecdsa::VerifyingKey;
use scmscommon::core_types::pre_linkage_value::Eplv;
use scmscommon::core_types::pre_linkage_value::Plv;
use sea_orm::DatabaseConnection;

pub async fn decrypt_eplv_pair(
    eplv_pair: Vec<Eplv>,
    db: &DatabaseConnection,
) -> Result<Vec<Plv>, DecryptError> {
    // Create a new vector to store the decrypted data
    let mut plv_pair: Vec<Plv> = Vec::with_capacity(eplv_pair.len());
    let private_key_p256 = latest_aca_private_key(db)
        .await
        .map_err(|_| DecryptError::new("Error fetching latest ACA private key"))?;

    // Check if starts with 0x04 (uncompressed) if not append it
    let private_key_p256 = if private_key_p256.len() == 64 {
        let mut uncompressed = vec![0x04];
        uncompressed.extend_from_slice(&private_key_p256);
        uncompressed
    } else {
        private_key_p256.to_vec()
    };

    // Iterate over each reference to Eplv in the encrypted_data vector
    for eplv in &eplv_pair {
        let ephemeral_public_key = eplv.public_key.clone();
        let shared_secret = derive_shared_secret(ephemeral_public_key, &private_key_p256)?;
        let plv = decrypt_eplv(eplv.clone(), shared_secret).map_err(|e| {
            DecryptError::new(&format!(
                "Something wrong happened in the decrypt_eplv function: {:?}",
                e
            ))
        })?;
        plv_pair.push(plv);
    }
    Ok(plv_pair)
}

fn derive_shared_secret(
    ephemeral_public_key: Vec<u8>,
    private_key_bytes: &[u8],
) -> Result<Vec<u8>, DeriveSharedSecretError> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;
    let mut ctx = BigNumContext::new()?;

    let point = EcPoint::from_bytes(&group, &ephemeral_public_key, &mut ctx).map_err(|e| {
        DeriveSharedSecretError::new(&format!("Failed to decode public key: {}", e))
    })?;

    let private_bn = BigNum::from_slice(private_key_bytes).map_err(|e| {
        DeriveSharedSecretError::new(&format!("Failed to create BigNum from private key: {}", e))
    })?;

    let mut public_point = EcPoint::new(&group)
        .map_err(|e| DeriveSharedSecretError::new(&format!("Failed to create point: {}", e)))?;

    public_point
        .mul_generator(&group, &private_bn, &ctx)
        .map_err(|e| {
            DeriveSharedSecretError::new(&format!("Failed to calculate public key: {}", e))
        })?;

    let private_ec_key = EcKey::from_private_components(&group, &private_bn, &public_point)
        .map_err(|e| {
            DeriveSharedSecretError::new(&format!("Failed to create complete private key: {}", e))
        })?;

    let private_pkey: PKey<Private> = private_ec_key.try_into().map_err(|e| {
        DeriveSharedSecretError::new(&format!("Failed to convert private EcKey to PKey: {}", e))
    })?;

    let ephemeral_ec_key = EcKey::from_public_key(&group, &point).map_err(|e| {
        DeriveSharedSecretError::new(&format!("Failed to create EcKey from public key: {}", e))
    })?;

    let ephemeral_pkey: PKey<_> = ephemeral_ec_key.try_into().map_err(|e| {
        DeriveSharedSecretError::new(&format!("Failed to convert EcKey to PKey: {}", e))
    })?;

    let mut deriver = Deriver::new(&private_pkey)
        .map_err(|e| DeriveSharedSecretError::new(&format!("Failed to create deriver: {}", e)))?;

    deriver
        .set_peer(&ephemeral_pkey)
        .map_err(|e| DeriveSharedSecretError::new(&format!("Failed to set peer: {}", e)))?;

    let shared_secret = deriver
        .derive_to_vec()
        .map_err(|e| DeriveSharedSecretError::new(&format!("Failed to derive secret: {}", e)))?;

    Ok(shared_secret)
}

fn decrypt_eplv(eplv: Eplv, shared_secret: Vec<u8>) -> Result<Plv, DecryptError> {
    let key = GenericArray::from_slice(&shared_secret);
    let cipher = Aes256Gcm::new(key);

    let nonce = GenericArray::from_slice(&(eplv.nonce));
    let eplv_value = &eplv.enc_value[..];
    let plaintext = cipher
        .decrypt(nonce, aead::Payload::from(eplv_value))
        .inspect_err(|e| {
            log::error!("There was some error while decrypting: {:?}", e); // Log the error
        })?;

    // Attempt to convert the Vec<u8> into a [u8; 16]
    let value: [u8; 16] = match plaintext.try_into() {
        Ok(value) => value,
        Err(_) => {
            return Err(DecryptError::new(
                "The Vec does not have exactly 16 elements.",
            ));
        }
    };
    let i_index = eplv.i_index;
    let j_index = eplv.j_index;
    let plv = Plv::new(value, i_index, j_index);

    Ok(plv)
}

/// Convert VerifyingKey into [u8; 64]
pub fn convert_verifying_key_into_bytes_64(
    pub_key_encrypt: Option<VerifyingKey>,
    pub_key_sign: Option<VerifyingKey>,
) -> Result<([u8; 64], [u8; 64]), GenCertificateError> {
    // Define a conversion function from VerifyingKey to [u8; 64]
    fn convert_to_bytes_64(key: &VerifyingKey) -> [u8; 64] {
        // Convert the VerifyingKey to an uncompressed form
        let encoded_point = key.to_encoded_point(false);

        // Get the bytes from the uncompressed form
        let uncompressed_bytes = encoded_point.as_bytes();

        // Extract the X and Y coordinates (each 32 bytes) from the uncompressed bytes
        // Skip the first byte which is the identifier (0x04 for uncompressed points)
        let x_bytes = &uncompressed_bytes[1..33];
        let y_bytes = &uncompressed_bytes[33..65];

        // Create an array to hold the 64 bytes (32 for X, 32 for Y)
        let mut result = [0u8; 64];

        // Copy X and Y coordinates into the result array
        result[..32].copy_from_slice(x_bytes);
        result[32..].copy_from_slice(y_bytes);

        result
    }

    // Handle the case where pub_key_encrypt is None
    let encrypt_value = if let Some(encrypt_key) = pub_key_encrypt {
        convert_to_bytes_64(&encrypt_key)
    } else {
        // empty
        [0; 64]
    };

    // Determine the sign key value
    let sign_value = if let Some(sign_key) = pub_key_sign {
        convert_to_bytes_64(&sign_key)
    } else {
        // empty
        [0; 64]
    };

    // Return the results as a tuple
    Ok((encrypt_value, sign_value))
}
