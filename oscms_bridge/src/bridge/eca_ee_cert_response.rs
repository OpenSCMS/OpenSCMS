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
use crate::OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR;
use crate::oscms_empty_octet_buffer;
use crate::{
    EeEcaCertRequestArgs, EeEcaCertRequestCheckParams, OscmsOctetBuffer, handle_ee_eca_cert_request,
};

use crate::util::{
    initialize_empty_oscms_octet_buffer, initialize_oscms_new_octet_buffer_from_vec,
    oscms_octet_buffer_to_vec,
};

use byteorder::{ByteOrder, LittleEndian};

/// Handles the enrollment certificate request using the ECA private and public keys.
///
/// This function interacts with a low-level handler (`ecaCertificateRequestHandler`) to process
/// the enrollment certificate request. It prepares the necessary arguments, calls the handler,
/// and processes the result.
///
/// # Arguments
///
/// * `enrollment_certificate_request` - A vector of bytes containing the enrollment certificate request message.
/// * `ecaPrivate` - A vector of bytes containing the ECA private key.
/// * `ecaPublicUncompressed` - A vector of bytes containing the uncompressed ECA public key.
/// * `debug` - A boolean flag indicating whether to enable debug mode.
///
/// # Returns
///
/// * `Ok(Vec<u8>)` - A vector of bytes containing the certificate response if the operation is successful.
/// * `Err(OscmsBridgeError)` - An error of type `OscmsBridgeError` if the operation fails.
///
/// # Errors
///
/// This function will return an error if the underlying `ecaCertificateRequestHandler` call fails or if the response is empty.
///
pub fn eca_enrollment_certificate_request_handler(
    enrollment_certificate_request: Vec<u8>,
    ecaPrivate: Vec<u8>,
    canonical_public_key: Vec<u8>,
    eca_certificate_chain: Vec<Vec<u8>>,
    ee_registration_vid: u64,
    time_request_received: u32,
    eca_max_wait: u32,
    eca_max_age: u32,
) -> Result<(Vec<u8>, u64, String), OscmsBridgeError> {
    log::debug!(
        "[OSCMS-BRIDGE] Handling ECA enrollment certificate request lib1609: request size {:?}",
        enrollment_certificate_request.len()
    );

    // Vehicle id from 64 to byte array
    let mut vid_array = [0u8; 64];
    LittleEndian::write_u64(&mut vid_array, ee_registration_vid);

    // Inputs
    let encoded_ee_eca_cert_request_spdu =
        initialize_oscms_new_octet_buffer_from_vec(enrollment_certificate_request);
    let device_public_key = initialize_oscms_new_octet_buffer_from_vec(canonical_public_key);
    let issuer_private_key = initialize_oscms_new_octet_buffer_from_vec(ecaPrivate.clone());
    let issuer_encoded_certificate =
        initialize_oscms_new_octet_buffer_from_vec(eca_certificate_chain[0].clone());
    let issuer_encoded_certificates_chain: Vec<OscmsOctetBuffer> = eca_certificate_chain
        .iter()
        .map(|v| initialize_oscms_new_octet_buffer_from_vec(v.clone()))
        .collect();
    let device_id = initialize_oscms_new_octet_buffer_from_vec(vid_array.to_vec());

    // Outputs
    let output_spdu = initialize_empty_oscms_octet_buffer();
    let issuer_id = initialize_empty_oscms_octet_buffer();

    // Craca ID will be filled using random value
    let craca_id = [0u8; 3];

    let mut args = EeEcaCertRequestArgs {
        encoded_ee_eca_cert_request_spdu: encoded_ee_eca_cert_request_spdu,
        device_public_key: device_public_key,
        issuer_private_key: issuer_private_key,
        issuer_encoded_certificate: issuer_encoded_certificate,
        issuer_encoded_certificates_chain: issuer_encoded_certificates_chain.as_ptr(),
        issuer_encoded_certificates_chain_count: issuer_encoded_certificates_chain.len(),
        device_id: device_id,
        craca_id,
        crl_series: 0,
        issuer_id: issuer_id,
        output_spdu: output_spdu,
        is_successor_enrollment: false,
    };

    let mut params = EeEcaCertRequestCheckParams {
        time_request_received,
        eca_max_wait,
        eca_max_age,
    };

    let result = unsafe { handle_ee_eca_cert_request(&mut args, &mut params) };
    if result != 0 {
        log::error!(
            "OSCMS-BRIDGE:  Error encoding Certificate Ack SPDU: {}",
            result
        );
        return Err(OscmsBridgeError::new(result));
    }

    // 3. Copy the output buffer from args to the output data (encoded).
    //    In this case, the encoded buffer is a Vec<u8> so we need to
    //    iterate over the output buffer and copy the data to the encoded buffer
    if args.output_spdu.data.is_null() {
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }

    let certificate_response = oscms_octet_buffer_to_vec(&args.output_spdu);
    if certificate_response.is_empty() {
        log::error!("OSCMS-BRIDGE:  Empty Certificate Response SPDU");
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }

    let issuer_id_bytes = oscms_octet_buffer_to_vec(&args.issuer_id);
    if issuer_id_bytes.is_empty() {
        log::error!("OSCMS-BRIDGE:  Empty Issuer ID");
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }

    let issuer_id_hex: String = issuer_id_bytes
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    unsafe {
        // Inputs
        oscms_empty_octet_buffer(&mut args.encoded_ee_eca_cert_request_spdu);
        oscms_empty_octet_buffer(&mut args.device_public_key);
        oscms_empty_octet_buffer(&mut args.issuer_private_key);
        oscms_empty_octet_buffer(&mut args.issuer_encoded_certificate);
        oscms_empty_octet_buffer(&mut args.device_id);
        for mut buffer in issuer_encoded_certificates_chain {
            crate::oscms_empty_octet_buffer(&mut buffer);
        }
        // Outputs
        oscms_empty_octet_buffer(&mut args.output_spdu);
        oscms_empty_octet_buffer(&mut args.issuer_id);
    }

    log::debug!(
        "Vid: {:?}, IssuerId {:?}",
        ee_registration_vid,
        issuer_id_hex
    );

    Ok((certificate_response, ee_registration_vid, issuer_id_hex))
}

