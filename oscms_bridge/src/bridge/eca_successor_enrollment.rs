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
use crate::OscmsOctetBuffer;
use crate::{
    EcaSuccessorEnrollmentArgs, EeEcaCertRequestCheckParams, empty_eca_successor_enrollment_args,
    handle_eca_successor_enrollment,
};

use crate::util::{
    initialize_empty_oscms_octet_buffer, initialize_oscms_new_octet_buffer_from_vec,
    oscms_octet_buffer_to_vec,
};

use byteorder::{ByteOrder, LittleEndian};
use scmscommon::GlobalConfig;

pub fn eca_successor_enrollment_certificate_request_handler(
    successor_enrollment_request: Vec<u8>,
    request_hash_id: String,
    eca_private_key: Vec<u8>,
    eca_public_uncompressed: Vec<u8>,
    eca_certificate_chain: Vec<Vec<u8>>,
    time_request_received: u32,
) -> Result<(String, Vec<u8>, (u64, String)), OscmsBridgeError> {
    log::debug!(
        "OSCMS-BRIDGE: Start ecaSuccessorRequestHandler processing: {:?}",
        successor_enrollment_request.len()
    );

    let mut params = EeEcaCertRequestCheckParams {
        time_request_received,
        eca_max_wait: GlobalConfig::global().param_eca_max_wait as u32,
        eca_max_age: GlobalConfig::global().param_eca_max_age as u32,
    };

    // Inputs
    let encoded_successor_enrollment_request =
        initialize_oscms_new_octet_buffer_from_vec(successor_enrollment_request.clone());
    let issuer_public_key =
        initialize_oscms_new_octet_buffer_from_vec(eca_public_uncompressed.to_vec());
    let issuer_private_key = initialize_oscms_new_octet_buffer_from_vec(eca_private_key.clone());
    let issuer_encoded_certificate =
        initialize_oscms_new_octet_buffer_from_vec(eca_certificate_chain[0].clone());

    let issuer_encoded_certificates_chain: Vec<OscmsOctetBuffer> = eca_certificate_chain
        .iter()
        .map(|v| initialize_oscms_new_octet_buffer_from_vec(v.clone()))
        .collect();

    // output buffer
    let device_id = initialize_empty_oscms_octet_buffer();
    let issuer_id = initialize_empty_oscms_octet_buffer();
    let signer_id = initialize_empty_oscms_octet_buffer();
    let output_successor_enrollment_certificate = initialize_empty_oscms_octet_buffer();

    let mut args = EcaSuccessorEnrollmentArgs {
        encoded_successor_enrollment_request,
        issuer_public_key,
        issuer_private_key,
        issuer_encoded_certificate,
        issuer_encoded_certificates_chain_count: eca_certificate_chain.len(),
        issuer_encoded_certificates_chain: issuer_encoded_certificates_chain.as_ptr(),
        craca_id: [0u8; 3],
        crl_series: 0,
        device_id,
        issuer_id,
        signer_id,
        output_successor_enrollment_certificate,
        request_hash: [0; 8],
    };

    let result = unsafe { handle_eca_successor_enrollment(&mut args, &mut params) };
    if result != 0 {
        unsafe {
            empty_eca_successor_enrollment_args(&mut args, false);
        }

        return Err(OscmsBridgeError::new(result));
    }

    // Those values are 12-bytes hex strings, so we need to read them as String
    let signer_output_value = oscms_octet_buffer_to_vec(&args.signer_id);
    if signer_output_value.is_empty() {
        log::error!("Output successor enrollment certificate is empty");
        unsafe {
            empty_eca_successor_enrollment_args(&mut args, false);
        }
        return Err(OscmsBridgeError::new(-1));
    }
    let signer_vid = LittleEndian::read_u64(&signer_output_value);

    let device_id_output_value = oscms_octet_buffer_to_vec(&args.device_id);
    if device_id_output_value.is_empty() {
        log::error!("Output successor enrollment certificate is empty");
        unsafe {
            empty_eca_successor_enrollment_args(&mut args, false);
        }
        return Err(OscmsBridgeError::new(-1));
    }
    let vid = LittleEndian::read_u64(&device_id_output_value);

    // both should be the same:
    if vid != signer_vid {
        log::error!(
            "Vid and SignerVid are not the same: Vid: {}, SignerVid: {}",
            vid,
            signer_vid
        );
        unsafe {
            empty_eca_successor_enrollment_args(&mut args, false);
        }
        return Err(OscmsBridgeError::new(-1));
    }

    let issuer_id_output_value = oscms_octet_buffer_to_vec(&args.issuer_id);
    if issuer_id_output_value.is_empty() {
        log::error!("Output successor enrollment certificate is empty");
        unsafe {
            empty_eca_successor_enrollment_args(&mut args, false);
        }
        return Err(OscmsBridgeError::new(-1));
    }

    let issuer_id_hex: String = issuer_id_output_value
        .iter()
        .map(|byte| format!("{:02x}", byte))
        .collect();

    log::debug!(
        "Vid: {:?}, IssuerId: {:?}, SignerId: {:?}",
        vid,
        issuer_id_hex,
        signer_vid,
    );

    // 3. Prepare decoder output by extracting filename from args.filename
    // Generate the filename as specified in 8.3.2
    // xxxxxxxxxxxxxxxx_enroll.oer where x is the hashId8 if the request
    let filename = format!("{}_enroll.oer", request_hash_id);
    let output_successor_enrollment_certificate =
        oscms_octet_buffer_to_vec(&args.output_successor_enrollment_certificate);

    if output_successor_enrollment_certificate.is_empty() {
        log::error!("Output successor enrollment certificate is empty");
        unsafe {
            empty_eca_successor_enrollment_args(&mut args, false);
        }

        return Err(OscmsBridgeError::new(-1));
    }

    unsafe {
        empty_eca_successor_enrollment_args(&mut args, false);
    }

    log::debug!(
        "Finish ecaSuccessorRequestHandler processing: filename {}, content len {}",
        filename,
        output_successor_enrollment_certificate.len()
    );

    // 4. Return Ok if everything went well
    Ok((
        filename,
        output_successor_enrollment_certificate,
        (vid, issuer_id_hex),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;
    use scmscommon::setup_global_config;

    static C_ECA_PRIVATE: [u8; 32] = [
        0xD0, 0x23, 0xD4, 0x9F, 0x14, 0x52, 0x75, 0x11, 0x09, 0x95, 0xF8, 0x8D, 0xDA, 0x3A, 0x51,
        0x2F, 0xFE, 0x3B, 0xE0, 0x5C, 0x56, 0x53, 0xAD, 0x99, 0x68, 0x1B, 0x38, 0x5B, 0x38, 0xE5,
        0x4D, 0x34,
    ];

    static C_ECA_PUBLIC_UNCOMPRESSED: [u8; 64] = [
        0x65, 0x1F, 0xB1, 0x57, 0x20, 0xAE, 0x59, 0xAD, 0x41, 0x5D, 0x69, 0x1D, 0x21, 0x57, 0x43,
        0x7D, 0x38, 0xC0, 0xEA, 0xB9, 0x7F, 0x90, 0xAE, 0x3F, 0x1C, 0xC1, 0xF2, 0x41, 0x24, 0x8D,
        0xB0, 0xF2, 0x5E, 0x68, 0xA8, 0xED, 0x1B, 0xE1, 0x00, 0x9B, 0x05, 0x74, 0xB9, 0x3C, 0x9A,
        0x41, 0x7F, 0x96, 0x61, 0x53, 0x74, 0x23, 0xDC, 0xC8, 0x22, 0x9B, 0x65, 0xC9, 0xCC, 0x31,
        0x55, 0xBD, 0x4A, 0x13,
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
    fn test_eca_successor_enrollment_certificate_request_handler() {
        let decoded_successor_enrollment_certificate_request = vec![
            0x03, 0x83, 0x82, 0x01, 0x63, 0x00, 0x02, 0x87, 0x84, 0x03, 0x83, 0x81, 0xc9, 0x00,
            0x02, 0x85, 0x80, 0x40, 0x02, 0x27, 0x2c, 0x63, 0x0a, 0x01, 0x06, 0x83, 0xb7, 0xb9,
            0x02, 0x00, 0x04, 0x27, 0x2c, 0x63, 0x0a, 0x86, 0x00, 0x06, 0x01, 0x01, 0xe0, 0x81,
            0x01, 0x00, 0x01, 0x01, 0xc0, 0x80, 0x80, 0x83, 0x0a, 0xdb, 0x98, 0xb3, 0x1c, 0x5a,
            0xd0, 0xae, 0x6b, 0x4b, 0x58, 0x56, 0xef, 0x95, 0x2d, 0x51, 0x1d, 0xe2, 0x62, 0x9a,
            0xbe, 0xf3, 0xe1, 0x31, 0x9d, 0x06, 0xe6, 0xd0, 0xc9, 0xed, 0x78, 0xa6, 0x40, 0x31,
            0x31, 0x44, 0x30, 0x39, 0x46, 0x41, 0x35, 0x39, 0x33, 0x33, 0x42, 0x46, 0x45, 0x36,
            0x31, 0x45, 0x43, 0x34, 0x41, 0x32, 0x36, 0x41, 0x30, 0x36, 0x45, 0x30, 0x32, 0x34,
            0x32, 0x30, 0x33, 0x41, 0x36, 0x36, 0x39, 0x39, 0x41, 0x35, 0x44, 0x34, 0x45, 0x33,
            0x39, 0x37, 0x33, 0x39, 0x43, 0x43, 0x44, 0x43, 0x35, 0x46, 0x43, 0x42, 0x42, 0x32,
            0x33, 0x37, 0x35, 0x39, 0x43, 0x36, 0x36, 0x82, 0x80, 0x80, 0x8a, 0x19, 0xd2, 0xaa,
            0x36, 0x40, 0x2a, 0xaf, 0x2b, 0x4b, 0x62, 0x0c, 0x00, 0xd1, 0xb8, 0x52, 0x14, 0xfc,
            0xd6, 0xac, 0xdf, 0x6f, 0x0b, 0xe9, 0xda, 0x47, 0x29, 0xe4, 0x7d, 0x4a, 0x30, 0xed,
            0xc3, 0x94, 0x73, 0xb6, 0xbf, 0x41, 0xb2, 0xd0, 0xe1, 0x10, 0x6a, 0x77, 0x24, 0xda,
            0xa4, 0x13, 0x0a, 0xb7, 0x29, 0x7e, 0xf2, 0xb5, 0x1f, 0x13, 0xb7, 0x69, 0x91, 0x4b,
            0x62, 0x00, 0x1f, 0x7b, 0x81, 0x01, 0x01, 0x00, 0x03, 0x01, 0x80, 0x5d, 0x21, 0x5d,
            0xa3, 0x1e, 0xfc, 0x7f, 0x75, 0x00, 0x82, 0x10, 0x99, 0xdb, 0x3e, 0x3a, 0x12, 0x37,
            0x1a, 0x75, 0x46, 0x7a, 0x1a, 0x02, 0xc4, 0xbe, 0xca, 0xb6, 0x01, 0x02, 0x03, 0x00,
            0x00, 0x27, 0x2c, 0x63, 0x02, 0x86, 0x00, 0x18, 0x81, 0x82, 0x0c, 0x69, 0xd5, 0xc8,
            0x8a, 0x60, 0xb4, 0x62, 0x5d, 0xab, 0x51, 0x00, 0x1b, 0x86, 0xc3, 0x9e, 0x37, 0x83,
            0x5d, 0x36, 0x2b, 0x97, 0x38, 0xe4, 0x80, 0x8f, 0x97, 0x94, 0x95, 0xcb, 0xad, 0xbf,
            0x80, 0x80, 0x7a, 0x88, 0xce, 0x85, 0xe6, 0xdc, 0x17, 0x6f, 0xca, 0xd8, 0xd0, 0x0d,
            0xfc, 0x06, 0x8f, 0xc4, 0xf3, 0xec, 0xba, 0xb9, 0x0d, 0x72, 0x52, 0xad, 0xa8, 0x33,
            0x38, 0x8f, 0x45, 0x26, 0x1b, 0x8b, 0x26, 0x78, 0x74, 0xaf, 0xa0, 0x49, 0x3c, 0x70,
            0x5b, 0xd2, 0x69, 0xc4, 0x24, 0x61, 0x45, 0x64, 0x3c, 0x14, 0x65, 0x9c, 0x7e, 0xba,
            0xc0, 0x89, 0xd7, 0x33, 0x9d, 0xb4, 0xe7, 0x57, 0x47, 0x1b,
        ];
        // Getting global config object
        setup_global_config();

        // Time received
        let time_request_received = 657220362;
        let request_hash_id = "7062e221b21f4fdd".to_string();

        match eca_successor_enrollment_certificate_request_handler(
            decoded_successor_enrollment_certificate_request,
            request_hash_id,
            C_ECA_PRIVATE.to_vec(),
            C_ECA_PUBLIC_UNCOMPRESSED.to_vec(),
            eca_cert_chain_test(),
            time_request_received,
        ) {
            Ok((filename, content, (_, _))) => {
                let re = Regex::new(r"^[0-9a-zA-Z]{1,16}_enroll\.oer$").unwrap();
                assert!(re.is_match(&filename));
                assert_eq!(content.len(), 497);
            }
            Err(e) => {
                // Is expected to failed because enrollment certificate validity
                if e == OscmsBridgeError::new(
                    crate::OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ENROLLMENT_CERTIFICATES_VALIDITY_PERIOD) {
                    return;
                }
                if e == OscmsBridgeError::new(crate::OscmsErrorCode_OSCMS_ERROR_FAILED_DECRYPTION) {
                    return;
                }
                panic!("Error: {:?}", e);
            }
        };
    }
}
