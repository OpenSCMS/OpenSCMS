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

use crate::OscmsBridgeError;
use scmscommon::{Caterpillar, CaterpillarObk, CaterpillarUbk, NonButterflyRequest};
use scmscommon::{CertificateType, ExpansionType};

use byteorder::{ByteOrder, LittleEndian};
use p256::EncodedPoint;
use p256::ecdsa::VerifyingKey;

use crate::util::{
    hashed_binary_hex, initialize_empty_oscms_octet_buffer,
    initialize_oscms_new_octet_buffer_from_vec, oscms_octet_buffer_to_vec,
};
use crate::{EeRaCertRequestArgs, handle_ee_ra_cert_request};

use crate::{
    OscmsErrorCode_OSCMS_ERROR_FAILED_PARSING, OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
    OscmsErrorCode_OSCMS_ERROR_INVALID_VERIFY_KEY_INDICATOR,
};

fn exp_type_from_i32(
    value: crate::OscmsAdditionalParamsType,
) -> Result<ExpansionType, OscmsBridgeError> {
    match value {
        crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_ORIGINAL => {
            Ok(ExpansionType::Original)
        }
        crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_UNIFIED => {
            Ok(ExpansionType::Unified)
        }
        crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_COMPACT_UNIFIED => {
            Ok(ExpansionType::Compact)
        }
        crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_NONE => {
            Ok(ExpansionType::NonButterfly)
        }
        crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_ENCRYPTION_KEY => {
            Ok(ExpansionType::NonButterflyEncrypted)
        }
        _ => Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_FAILED_PARSING,
        )),
    }
}

fn certificate_type_from_u32(value: u32) -> Result<CertificateType, OscmsBridgeError> {
    match value {
        0 => Ok(CertificateType::Explicit),
        1 => Ok(CertificateType::Implicit),
        _ => Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_FAILED_PARSING,
        )),
    }
}

/// Extracts a verifying key from a byte array representing an encoded point on a specific curve.
///
/// # Arguments
/// * `encoded_point` - A byte array of length 33: first byte representing the compression and following 32 representing the encoded point
///
/// # Returns
///
/// A `Result` containing the extracted verifying key if successful, or a `CertRequestDecoderError` if an error occurs.
///
/// # Errors
///
/// Returns `Err` if:
/// * The encoded point fails to be imported.
/// * The verifying key fails to be generated from the encoded point.
///
fn extract_pub_key_from_bytes(encoded_point: Vec<u8>) -> Result<VerifyingKey, OscmsBridgeError> {
    // Constructing encoded point from full_encoded_point
    let encoded_point_from_bytes = match EncodedPoint::from_bytes(encoded_point.clone()) {
        Ok(key) => key,
        Err(_) => {
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INVALID_VERIFY_KEY_INDICATOR,
            ));
        }
    };

    // Finally, generating verifying key from encoded point
    match VerifyingKey::from_encoded_point(&encoded_point_from_bytes) {
        Ok(key) => Ok(key),
        Err(_) => Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INVALID_VERIFY_KEY_INDICATOR,
        )),
    }
}

/// Build a `Caterpillar` instance based on the encoded data and additional arguments.
///
/// This function constructs a `Caterpillar` instance using the provided encoded data and arguments.
///
/// # Arguments
///
/// * `encoded` - A vector of bytes Vec<u8> representing the encoded data.
/// * `args` - An instance of `EeRaCertRequestArgs` containing output data from OSCMS-Bridge C-layer after certificate decoding.
///
/// # Returns
///
/// A `Result` containing the constructed `Caterpillar` instance if successful, or a `CertRequestDecoderError` if an error occurs.
///
/// # Errors
///
/// Returns `Err` if:
/// * Failed to extract public key from bytes.
/// * Unsupported scmsType encountered.
///
/// encoded,
fn build_caterpillar(
    vid: u64,
    certificate_type: CertificateType,
    verifying_key_raw: Vec<u8>,
    encryption_key_raw: Vec<u8>,
    hash_id: String,
    args: EeRaCertRequestArgs,
) -> Result<Caterpillar, OscmsBridgeError> {
    // Extracting pub_key_sign
    let pub_key_sign = extract_pub_key_from_bytes(verifying_key_raw)?;

    // Match scmsType to build Caterpillar instance
    let exp_type = exp_type_from_i32(args.certificate_flavor_type)?;
    match exp_type {
        ExpansionType::Original => {
            // Extracting pub_key_encrypt for Obk type
            log::debug!("Building Caterpillar for Obk");

            let pub_key_encrypt = extract_pub_key_from_bytes(encryption_key_raw)?;

            let f_sign: [u8; 16] = oscms_octet_buffer_to_vec(&args.signing_expansion)
                .try_into()
                .map_err(|_| {
                    OscmsBridgeError::new(OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR)
                })?;

            let f_encrypt: [u8; 16] = oscms_octet_buffer_to_vec(&args.encryption_expansion)
                .try_into()
                .map_err(|_| {
                    OscmsBridgeError::new(OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR)
                })?;

            let caterpillar = CaterpillarObk::new(
                vid,
                pub_key_sign,
                pub_key_encrypt,
                f_sign,
                f_encrypt,
                ExpansionType::Original,
                hash_id,
                certificate_type,
            );
            Ok(scmscommon::Caterpillar::Obk(caterpillar))
        }
        ExpansionType::Unified | ExpansionType::Compact => {
            // Only when scmsType is Ubk or Cubk
            log::debug!("Building Caterpillar for {:#?}", exp_type);

            let f_sign: [u8; 16] = oscms_octet_buffer_to_vec(&args.signing_expansion)
                .try_into()
                .map_err(|_| {
                    OscmsBridgeError::new(OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR)
                })?;

            let caterpillar = CaterpillarUbk::new(
                vid,
                pub_key_sign,
                f_sign,
                hash_id,
                certificate_type,
                exp_type,
            );
            Ok(scmscommon::Caterpillar::Ubk(caterpillar))
        }
        _ => {
            log::error!(
                "Unsupported scmsType for butterfly process: {}",
                args.certificate_flavor_type
            );
            Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_FAILED_PARSING,
            ))
        }
    }
}