#[cfg(test)]
mod tests {
    use super::*;

    static C_ECA_PRIVATE: [u8; 32] = [
        0xD0, 0x23, 0xD4, 0x9F, 0x14, 0x52, 0x75, 0x11, 0x09, 0x95, 0xF8, 0x8D, 0xDA, 0x3A, 0x51,
        0x2F, 0xFE, 0x3B, 0xE0, 0x5C, 0x56, 0x53, 0xAD, 0x99, 0x68, 0x1B, 0x38, 0x5B, 0x38, 0xE5,
        0x4D, 0x34,
    ];

    fn eca_cert_chain_test() -> Vec<Vec<u8>> {
        vec![vec![
            0x80, 0x03, 0x00, 0x80, 0xA5, 0xE9, 0x44, 0x21, 0x4B, 0xE5, 0xB9, 0xE6, 0x00, 0x82,
            0x10, 0xB9, 0x54, 0x92, 0xD2, 0x16, 0xED, 0x9D, 0x18, 0x3B, 0x55, 0x10, 0xAB, 0x6D,
            0x3F, 0x77, 0x0B, 0x01, 0x02, 0x03, 0x00, 0x00, 0x27, 0x27, 0x38, 0x80, 0x86, 0x00,
            0x0A, 0x80, 0x80, 0x83, 0x65, 0x1F, 0xB1, 0x57, 0x20, 0xAE, 0x59, 0xAD, 0x41, 0x5D,
            0x69, 0x1D, 0x21, 0x57, 0x43, 0x7D, 0x38, 0xC0, 0xEA, 0xB9, 0x7F, 0x90, 0xAE, 0x3F,
            0x1C, 0xC1, 0xF2, 0x41, 0x24, 0x8D, 0xB0, 0xF2, 0x80, 0x80, 0xD3, 0x36, 0x53, 0x7E,
            0xD6, 0x8F, 0xAE, 0xC7, 0x11, 0x82, 0x34, 0x37, 0x70, 0xF2, 0x00, 0x3F, 0xBC, 0x7E,
            0xBA, 0x6F, 0x21, 0x98, 0xCA, 0x3A, 0xDE, 0xF1, 0x9B, 0x82, 0x78, 0x02, 0xE2, 0xF6,
            0x41, 0xAB, 0x23, 0x04, 0x73, 0xC1, 0xD9, 0xB7, 0xC0, 0x2B, 0x60, 0x5C, 0x87, 0x2A,
            0xF2, 0xD3, 0xE5, 0x1E, 0xFD, 0x3D, 0xBD, 0xCB, 0x99, 0x3E, 0xAB, 0x30, 0x90, 0xD1,
            0xE6, 0xEC, 0xDC, 0x6B,
        ]]
    }

