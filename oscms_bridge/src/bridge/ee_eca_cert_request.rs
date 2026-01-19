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
use crate::extract_canonical_id_from_ee_eca_cert_request;
use crate::{OscmsOctetBuffer, oscms_empty_octet_buffer, oscms_octet_buffer_init_from_buffer};

pub fn eca_extract_canonical_id_from_enrollment_request(
    mut enrollment_certificate_request: Vec<u8>,
) -> Result<String, OscmsBridgeError> {
    log::debug!(
        "OSCMS-BRIDGE: Extracting Canonical Id from request size {:?}",
        enrollment_certificate_request.len()
    );

    unsafe {
        let mut encoded_ee_eca_cert_request_spdu = OscmsOctetBuffer {
            data: std::ptr::null_mut(),
            length: 0,
        };

        let mut canonical_id_buffer = OscmsOctetBuffer {
            data: std::ptr::null_mut(),
            length: 0,
        };

        oscms_octet_buffer_init_from_buffer(
            &mut encoded_ee_eca_cert_request_spdu,
            enrollment_certificate_request.as_mut_ptr(),
            enrollment_certificate_request.len(),
        );

        if encoded_ee_eca_cert_request_spdu.data.is_null() {
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }

        let result = extract_canonical_id_from_ee_eca_cert_request(
            &mut encoded_ee_eca_cert_request_spdu,
            &mut canonical_id_buffer,
        );

        if result != 0 {
            // Free buffers
            oscms_empty_octet_buffer(&mut canonical_id_buffer);
            oscms_empty_octet_buffer(&mut encoded_ee_eca_cert_request_spdu);
            return Err(OscmsBridgeError::new(result));
        }

        if canonical_id_buffer.data.is_null() {
            // Free buffers
            oscms_empty_octet_buffer(&mut encoded_ee_eca_cert_request_spdu);
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }

        let mut canonicalId = String::new();
        if canonical_id_buffer.length != 0 {
            // 3. Prepare decoder output by extracting filename from args.filename
            canonicalId = String::from_utf8_lossy(std::slice::from_raw_parts(
                canonical_id_buffer.data,
                canonical_id_buffer.length,
            ))
            .trim_end_matches('\0')
            .to_string();
        }

        // Free buffers
        oscms_empty_octet_buffer(&mut canonical_id_buffer);
        oscms_empty_octet_buffer(&mut encoded_ee_eca_cert_request_spdu);

        log::debug!(
            "Canonical Id extracted from request (if any): {:?}",
            canonicalId
        );

        Ok(canonicalId)
    }
}
