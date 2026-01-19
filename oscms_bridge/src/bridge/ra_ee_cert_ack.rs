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
use crate::encode_ra_ee_cert_ack_spdu;
use crate::util::hashed_binary;
use crate::{OscmsOctetBuffer, oscms_octet_buffer_init_from_buffer};
use crate::{OscmsRaEeCertAck, RaEeCertAckSpduArgs};
use scmscommon::{get_current_time_period_days, get_current_time_seconds};

/// Encode a certificate acknowledgment message.
///
/// # Arguments
///
/// * `input_data`: Input data to be included in the acknowledgment.
/// * `delta_next_dl_time`: Delta time until the next download.
/// * `ra_private`: The private key of the RA.
/// * `ra_certificate_encoded`: The encoded certificate of the RA.
/// * `debug`: A boolean flag indicating whether to enable debug mode or not.
///
/// # Returns
///
/// A `Result` containing the encoded acknowledgment message as a vector of bytes (`Vec<u8>`) if successful,
/// or a `Lgasn1609Error` if encoding encounters an error during execution.
///
/// # Errors
///
/// Returns a `Lgasn1609Error` if encoding encounters an error during execution or if the encoded output is empty.
///
/// # Safety
///
/// This function uses unsafe code to call into an external C function (`raEeCertAckSpduEncode`),
/// so it must be used with caution and must ensure that the provided input data is valid.
///
pub fn encode_certificate_ack(
    input_data: Vec<u8>,
    delta_next_dl_time: u32,
    mut signer_private: Vec<u8>,
    mut signer_certificate: Vec<u8>,
) -> Result<Vec<u8>, OscmsBridgeError> {
    log::debug!("OSCMS-BRIDGE: Encoding Certificate Ack SPDU...");

    // 0. Preparing some input data
    let current_time_secs = get_current_time_seconds();
    let next_dl_time = current_time_secs + delta_next_dl_time;
    let hashed_binary = match hashed_binary(input_data) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("OSCMS-BRIDGE: Error hashing input data: {:?}", e);
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }
    };

    // I value
    let first_i = get_current_time_period_days() as u16;

    log::debug!(
        "OSCMS-BRIDGE: current_time_secs: {}, next_dl_time: {}, first_i: {}",
        current_time_secs,
        next_dl_time,
        first_i
    );

    let oscms_ra_ee_cert_ack_data = OscmsRaEeCertAck {
        generation_time: current_time_secs,
        request_hash: hashed_binary.clone(),
        next_dl_time,
        first_i,
    };

    unsafe {
        let mut oscms_signer_private_key_buffer = OscmsOctetBuffer {
            length: 0,
            data: std::ptr::null_mut(),
        };

        let mut oscms_signer_certificate_buffer = OscmsOctetBuffer {
            length: 0,
            data: std::ptr::null_mut(),
        };

        let oscms_output_buffer = OscmsOctetBuffer {
            length: 0,
            data: std::ptr::null_mut(),
        };

        oscms_octet_buffer_init_from_buffer(
            &mut oscms_signer_private_key_buffer,
            signer_private.as_mut_ptr(),
            signer_private.len(),
        );

        oscms_octet_buffer_init_from_buffer(
            &mut oscms_signer_certificate_buffer,
            signer_certificate.as_mut_ptr(),
            signer_certificate.len(),
        );

        // 1. Create a new instance of RaEeCertAckSpduArgs
        //    This struct holds the input and output values used by
        //    raEeCertAckSpduEncode
        let mut args = RaEeCertAckSpduArgs {
            ra_ee_cert_ack: oscms_ra_ee_cert_ack_data,
            signer_certificate: oscms_signer_certificate_buffer,
            signer_private_key: oscms_signer_private_key_buffer,
            output_spdu: oscms_output_buffer,
        };

        log::debug!("OSCMS-BRIDGE: Calling encode_ra_ee_cert_ack_spdu");

        // 2. Call raEeCertAckSpduEncode and handle result.
        //    Is expected to return a 0 if the function was successful
        //    and args must have output data filled in.

        let result = encode_ra_ee_cert_ack_spdu(&mut args);
        if result != 0 {
            log::error!(
                "OSCMS-BRIDGE:  Error encoding Certificate Ack SPDU: {}",
                result
            );
            return Err(OscmsBridgeError::new(result));
        }

        log::debug!("OSCMS-BRIDGE: encode_ra_ee_cert_ack_spdu returned successfully.");

        // 3. Copy the output buffer from args to the output data (encoded).
        //    In this case, the encoded buffer is a Vec<u8> so we need to
        //    iterate over the output buffer and copy the data to the encoded buffer
        if args.output_spdu.data.is_null() {
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }

        let data_ptr = args.output_spdu.data;
        let length = args.output_spdu.length;
        let encoded = std::slice::from_raw_parts(data_ptr, length).to_vec();

        // 4. Return Ok if everything went well
        log::debug!("OSCMS-BRIDGE: Certificate Ack SPDU encoded successfully.");
        Ok(encoded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ra_certificate_chain() -> Vec<u8> {
        vec![
            0x80, 0x03, 0x00, 0x80, 0x45, 0x6C, 0x07, 0x77, 0xDC, 0x0E, 0xBF, 0x29, 0x00, 0x82,
            0x08, 0x6E, 0xCB, 0x90, 0xFC, 0x1D, 0x75, 0x48, 0x9F, 0x01, 0x02, 0x03, 0x00, 0x00,
            0x23, 0x94, 0x72, 0x6E, 0x84, 0xFF, 0xFF, 0x80, 0x80, 0x82, 0x7C, 0x80, 0x25, 0x67,
            0x96, 0x02, 0x04, 0xCA, 0x5E, 0x7D, 0xAE, 0x21, 0x79, 0x25, 0xAE, 0x56, 0x9A, 0x43,
            0xCE, 0x21, 0x34, 0x27, 0xEB, 0x50, 0x58, 0xF0, 0xC2, 0xAF, 0x36, 0x1E, 0x13, 0x66,
            0x80, 0x80, 0xBF, 0x20, 0x04, 0xDC, 0x87, 0x9A, 0xCA, 0x95, 0x2B, 0xB7, 0xCB, 0x1D,
            0x0E, 0x5E, 0x20, 0x11, 0x0D, 0x57, 0x7F, 0xE2, 0x82, 0xF7, 0xB7, 0x7F, 0xC0, 0x6B,
            0xFD, 0xF3, 0xF5, 0x8C, 0xFF, 0x7A, 0x94, 0xCD, 0xD8, 0x39, 0x2E, 0xCC, 0xC0, 0x0D,
            0x7C, 0x06, 0x5F, 0xD1, 0x68, 0x23, 0x47, 0xAE, 0xE0, 0xAD, 0x0B, 0x58, 0x82, 0x8B,
            0x6C, 0x43, 0x31, 0x70, 0x37, 0x1D, 0x16, 0xB7, 0x41, 0xF0,
        ]
    }

    fn ra_private() -> Vec<u8> {
        vec![
            0xD6, 0xF7, 0x75, 0xB1, 0x73, 0xEB, 0x99, 0x41, 0xE7, 0x2B, 0x3A, 0x02, 0x50, 0xF6,
            0x9C, 0x6D, 0xB6, 0xA4, 0xF3, 0xBB, 0x64, 0x65, 0xCC, 0x50, 0xA2, 0x39, 0x1E, 0x96,
            0x35, 0x76, 0x59, 0x93,
        ]
    }

    #[test]
    fn test_encode_ra_ee_cert_ack_spdu_() {
        let encoded_ee_ra_cert_request_spdu = vec![
            3, 131, 130, 1, 124, 0, 2, 135, 128, 64, 2, 35, 55, 175, 94, 0, 0, 131, 0, 0, 0, 0, 0,
            0, 0, 48, 57, 132, 0, 168, 128, 128, 130, 157, 80, 61, 147, 73, 153, 226, 212, 91, 149,
            126, 241, 72, 113, 9, 54, 168, 63, 165, 144, 75, 30, 59, 140, 223, 133, 20, 30, 198,
            242, 152, 252, 128, 128, 41, 116, 127, 66, 145, 26, 235, 170, 190, 198, 11, 184, 233,
            172, 44, 138, 0, 128, 130, 233, 100, 182, 56, 101, 194, 169, 43, 1, 104, 10, 197, 150,
            255, 35, 167, 63, 75, 24, 144, 191, 30, 195, 174, 107, 55, 82, 23, 55, 156, 69, 184,
            128, 93, 222, 135, 77, 153, 69, 75, 191, 166, 228, 48, 60, 247, 136, 246, 99, 129, 1,
            1, 128, 3, 0, 128, 69, 108, 7, 119, 220, 14, 191, 41, 0, 130, 53, 9, 220, 15, 67, 220,
            226, 90, 28, 228, 33, 109, 172, 24, 103, 69, 168, 28, 245, 104, 103, 206, 71, 137, 21,
            155, 186, 224, 177, 196, 111, 211, 2, 248, 75, 173, 105, 191, 179, 194, 116, 212, 214,
            205, 128, 118, 223, 253, 98, 228, 215, 0, 160, 199, 1, 2, 3, 0, 0, 35, 55, 175, 94,
            132, 255, 255, 128, 128, 131, 253, 172, 213, 156, 56, 254, 231, 238, 252, 210, 120,
            166, 242, 98, 129, 148, 141, 225, 121, 192, 235, 242, 62, 11, 252, 11, 234, 162, 30,
            126, 116, 109, 128, 128, 72, 24, 45, 104, 246, 232, 182, 239, 133, 124, 175, 51, 55,
            131, 12, 48, 101, 217, 205, 214, 247, 219, 20, 77, 30, 157, 67, 130, 4, 154, 155, 199,
            172, 163, 16, 37, 149, 8, 63, 248, 143, 79, 54, 242, 211, 72, 242, 57, 14, 133, 43, 33,
            108, 92, 104, 47, 147, 62, 243, 124, 101, 106, 131, 161, 128, 128, 10, 217, 149, 192,
            224, 13, 106, 204, 82, 183, 42, 68, 178, 157, 233, 128, 237, 141, 38, 0, 31, 6, 190,
            178, 107, 163, 88, 108, 30, 193, 31, 51, 170, 212, 81, 65, 113, 142, 185, 152, 110,
            244, 91, 97, 0, 64, 177, 83, 2, 130, 226, 131, 91, 235, 53, 165, 173, 136, 238, 74, 51,
            95, 92, 79,
        ];
        let delta_next_dl_time = 60;
        let ra_private = ra_private();
        let ra_certificate = ra_certificate_chain();

        scmscommon::setup_global_config();

        match encode_certificate_ack(
            encoded_ee_ra_cert_request_spdu,
            delta_next_dl_time,
            ra_private,
            ra_certificate,
        ) {
            Ok(encoded) => {
                assert_ne!(encoded.len(), 0);
                assert_eq!(encoded.len(), 238);
            }
            Err(e) => {
                panic!("Error: {:?}", e);
            }
        }
    }
}