pub fn decode_certificate_request_spdu(
    encoded_ee_ra_cert_request: Vec<u8>,
    eca_public: [u8; 64],
    ra_private: Vec<u8>,
    ra_certificate: Vec<u8>,
    eca_certificate: Vec<u8>,
) -> Result<
    (
        Option<Caterpillar>,
        Option<NonButterflyRequest>,
        (u64, u32, u32),
    ),
    OscmsBridgeError,
> {
    log::debug!(
        "OSCMS-BRIDGE: Decoding Certificate Request {:?}",
        encoded_ee_ra_cert_request.len()
    );

    // Inputs
    let mut encoded_ee_ra_cert_request_spdu =
        initialize_oscms_new_octet_buffer_from_vec(encoded_ee_ra_cert_request.clone());
    let mut eca_public_key = initialize_oscms_new_octet_buffer_from_vec(eca_public.to_vec());
    let mut eca_encoded_certificate =
        initialize_oscms_new_octet_buffer_from_vec(eca_certificate.clone());
    let mut ra_private_key = initialize_oscms_new_octet_buffer_from_vec(ra_private.clone());
    let mut ra_encoded_certificate =
        initialize_oscms_new_octet_buffer_from_vec(ra_certificate.clone());

    if encoded_ee_ra_cert_request_spdu.data.is_null()
        || eca_public_key.data.is_null()
        || eca_encoded_certificate.data.is_null()
        || ra_private_key.data.is_null()
        || ra_encoded_certificate.data.is_null()
    {
        unsafe {
            crate::oscms_empty_octet_buffer(&mut encoded_ee_ra_cert_request_spdu);
            crate::oscms_empty_octet_buffer(&mut eca_public_key);
            crate::oscms_empty_octet_buffer(&mut eca_encoded_certificate);
            crate::oscms_empty_octet_buffer(&mut ra_private_key);
            crate::oscms_empty_octet_buffer(&mut ra_encoded_certificate);
        }
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }
    // outputs
    let device_id = initialize_empty_oscms_octet_buffer();
    let issuer_id = initialize_empty_oscms_octet_buffer();
    let caterpillar_public_key = initialize_empty_oscms_octet_buffer();
    let signing_expansion = initialize_empty_oscms_octet_buffer();
    let encryption_expansion = initialize_empty_oscms_octet_buffer();
    let encryption_key = initialize_empty_oscms_octet_buffer();

    // 1. Create a new instance of EeRaCertRequestSpduArgs
    let mut args = EeRaCertRequestArgs {
        encoded_ee_ra_cert_request_spdu,
        eca_public_key,
        eca_encoded_certificate,
        ra_private_key,
        ra_encoded_certificate,
        request_generation_time: 0,
        request_certificate_validity_start: 0,
        certificate_flavor_type: 0,
        certificate_type: 0,
        device_id,
        issuer_id,
        caterpillar_public_key,
        signing_expansion,
        encryption_expansion,
        encryption_key,
    };

    // 2. Call handle_ee_ra_cert_request and handle result
    let result = unsafe { handle_ee_ra_cert_request(&mut args) };
    if result != 0 {
        empty_args(&mut args);
        return Err(OscmsBridgeError::new(result));
    }

    log::debug!("Certificate request SPDU decoded successfully");

    // 4. Build Caterpillar instance based on AdditionalParams from certificate request
    // Or build NonButterfly request if no butterfly process was requested
    let exp_type = exp_type_from_i32(args.certificate_flavor_type)?;

    // Vehicle Id
    let device_id_buf = oscms_octet_buffer_to_vec(&args.device_id);
    let vid = LittleEndian::read_u64(device_id_buf.as_slice());
    log::debug!("Vehicle ID: {}", vid);

    // Certificate Type
    let certificate_type = certificate_type_from_u32(args.certificate_type)?;
    log::debug!("Certificate Type: {}", certificate_type);

    // Extracting verifying_key_raw
    let verifying_key_raw = oscms_octet_buffer_to_vec(&args.caterpillar_public_key);

    // Extracting verifying_key_raw
    let encryption_key_raw = oscms_octet_buffer_to_vec(&args.encryption_key);

    // Generating hash id based on input encoded data
    let hash_id = match hashed_binary_hex(encoded_ee_ra_cert_request) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("Failed to generate hash id: {:?}", e);
            empty_args(&mut args);
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }
    };

    let mut caterpillar = None;
    let mut non_butterfly = None;

    if exp_type == ExpansionType::NonButterfly || exp_type == ExpansionType::NonButterflyEncrypted {
        log::debug!("Building instance for NonButterfly process");

        let local_encryption_key = if exp_type == ExpansionType::NonButterflyEncrypted {
            Some(encryption_key_raw.clone())
        } else {
            None
        };

        non_butterfly = Some(NonButterflyRequest {
            vid,
            verifying_key_raw,
            encryption_key_raw: local_encryption_key,
            hash_id,
            exp_type,
            certificate_type,
        });
    } else {
        caterpillar = Some(build_caterpillar(
            vid,
            certificate_type,
            verifying_key_raw,
            encryption_key_raw,
            hash_id,
            args,
        )?);

        log::debug!(
            "Caterpillar instance built successfully: generation time {}",
            args.request_generation_time
        );
    }

    // Clean up buffers
    empty_args(&mut args);

    // 5. Return Caterpillar if everything went well
    // Return caterpillar when butterfly and non butterfly otherwise
    log::debug!("Payload decoded successfully");
    Ok((
        caterpillar,
        non_butterfly,
        (
            vid,
            args.request_generation_time,
            args.request_certificate_validity_start,
        ),
    ))
}

