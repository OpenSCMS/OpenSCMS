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

use crate::{AcaEeCertResponseArgs, handle_aca_ee_cert_response};
use crate::{OscmsBridgeError, empty_aca_ee_cert_response_args};
use scmscommon::{CertificateType, ExpansionType};

fn exp_type_to_oscms_flavor_type(exp_type: ExpansionType) -> crate::OscmsAdditionalParamsType {
    match exp_type {
        ExpansionType::Original => {
            crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_ORIGINAL
        }
        ExpansionType::Unified => {
            crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_UNIFIED
        }
        ExpansionType::Compact => {
            crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_COMPACT_UNIFIED
        }
        ExpansionType::NonButterfly => {
            crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_NONE
        }
        ExpansionType::NonButterflyEncrypted => {
            crate::OscmsAdditionalParamsType_OSCMS_ADDITIONAL_PARAMS_TYPE_ENCRYPTION_KEY
        }
    }
}

fn certificate_type_to_oscms_cert_type(cert_type: CertificateType) -> crate::OscmsCertificateType {
    match cert_type {
        CertificateType::Explicit => crate::OscmsCertificateType_OSCMS_CERTIFICATE_TYPE_EXPLICIT,
        CertificateType::Implicit => crate::OscmsCertificateType_OSCMS_CERTIFICATE_TYPE_IMPLICIT,
    }
}