    #[test]
    fn test_eca_enrollment_certificate_request_handler() {
        let enrollment_certificate_request = vec![
            0x03, 0x83, 0x81, 0xc9, 0x00, 0x02, 0x85, 0x80, 0x40, 0x02, 0x27, 0x2c, 0x31, 0xf0,
            0x01, 0x04, 0x83, 0x00, 0x00, 0x00, 0x00, 0x00, 0x27, 0x2c, 0x31, 0xf0, 0x86, 0x00,
            0x06, 0x01, 0x01, 0xe0, 0x81, 0x01, 0x00, 0x01, 0x01, 0x01, 0x80, 0x80, 0x82, 0xe3,
            0x96, 0x7b, 0xfd, 0x0c, 0x19, 0x36, 0x8d, 0xa2, 0xbc, 0x2b, 0x94, 0xff, 0x0f, 0xfb,
            0x18, 0x9a, 0xba, 0x4e, 0xb4, 0xce, 0x4f, 0xf8, 0x0f, 0x9f, 0x2d, 0xbe, 0xef, 0x2f,
            0xad, 0x46, 0x9e, 0x40, 0x31, 0x31, 0x44, 0x30, 0x39, 0x46, 0x41, 0x35, 0x39, 0x33,
            0x33, 0x42, 0x46, 0x45, 0x36, 0x31, 0x45, 0x43, 0x34, 0x41, 0x32, 0x36, 0x41, 0x30,
            0x36, 0x45, 0x30, 0x32, 0x34, 0x32, 0x30, 0x33, 0x41, 0x36, 0x36, 0x39, 0x39, 0x41,
            0x35, 0x44, 0x34, 0x45, 0x33, 0x39, 0x37, 0x33, 0x39, 0x43, 0x43, 0x44, 0x43, 0x35,
            0x46, 0x43, 0x42, 0x42, 0x32, 0x33, 0x37, 0x35, 0x39, 0x43, 0x36, 0x36, 0x82, 0x80,
            0x80, 0xae, 0x72, 0x2c, 0xcc, 0x28, 0x8c, 0xa1, 0x87, 0x13, 0x3e, 0x3d, 0x8b, 0xad,
            0x9c, 0x59, 0x6e, 0xf5, 0xa7, 0x96, 0x1f, 0xd8, 0xcb, 0x07, 0x6d, 0x52, 0xab, 0x7c,
            0x0d, 0xf1, 0x61, 0x16, 0x7e, 0xbe, 0xbb, 0x31, 0x1e, 0x40, 0x55, 0x48, 0xfa, 0xc5,
            0x1a, 0xcb, 0x0d, 0xce, 0xee, 0xd0, 0xe8, 0x3b, 0xa6, 0x50, 0x90, 0x49, 0x5c, 0x8d,
            0xf6, 0x0d, 0xc7, 0xc6, 0x45, 0x3e, 0xf4, 0x8a, 0x58,
        ];

        // Time received
        let time_request_received = 657207792;
        let canonical_public_key: Vec<u8> = vec![
            0x2C, 0xC5, 0x50, 0x0F, 0x95, 0xF2, 0x12, 0x4F, 0x0F, 0x03, 0x2E, 0xAB, 0xAF, 0xD9,
            0x1C, 0x15, 0x0B, 0xC4, 0x14, 0xBC, 0x1C, 0x30, 0x3C, 0xF8, 0x41, 0x6B, 0xAD, 0xB0,
            0x9B, 0x6E, 0xEA, 0x5F, 0xF9, 0xFE, 0xB6, 0x58, 0xB6, 0xED, 0x6F, 0xB2, 0xF3, 0x48,
            0xC3, 0x8C, 0xD9, 0xB1, 0xFE, 0xD3, 0x32, 0x5D, 0x3D, 0x81, 0xF4, 0x6E, 0x3D, 0x16,
            0x4A, 0x5B, 0xA8, 0xEB, 0x25, 0x2B, 0x40, 0x99,
        ];

        let ee_registration_vid: u64 = 7127127231;

        match eca_enrollment_certificate_request_handler(
            enrollment_certificate_request,
            C_ECA_PRIVATE.to_vec(),
            canonical_public_key,
            eca_cert_chain_test(),
            ee_registration_vid,
            time_request_received,
            300,
            86400,
        ) {
            Ok((encoded, vid, issuer_id)) => {
                assert_eq!(encoded.len(), 545);
                assert_eq!(vid, 7127127231);
                assert_eq!(issuer_id, "5d215da31efc7f75");
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        };
    }
}