fn empty_args(args: &mut EeRaCertRequestArgs) {
    unsafe {
        // Inputs
        crate::oscms_empty_octet_buffer(&mut args.encoded_ee_ra_cert_request_spdu);
        crate::oscms_empty_octet_buffer(&mut args.eca_public_key);
        crate::oscms_empty_octet_buffer(&mut args.eca_encoded_certificate);
        crate::oscms_empty_octet_buffer(&mut args.ra_private_key);
        crate::oscms_empty_octet_buffer(&mut args.ra_encoded_certificate);
        // Outputs
        crate::oscms_empty_octet_buffer(&mut args.device_id);
        crate::oscms_empty_octet_buffer(&mut args.issuer_id);
        crate::oscms_empty_octet_buffer(&mut args.caterpillar_public_key);
        crate::oscms_empty_octet_buffer(&mut args.signing_expansion);
        crate::oscms_empty_octet_buffer(&mut args.encryption_expansion);
        crate::oscms_empty_octet_buffer(&mut args.encryption_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmscommon::CertificateType;

    #[test]
    fn test_butterfly_ee_ra_cert_request_spdu_dat() {
        let encoded_ee_ra_cert_request: Vec<u8> = vec![
            0x03, 0x82, 0x01, 0x01, 0x82, 0x41, 0xB9, 0xB9, 0x62, 0x31, 0x26, 0xC5, 0xBF, 0x80,
            0x83, 0x6C, 0xAF, 0xF9, 0x07, 0xEF, 0xE3, 0xB6, 0xD3, 0xB1, 0xE2, 0x80, 0x9E, 0x25,
            0x21, 0x9E, 0x30, 0xF6, 0xBE, 0x1C, 0xF7, 0xA7, 0x30, 0x39, 0x85, 0x14, 0xFB, 0x4E,
            0x4F, 0x6B, 0x47, 0xD8, 0x13, 0x78, 0x01, 0x72, 0x6C, 0x80, 0x81, 0x4A, 0x5F, 0x45,
            0xE8, 0x48, 0x9F, 0x9B, 0xD3, 0x8E, 0xC7, 0xBE, 0x51, 0x3E, 0x06, 0x52, 0xEB, 0xD2,
            0x2F, 0xD1, 0xCA, 0x1D, 0x0C, 0x5B, 0x9E, 0xF3, 0xA3, 0x80, 0x13, 0xCC, 0x74, 0xCB,
            0x94, 0xB1, 0xB3, 0x80, 0x3B, 0xBE, 0x0D, 0xF1, 0x82, 0x01, 0x6A, 0x98, 0x84, 0x28,
            0x7B, 0x48, 0xE0, 0xD9, 0x8B, 0xD8, 0x50, 0x79, 0x76, 0x56, 0x40, 0x32, 0xF1, 0xEC,
            0x56, 0xC3, 0xB7, 0x09, 0x52, 0xE8, 0xA2, 0x0B, 0x24, 0xF6, 0x9C, 0x77, 0xE7, 0x6C,
            0xA9, 0xB5, 0x88, 0xC7, 0x1C, 0xE0, 0xBF, 0xA7, 0xEE, 0x6E, 0x7B, 0xB2, 0x0C, 0x7A,
            0x14, 0xF2, 0x36, 0xF6, 0xF3, 0xDB, 0xD5, 0xB4, 0xE6, 0x36, 0x12, 0xBE, 0x68, 0x50,
            0xF1, 0x83, 0xF5, 0x9B, 0x0D, 0x64, 0x5C, 0xFA, 0x9A, 0x8B, 0xC4, 0x5E, 0x60, 0xD3,
            0x83, 0x08, 0xBD, 0x6A, 0x60, 0xF4, 0xA6, 0xC7, 0x95, 0xEA, 0xA6, 0x71, 0x08, 0xC5,
            0x8F, 0x16, 0x51, 0x61, 0x7F, 0x03, 0x8E, 0xE2, 0x87, 0xA8, 0xE7, 0x71, 0x0E, 0x8F,
            0x9B, 0x5B, 0x84, 0x04, 0x03, 0x58, 0x41, 0x3E, 0xC3, 0xB2, 0xDC, 0x01, 0x97, 0x32,
            0x40, 0xF1, 0xD3, 0xEF, 0xFB, 0xF7, 0x56, 0xDD, 0x8F, 0x32, 0xC1, 0x82, 0x8E, 0x1B,
            0x0E, 0x36, 0x56, 0xCA, 0xDD, 0x6F, 0x71, 0x3C, 0x63, 0xD3, 0x02, 0xBD, 0xEE, 0x90,
            0x97, 0x4B, 0xA8, 0x34, 0x8B, 0x41, 0x74, 0x63, 0x81, 0x66, 0xEF, 0x06, 0x6C, 0xCB,
            0xCC, 0x86, 0xAD, 0x58, 0xC4, 0x8A, 0x82, 0x9F, 0x21, 0xC0, 0x77, 0xAD, 0xFC, 0x15,
            0x19, 0x46, 0x62, 0x8A, 0x04, 0x7D, 0xA2, 0x99, 0x82, 0xD7, 0x7B, 0x33, 0x89, 0xFC,
            0x22, 0x8A, 0x3D, 0xF3, 0x6C, 0x73, 0xB7, 0xD1, 0x19, 0xF2, 0x1B, 0x16, 0xCE, 0x25,
            0xFF, 0x49, 0x9B, 0x29, 0xCE, 0x26, 0x11, 0xD3, 0xAE, 0x5F, 0x91, 0x44, 0x14, 0x94,
            0x2E, 0x71, 0x4C, 0xC8, 0xC4, 0x37, 0xB8, 0x7C, 0x3D, 0x51, 0x72, 0xAC, 0x40, 0xC2,
            0x6C, 0xD9, 0x7B, 0x4A, 0xA6, 0xDB, 0x94, 0xCB, 0x80, 0xF3, 0x39, 0xA4, 0xCA, 0x1B,
            0xA7, 0x94, 0x1E, 0xEC, 0x73, 0x42, 0x1F, 0xA1, 0xFD, 0xB2, 0xC5, 0xDB, 0x80, 0x1F,
            0x2B, 0x22, 0x35, 0x17, 0xD7, 0xDD, 0x8C, 0xBB, 0x99, 0x0C, 0x4D, 0x29, 0x80, 0x48,
            0x86, 0x90, 0x6C, 0x37, 0xEB, 0xB9, 0x57, 0x94, 0xD8, 0x83, 0x3C, 0xC1, 0x3F, 0x5D,
            0x2F, 0x83, 0x71, 0xF1, 0x81, 0xB4, 0x09, 0xB5, 0xF8, 0x3D, 0x74, 0xED, 0x37, 0x86,
            0x1B, 0xEF, 0xFA, 0x40, 0x16, 0x43, 0xAF, 0xBF, 0x7F, 0xC1, 0xA1, 0x15, 0xD7, 0xE9,
            0x67, 0x20, 0xCF, 0x68, 0x9C, 0x3B, 0x99, 0x9A, 0xB1, 0x98, 0xFC, 0x96, 0x67, 0x15,
            0x82, 0xBA, 0x0F, 0xAB, 0x4A, 0xDB, 0x51, 0xFD, 0x3E, 0xDD, 0x2F, 0x0E, 0xA2, 0x88,
            0x63, 0x3D, 0xF9, 0xBE, 0x7D, 0xC2, 0xAE, 0xF4, 0x88, 0x01, 0xCF, 0x0C, 0x41, 0xD2,
            0x4C, 0x63, 0x55, 0x1E, 0x10, 0xDD, 0x47, 0xC6, 0xC7,
        ];

        let eca_public: [u8; 64] = [
            0x8E, 0xD8, 0x59, 0x85, 0x54, 0x06, 0x4C, 0x2E, 0xD1, 0xB0, 0x9E, 0x61, 0xA1, 0xD1,
            0x62, 0x4F, 0x01, 0x03, 0x93, 0xFC, 0xF8, 0x48, 0xCB, 0x1C, 0x77, 0x86, 0x61, 0xB0,
            0xBF, 0x02, 0x17, 0xD5, 0x5A, 0xC2, 0x34, 0x62, 0x46, 0x2A, 0x97, 0x2F, 0x55, 0xC0,
            0x57, 0x35, 0x77, 0x17, 0xA9, 0x2F, 0x49, 0x2A, 0xF9, 0xD9, 0x43, 0xE1, 0x30, 0x68,
            0xD3, 0xB8, 0x2D, 0x7C, 0xDC, 0x7D, 0xCC, 0x04,
        ];

        let eca_certificate: Vec<u8> = vec![
            0x80, 0x03, 0x00, 0x80, 0xB8, 0x48, 0x55, 0x2B, 0x1C, 0xD8, 0x01, 0xC2, 0x10, 0x82,
            0x10, 0xFC, 0x5B, 0xE6, 0xA9, 0xD6, 0xFA, 0x07, 0x7D, 0x86, 0xA1, 0xDB, 0xD7, 0xC6,
            0x71, 0x26, 0xED, 0x01, 0x02, 0x03, 0x00, 0x00, 0x29, 0x10, 0x92, 0x2A, 0x86, 0x00,
            0x0A, 0x01, 0x01, 0x80, 0x01, 0x23, 0x80, 0x03, 0x84, 0x00, 0x02, 0x80, 0x80, 0x82,
            0x8E, 0xD8, 0x59, 0x85, 0x54, 0x06, 0x4C, 0x2E, 0xD1, 0xB0, 0x9E, 0x61, 0xA1, 0xD1,
            0x62, 0x4F, 0x01, 0x03, 0x93, 0xFC, 0xF8, 0x48, 0xCB, 0x1C, 0x77, 0x86, 0x61, 0xB0,
            0xBF, 0x02, 0x17, 0xD5, 0x80, 0x80, 0xD4, 0x67, 0x93, 0x87, 0xAE, 0xBD, 0x73, 0xDC,
            0xB4, 0x46, 0x38, 0x3F, 0x01, 0x71, 0x2C, 0x3D, 0x75, 0x56, 0xC8, 0x53, 0x42, 0x02,
            0xB3, 0x7F, 0x49, 0x50, 0x15, 0xF9, 0x10, 0xC6, 0xA9, 0x7D, 0xC7, 0xA7, 0xFE, 0x7E,
            0x1A, 0x36, 0xA3, 0xB4, 0x74, 0x2A, 0x30, 0xF9, 0x0C, 0x5E, 0x38, 0xBF, 0x41, 0x23,
            0x9A, 0xA8, 0x62, 0x8E, 0x06, 0xDD, 0xF6, 0xB2, 0xFA, 0xB3, 0xF4, 0xF4, 0x18, 0x80,
        ];

        let ra_certificate: Vec<u8> = vec![
            0x80, 0x03, 0x00, 0x80, 0xB8, 0x48, 0x55, 0x2B, 0x1C, 0xD8, 0x01, 0xC2, 0x11, 0x82,
            0x10, 0x8F, 0xEE, 0x81, 0xD8, 0xDB, 0xA9, 0x4A, 0x0E, 0xFA, 0x0C, 0xB5, 0x25, 0x40,
            0x00, 0x10, 0xF4, 0x01, 0x02, 0x03, 0x00, 0x00, 0x29, 0x10, 0x92, 0x2A, 0x86, 0x00,
            0x0A, 0x01, 0x01, 0x80, 0x01, 0x23, 0x80, 0x03, 0x8B, 0x00, 0x02, 0x00, 0x80, 0x83,
            0x72, 0x6A, 0x60, 0x52, 0xDB, 0x3F, 0xA4, 0xAC, 0x07, 0xB9, 0x8F, 0x30, 0x70, 0x55,
            0x29, 0xE8, 0x76, 0x33, 0x08, 0x60, 0x2B, 0x46, 0x90, 0x45, 0x05, 0x54, 0x08, 0x73,
            0x01, 0xA7, 0xBA, 0x05, 0x80, 0x80, 0x82, 0x15, 0x3D, 0xE8, 0x27, 0x46, 0x11, 0x9F,
            0x53, 0x74, 0x3E, 0x82, 0xF6, 0x81, 0xF2, 0x5C, 0x0B, 0x42, 0x99, 0x5D, 0x25, 0x64,
            0x9A, 0x54, 0x87, 0x3C, 0xB9, 0x87, 0x5B, 0xB2, 0x18, 0xF3, 0xD8, 0x80, 0x80, 0xE2,
            0x1E, 0xAB, 0x1C, 0x87, 0x1F, 0x8C, 0x81, 0xD5, 0x73, 0xD7, 0xD6, 0x5A, 0x50, 0xD3,
            0xC6, 0x54, 0x5E, 0xA6, 0x9B, 0xAD, 0xF7, 0x19, 0xBC, 0x1B, 0x12, 0xD1, 0x87, 0x63,
            0x2A, 0x79, 0xBC, 0xEA, 0x0C, 0xCC, 0x47, 0x2D, 0xBC, 0xB0, 0x34, 0xFD, 0xFB, 0x59,
            0x87, 0x80, 0xC1, 0xF7, 0x74, 0xD7, 0xFD, 0x02, 0x67, 0xE4, 0x16, 0xDC, 0xF0, 0xB8,
            0x1E, 0x9F, 0xC7, 0xF0, 0x8D, 0x4C, 0x8C,
        ];

        let ra_private: Vec<u8> = vec![
            0x43, 0x0E, 0xC7, 0xCA, 0x56, 0xC6, 0x3B, 0x31, 0x98, 0xA3, 0x72, 0xE6, 0x95, 0xF4,
            0x79, 0xF7, 0xCB, 0x7C, 0x14, 0xC4, 0xE4, 0xDF, 0x6C, 0xBA, 0x07, 0x4D, 0x86, 0xAF,
            0x34, 0xED, 0x3C, 0x7A,
        ];

        match decode_certificate_request_spdu(
            encoded_ee_ra_cert_request,
            eca_public.clone(),
            ra_private.clone(),
            ra_certificate.clone(),
            eca_certificate.clone(),
        ) {
            Ok((c, n, _)) => {
                assert!(c.is_some());
                assert!(n.is_none());

                let expected_out = Caterpillar::Obk(CaterpillarObk::new(
                    8766147116415731183,
                    VerifyingKey::from_sec1_bytes(&[
                        4, 144, 153, 199, 166, 244, 197, 108, 82, 219, 252, 32, 247, 241, 235, 94,
                        250, 67, 0, 106, 211, 139, 58, 108, 225, 71, 127, 145, 189, 240, 175, 35,
                        229, 95, 26, 222, 96, 49, 215, 219, 118, 23, 34, 183, 121, 192, 230, 26,
                        34, 246, 100, 109, 91, 87, 60, 93, 78, 93, 191, 29, 125, 24, 70, 210, 93,
                    ])
                    .unwrap(),
                    VerifyingKey::from_sec1_bytes(&[
                        4, 169, 43, 33, 27, 123, 31, 167, 25, 197, 164, 131, 92, 86, 45, 83, 66,
                        138, 104, 36, 246, 33, 179, 98, 60, 218, 103, 192, 125, 44, 21, 76, 120,
                        137, 201, 224, 200, 174, 104, 183, 205, 115, 255, 76, 116, 57, 49, 89, 171,
                        180, 144, 118, 222, 88, 42, 8, 251, 221, 251, 172, 66, 134, 113, 44, 211,
                    ])
                    .unwrap(),
                    [
                        226, 125, 107, 133, 211, 199, 64, 111, 135, 113, 82, 244, 245, 230, 20, 207,
                    ],
                    [
                        13, 16, 179, 189, 53, 35, 67, 178, 62, 68, 21, 185, 206, 75, 67, 228,
                    ],
                    ExpansionType::Original,
                    "cfcf7aecd37bd940".to_string(),
                    CertificateType::Implicit,
                ));
                assert!(c.unwrap().eq(&expected_out));
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
    }

    #[test]
    fn test_nonbutterfly_ee_ra_cert_request_spdu_dat() {
        let encoded_ee_ra_cert_request_nb_plain: Vec<u8> = vec![
            3, 130, 1, 1, 130, 65, 185, 185, 98, 49, 38, 197, 191, 128, 131, 112, 196, 196, 88,
            129, 240, 64, 18, 220, 204, 47, 177, 25, 218, 68, 157, 41, 182, 235, 201, 109, 235,
            145, 40, 53, 44, 93, 58, 230, 117, 76, 37, 60, 116, 19, 46, 233, 127, 203, 166, 218,
            168, 2, 205, 172, 79, 211, 76, 208, 45, 239, 242, 212, 130, 212, 195, 85, 150, 223,
            117, 118, 218, 251, 89, 128, 122, 24, 181, 232, 217, 237, 204, 228, 195, 251, 154, 30,
            130, 1, 112, 180, 116, 241, 107, 229, 110, 68, 30, 164, 206, 7, 223, 107, 114, 10, 4,
            207, 28, 87, 64, 52, 189, 51, 133, 227, 172, 103, 147, 205, 177, 170, 226, 44, 46, 252,
            142, 212, 114, 168, 131, 23, 172, 254, 143, 202, 149, 131, 120, 192, 66, 163, 28, 67,
            115, 67, 119, 253, 53, 160, 188, 54, 240, 110, 72, 97, 16, 182, 251, 106, 166, 2, 25,
            58, 24, 75, 29, 50, 148, 13, 252, 178, 146, 1, 21, 235, 152, 117, 50, 50, 133, 48, 44,
            239, 73, 255, 96, 153, 98, 151, 24, 251, 190, 143, 235, 125, 191, 34, 252, 5, 197, 46,
            73, 6, 124, 242, 86, 168, 121, 157, 221, 117, 71, 112, 224, 199, 26, 92, 254, 243, 140,
            23, 0, 206, 38, 173, 180, 183, 126, 45, 45, 135, 144, 150, 161, 52, 125, 27, 18, 5,
            157, 78, 133, 134, 175, 26, 180, 84, 104, 195, 200, 33, 83, 113, 235, 52, 119, 247, 82,
            82, 132, 33, 240, 160, 1, 254, 95, 145, 245, 192, 116, 175, 151, 191, 237, 203, 174,
            220, 72, 54, 158, 70, 118, 111, 43, 133, 28, 242, 93, 183, 164, 217, 209, 248, 130,
            217, 70, 121, 140, 137, 19, 250, 219, 158, 183, 152, 140, 155, 197, 104, 32, 189, 107,
            116, 161, 240, 115, 39, 12, 220, 242, 186, 37, 117, 40, 183, 171, 234, 204, 18, 207,
            52, 20, 231, 113, 14, 63, 90, 29, 100, 77, 98, 37, 26, 78, 210, 36, 108, 247, 186, 101,
            152, 131, 158, 253, 198, 44, 210, 169, 121, 96, 207, 41, 104, 91, 22, 76, 185, 70, 49,
            100, 82, 28, 8, 87, 232, 219, 199, 198, 221, 218, 74, 196, 55, 221, 92, 2, 186, 103,
            77, 197, 89, 92, 198, 241, 70, 75, 163, 32, 144, 82, 204, 242, 170, 79, 145, 66, 4,
            106, 192, 166, 44, 162, 224, 74, 4, 71, 224, 201, 31, 78, 188, 118, 156, 224, 10, 214,
            182, 28, 144, 164, 35, 181, 249, 150, 239, 27, 124, 78, 227, 161, 241, 179, 121, 118,
            199, 17, 199, 103, 3, 253, 32, 6, 27, 47, 96, 205, 170, 133,
        ];

        let ee_ra_cert_request_nb_enc: Vec<u8> = vec![
            3, 130, 1, 1, 130, 65, 185, 185, 98, 49, 38, 197, 191, 128, 130, 68, 110, 249, 22, 101,
            25, 98, 186, 130, 128, 56, 215, 105, 20, 125, 79, 55, 66, 235, 192, 133, 25, 181, 138,
            141, 166, 68, 102, 56, 138, 130, 126, 224, 19, 36, 103, 192, 18, 112, 197, 207, 86, 61,
            42, 12, 128, 14, 236, 137, 205, 47, 110, 222, 203, 153, 207, 137, 31, 202, 82, 72, 159,
            180, 241, 128, 192, 188, 29, 222, 30, 241, 178, 76, 106, 36, 9, 177, 130, 1, 112, 255,
            238, 111, 105, 109, 199, 190, 100, 193, 172, 177, 210, 86, 204, 170, 84, 81, 135, 35,
            137, 248, 121, 254, 164, 135, 17, 155, 168, 142, 95, 103, 145, 167, 86, 179, 2, 121,
            107, 60, 188, 255, 97, 32, 37, 109, 133, 59, 222, 153, 112, 137, 13, 85, 38, 70, 199,
            90, 173, 193, 220, 165, 172, 20, 188, 223, 29, 241, 189, 127, 74, 109, 181, 109, 223,
            98, 172, 2, 104, 1, 115, 46, 76, 107, 158, 125, 79, 75, 233, 204, 178, 45, 12, 254,
            190, 135, 80, 143, 69, 105, 106, 220, 103, 204, 195, 31, 85, 37, 201, 193, 89, 227,
            246, 177, 206, 135, 87, 50, 19, 112, 72, 155, 86, 174, 65, 203, 93, 228, 16, 191, 83,
            24, 118, 41, 208, 18, 66, 58, 93, 45, 167, 2, 129, 71, 163, 171, 114, 91, 56, 204, 250,
            19, 79, 44, 42, 121, 169, 219, 88, 119, 106, 197, 47, 100, 88, 250, 73, 103, 145, 195,
            244, 191, 14, 166, 115, 174, 170, 178, 4, 43, 230, 118, 42, 221, 161, 145, 183, 40,
            107, 235, 229, 176, 43, 16, 80, 178, 217, 197, 208, 201, 135, 40, 224, 124, 80, 135,
            84, 118, 199, 142, 130, 146, 112, 198, 67, 65, 37, 73, 148, 144, 208, 76, 116, 37, 98,
            92, 4, 185, 224, 55, 13, 156, 113, 232, 66, 42, 198, 163, 145, 79, 153, 186, 181, 47,
            61, 10, 93, 134, 226, 190, 76, 136, 133, 16, 47, 194, 230, 161, 161, 75, 209, 161, 78,
            82, 220, 215, 228, 102, 243, 249, 21, 241, 253, 209, 218, 225, 63, 76, 22, 11, 188,
            122, 212, 55, 133, 122, 103, 9, 180, 222, 253, 76, 109, 42, 223, 69, 130, 115, 117, 66,
            80, 14, 33, 217, 87, 125, 120, 255, 219, 32, 0, 40, 22, 50, 137, 160, 254, 8, 6, 175,
            67, 224, 67, 181, 191, 42, 66, 182, 30, 138, 193, 234, 228, 109, 57, 61, 253, 96, 96,
            155, 247, 195, 131, 235, 94, 167, 172, 254, 200, 82, 112, 42, 81, 118, 39, 71, 85, 222,
            60, 48, 13, 226, 238, 160, 248, 227, 149, 238, 199,
        ];

        let eca_public: [u8; 64] = [
            0x8E, 0xD8, 0x59, 0x85, 0x54, 0x06, 0x4C, 0x2E, 0xD1, 0xB0, 0x9E, 0x61, 0xA1, 0xD1,
            0x62, 0x4F, 0x01, 0x03, 0x93, 0xFC, 0xF8, 0x48, 0xCB, 0x1C, 0x77, 0x86, 0x61, 0xB0,
            0xBF, 0x02, 0x17, 0xD5, 0x5A, 0xC2, 0x34, 0x62, 0x46, 0x2A, 0x97, 0x2F, 0x55, 0xC0,
            0x57, 0x35, 0x77, 0x17, 0xA9, 0x2F, 0x49, 0x2A, 0xF9, 0xD9, 0x43, 0xE1, 0x30, 0x68,
            0xD3, 0xB8, 0x2D, 0x7C, 0xDC, 0x7D, 0xCC, 0x04,
        ];

        let eca_certificate: Vec<u8> = vec![
            0x80, 0x03, 0x00, 0x80, 0xB8, 0x48, 0x55, 0x2B, 0x1C, 0xD8, 0x01, 0xC2, 0x10, 0x82,
            0x10, 0xFC, 0x5B, 0xE6, 0xA9, 0xD6, 0xFA, 0x07, 0x7D, 0x86, 0xA1, 0xDB, 0xD7, 0xC6,
            0x71, 0x26, 0xED, 0x01, 0x02, 0x03, 0x00, 0x00, 0x29, 0x10, 0x92, 0x2A, 0x86, 0x00,
            0x0A, 0x01, 0x01, 0x80, 0x01, 0x23, 0x80, 0x03, 0x84, 0x00, 0x02, 0x80, 0x80, 0x82,
            0x8E, 0xD8, 0x59, 0x85, 0x54, 0x06, 0x4C, 0x2E, 0xD1, 0xB0, 0x9E, 0x61, 0xA1, 0xD1,
            0x62, 0x4F, 0x01, 0x03, 0x93, 0xFC, 0xF8, 0x48, 0xCB, 0x1C, 0x77, 0x86, 0x61, 0xB0,
            0xBF, 0x02, 0x17, 0xD5, 0x80, 0x80, 0xD4, 0x67, 0x93, 0x87, 0xAE, 0xBD, 0x73, 0xDC,
            0xB4, 0x46, 0x38, 0x3F, 0x01, 0x71, 0x2C, 0x3D, 0x75, 0x56, 0xC8, 0x53, 0x42, 0x02,
            0xB3, 0x7F, 0x49, 0x50, 0x15, 0xF9, 0x10, 0xC6, 0xA9, 0x7D, 0xC7, 0xA7, 0xFE, 0x7E,
            0x1A, 0x36, 0xA3, 0xB4, 0x74, 0x2A, 0x30, 0xF9, 0x0C, 0x5E, 0x38, 0xBF, 0x41, 0x23,
            0x9A, 0xA8, 0x62, 0x8E, 0x06, 0xDD, 0xF6, 0xB2, 0xFA, 0xB3, 0xF4, 0xF4, 0x18, 0x80,
        ];

        let ra_certificate: Vec<u8> = vec![
            0x80, 0x03, 0x00, 0x80, 0xB8, 0x48, 0x55, 0x2B, 0x1C, 0xD8, 0x01, 0xC2, 0x11, 0x82,
            0x10, 0x8F, 0xEE, 0x81, 0xD8, 0xDB, 0xA9, 0x4A, 0x0E, 0xFA, 0x0C, 0xB5, 0x25, 0x40,
            0x00, 0x10, 0xF4, 0x01, 0x02, 0x03, 0x00, 0x00, 0x29, 0x10, 0x92, 0x2A, 0x86, 0x00,
            0x0A, 0x01, 0x01, 0x80, 0x01, 0x23, 0x80, 0x03, 0x8B, 0x00, 0x02, 0x00, 0x80, 0x83,
            0x72, 0x6A, 0x60, 0x52, 0xDB, 0x3F, 0xA4, 0xAC, 0x07, 0xB9, 0x8F, 0x30, 0x70, 0x55,
            0x29, 0xE8, 0x76, 0x33, 0x08, 0x60, 0x2B, 0x46, 0x90, 0x45, 0x05, 0x54, 0x08, 0x73,
            0x01, 0xA7, 0xBA, 0x05, 0x80, 0x80, 0x82, 0x15, 0x3D, 0xE8, 0x27, 0x46, 0x11, 0x9F,
            0x53, 0x74, 0x3E, 0x82, 0xF6, 0x81, 0xF2, 0x5C, 0x0B, 0x42, 0x99, 0x5D, 0x25, 0x64,
            0x9A, 0x54, 0x87, 0x3C, 0xB9, 0x87, 0x5B, 0xB2, 0x18, 0xF3, 0xD8, 0x80, 0x80, 0xE2,
            0x1E, 0xAB, 0x1C, 0x87, 0x1F, 0x8C, 0x81, 0xD5, 0x73, 0xD7, 0xD6, 0x5A, 0x50, 0xD3,
            0xC6, 0x54, 0x5E, 0xA6, 0x9B, 0xAD, 0xF7, 0x19, 0xBC, 0x1B, 0x12, 0xD1, 0x87, 0x63,
            0x2A, 0x79, 0xBC, 0xEA, 0x0C, 0xCC, 0x47, 0x2D, 0xBC, 0xB0, 0x34, 0xFD, 0xFB, 0x59,
            0x87, 0x80, 0xC1, 0xF7, 0x74, 0xD7, 0xFD, 0x02, 0x67, 0xE4, 0x16, 0xDC, 0xF0, 0xB8,
            0x1E, 0x9F, 0xC7, 0xF0, 0x8D, 0x4C, 0x8C,
        ];

        let ra_private: Vec<u8> = vec![
            0x43, 0x0E, 0xC7, 0xCA, 0x56, 0xC6, 0x3B, 0x31, 0x98, 0xA3, 0x72, 0xE6, 0x95, 0xF4,
            0x79, 0xF7, 0xCB, 0x7C, 0x14, 0xC4, 0xE4, 0xDF, 0x6C, 0xBA, 0x07, 0x4D, 0x86, 0xAF,
            0x34, 0xED, 0x3C, 0x7A,
        ];
        match decode_certificate_request_spdu(
            encoded_ee_ra_cert_request_nb_plain,
            eca_public.clone(),
            ra_private.clone(),
            ra_certificate.clone(),
            eca_certificate.clone(),
        ) {
            Ok((c, n, _)) => {
                assert!(c.is_none());
                assert!(n.is_some());

                let expected_out = NonButterflyRequest {
                    vid: 7046473490387245440,
                    verifying_key_raw: vec![
                        2, 172, 8, 76, 189, 219, 157, 237, 19, 168, 175, 2, 242, 144, 121, 155,
                        181, 28, 202, 77, 129, 220, 68, 107, 200, 85, 64, 229, 130, 212, 128, 125,
                        44,
                    ],
                    encryption_key_raw: Some(vec![
                        3, 117, 72, 221, 93, 183, 224, 48, 127, 58, 216, 29, 239, 185, 77, 176, 2,
                        182, 135, 90, 72, 222, 6, 35, 133, 82, 145, 176, 45, 50, 67, 53, 97,
                    ]),
                    hash_id: "66ef9cfa6709e04f".to_string(),
                    exp_type: ExpansionType::NonButterflyEncrypted,
                    certificate_type: CertificateType::Implicit,
                };
                assert!(n.unwrap().eq(&expected_out));
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };

        match decode_certificate_request_spdu(
            ee_ra_cert_request_nb_enc,
            eca_public,
            ra_private,
            ra_certificate,
            eca_certificate,
        ) {
            Ok((c, n, _)) => {
                assert!(c.is_none());
                assert!(n.is_some());

                let expected_out = NonButterflyRequest {
                    vid: 11439867982639064222,
                    verifying_key_raw: vec![
                        3, 126, 50, 67, 100, 152, 29, 182, 85, 177, 234, 141, 95, 141, 6, 149, 192,
                        147, 102, 149, 80, 243, 191, 26, 179, 169, 12, 71, 21, 245, 70, 209, 233,
                    ],
                    encryption_key_raw: Some(vec![
                        2, 222, 21, 92, 128, 148, 91, 126, 238, 152, 111, 130, 83, 5, 164, 50, 204,
                        111, 12, 138, 240, 128, 191, 18, 47, 62, 12, 99, 60, 227, 30, 228, 45,
                    ]),
                    hash_id: "cc8c0478e69d4c4c".to_string(),
                    exp_type: ExpansionType::NonButterflyEncrypted,
                    certificate_type: CertificateType::Implicit,
                };
                assert!(n.unwrap().eq(&expected_out));
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }
    }
}