pub fn issue_authorization_certificate(
    aca_private_key: Vec<u8>,
    aca_encoded_certificate: Vec<u8>,
    caterpillar_public_key: Vec<u8>,
    encryption_key: Vec<u8>,
    private_key_info: Option<Vec<u8>>,
    scms_Type: ExpansionType,
    certificate_validity_begin: u32,
    certificate_validity_duration: u16,
    certificate_type: CertificateType,
    craca_id: [u8; 3],
    crl_series: u16,
) -> Result<Vec<u8>, OscmsBridgeError> {
    log::debug!(
        "OSCMS-BRIDGE: Bridging ACA P-256 with SCMS type: {:?}",
        scms_Type
    );

    // Inputs
    let private_key_info_buffer =
        crate::initialize_oscms_new_octet_buffer_from_vec(private_key_info.unwrap_or_default());
    let caterpillar_public_key_buffer =
        crate::initialize_oscms_new_octet_buffer_from_vec(caterpillar_public_key);
    let encryption_key_buffer = crate::initialize_oscms_new_octet_buffer_from_vec(encryption_key);
    let aca_private_key_buffer = crate::initialize_oscms_new_octet_buffer_from_vec(aca_private_key);
    let aca_encoded_certificate_buffer =
        crate::initialize_oscms_new_octet_buffer_from_vec(aca_encoded_certificate);

    // Output
    let encoded_aca_ee_cert_response_spdu_buffer = crate::initialize_empty_oscms_octet_buffer();

    let mut args = AcaEeCertResponseArgs {
        caterpillar_public_key: caterpillar_public_key_buffer,
        encryption_key: encryption_key_buffer,
        private_key_info: private_key_info_buffer,
        response_certificate_validity_duration_years: certificate_validity_duration,
        response_certificate_validity_begin: certificate_validity_begin,
        aca_private_key: aca_private_key_buffer,
        aca_encoded_certificate: aca_encoded_certificate_buffer,
        certificate_type: certificate_type_to_oscms_cert_type(certificate_type),
        certificate_flavor_type: exp_type_to_oscms_flavor_type(scms_Type),
        craca_id,
        crl_series,
        encoded_aca_ee_cert_response_spdu: encoded_aca_ee_cert_response_spdu_buffer,
    };

    // Call unsafe function to bridge ACA P-256

    let result = unsafe { handle_aca_ee_cert_response(&mut args) };
    if result != 0 {
        unsafe {
            empty_aca_ee_cert_response_args(&mut args);
        }
        return Err(OscmsBridgeError::new(result));
    }

    log::debug!("ACA P-256 bridging successful");

    let encoded_response =
        crate::oscms_octet_buffer_to_vec(&args.encoded_aca_ee_cert_response_spdu);

    // Clean up buffers
    unsafe {
        empty_aca_ee_cert_response_args(&mut args);
    }

    log::debug!("OSCMS-BRIDGE: ACA P-256 bridging completed successfully");
    Ok(encoded_response)
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmscommon::ExpansionType;

    #[test]
    fn test_issue_authorization_certificate() {
        let aca_private: Vec<u8> = vec![
            0x52, 0xb2, 0xb8, 0xf4, 0xb8, 0x47, 0x46, 0x73, 0xbd, 0xab, 0x63, 0x73, 0x52, 0x8c,
            0xa2, 0xf5, 0x08, 0x1c, 0x75, 0x37, 0x6b, 0x99, 0xe9, 0x35, 0x57, 0x8c, 0x73, 0x43,
            0xd0, 0x76, 0x1e, 0x2d,
        ];
        let aca_certificate: Vec<u8> = vec![
            0x80, 0x03, 0x00, 0x80, 0xBA, 0xC7, 0x00, 0x87, 0xDC, 0x64, 0xF2, 0x90, 0x00, 0x81,
            0x0F, 0x61, 0x63, 0x61, 0x2E, 0x6C, 0x67, 0x73, 0x63, 0x6D, 0x73, 0x2E, 0x74, 0x65,
            0x73, 0x74, 0x01, 0x02, 0x03, 0x00, 0x00, 0x26, 0xF7, 0x28, 0x3C, 0x84, 0x00, 0x0A,
            0x80, 0x80, 0x83, 0x2B, 0xEC, 0x66, 0x08, 0x0C, 0x3B, 0xC2, 0x59, 0xE5, 0xE9, 0x64,
            0xE3, 0xA2, 0x0E, 0x41, 0x07, 0x20, 0x05, 0x25, 0xB9, 0xE9, 0x1B, 0xFE, 0xBF, 0x34,
            0x07, 0x15, 0x61, 0x8E, 0xD5, 0x0D, 0x4F, 0x80, 0x80, 0xDD, 0x00, 0x9B, 0x18, 0xF0,
            0x3D, 0x62, 0xC9, 0x70, 0x05, 0xCD, 0x96, 0xED, 0x00, 0xD0, 0x3D, 0x55, 0x34, 0xAA,
            0x9E, 0xA3, 0x09, 0x10, 0xAA, 0x3C, 0xA9, 0x72, 0x56, 0x02, 0xC6, 0xBF, 0x94, 0xD9,
            0x55, 0xAB, 0x3B, 0xE8, 0x1E, 0x3F, 0x11, 0xC4, 0xA1, 0x04, 0x25, 0x5B, 0x9E, 0x1D,
            0xB5, 0x64, 0xF6, 0xDF, 0x24, 0x4A, 0x9F, 0x83, 0x72, 0x8F, 0x6B, 0xCC, 0x74, 0x6F,
            0xF8, 0x8B, 0xC3,
        ];
        let param_S: Vec<u8> = vec![
            0xb9, 0xc4, 0xe2, 0x91, 0xaf, 0x3b, 0xf2, 0x1c, 0x84, 0x8c, 0x85, 0x74, 0x94, 0xba,
            0xe1, 0x5f, 0x57, 0x7a, 0x4d, 0xe9, 0x77, 0x84, 0x34, 0x05, 0x7b, 0xb2, 0xc4, 0xb6,
            0xdb, 0x89, 0x8f, 0xa8, 0x71, 0x6b, 0x29, 0x76, 0x75, 0xe5, 0xfd, 0x5f, 0x39, 0x0b,
            0x15, 0x42, 0x92, 0x3e, 0x10, 0x8d, 0x9c, 0xca, 0x86, 0x0c, 0x73, 0x4b, 0x0f, 0x67,
            0xd3, 0x40, 0x0c, 0x0b, 0x3d, 0xd6, 0x8a, 0x16,
        ];
        let param_E: Vec<u8> = vec![
            0xdf, 0x54, 0x08, 0x95, 0x75, 0x5d, 0x3a, 0x9e, 0xba, 0x1b, 0x47, 0xee, 0x46, 0xd9,
            0x14, 0x91, 0x25, 0x77, 0xb0, 0x70, 0x8f, 0x38, 0x67, 0x76, 0xba, 0x1e, 0xeb, 0x3d,
            0x52, 0x7a, 0x85, 0xe8, 0xb6, 0x35, 0x84, 0x5c, 0xc5, 0xfd, 0xe1, 0xee, 0x3d, 0x73,
            0x18, 0xcb, 0x68, 0x2d, 0x43, 0x99, 0x7b, 0xdd, 0x4c, 0xe2, 0xe0, 0x17, 0x73, 0xaf,
            0x94, 0xe0, 0x8d, 0x55, 0x1d, 0x6b, 0xe2, 0xea,
        ];
        let scms_type = ExpansionType::Original;
        let certificate_validity_duration = 123;
        let certificate_validity_start = 69420;
        let craca_id = [0x01, 0x02, 0x03];
        let crl_series = 0;

        let privave_key_info: Option<Vec<u8>> = Some(vec![0; 32]);
        let certificate_type = CertificateType::Explicit;

        match issue_authorization_certificate(
            aca_private,
            aca_certificate,
            param_S,
            param_E,
            privave_key_info,
            scms_type,
            certificate_validity_start,
            certificate_validity_duration,
            certificate_type,
            craca_id,
            crl_series,
        ) {
            Ok(encoded_response) => {
                assert_ne!(encoded_response.len(), 0);
            }
            Err(e) => {
                panic!("Failed to generate encoded response: {:?}", e);
            }
        }
    }
}
