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
use crate::{RaEeCertInfoArgs, encode_ra_ee_cert_info};

use crate::util::{initialize_empty_oscms_octet_buffer, oscms_octet_buffer_to_vec};

use scmscommon::get_current_time_seconds;

pub fn make_certificate_info_spdu(
    current_i_period: u16,
    request_hash: [u8; 8],
) -> Result<Vec<u8>, OscmsBridgeError> {
    // Obtain current time in seconds
    let generation_time = get_current_time_seconds();

    // Calculate next download time as generation time + 60 seconds
    let next_dl_time = generation_time + 60;

    // Allocate buffer for encoded response
    let encoded_ra_ee_cert_info = initialize_empty_oscms_octet_buffer();

    let mut args = RaEeCertInfoArgs {
        generation_time,
        next_dl_time,
        current_i_period,
        request_hash,
        encoded_ra_ee_cert_info,
    };

    // 2. Call handle_ee_ra_download_request and handle result
    let result = unsafe { encode_ra_ee_cert_info(&mut args) };
    if result != 0 {
        return Err(OscmsBridgeError::new(result));
    }

    log::debug!("Make RaEeCertInfo encoded successfully");

    let encoded_output = oscms_octet_buffer_to_vec(&args.encoded_ra_ee_cert_info);

    // Clean up
    unsafe {
        crate::oscms_empty_octet_buffer(&mut args.encoded_ra_ee_cert_info);
    }

    if encoded_output.is_empty() {
        log::debug!("Failed to generate encoded RaEeCertInfo SPDU");
        return Err(OscmsBridgeError::new(-1));
    }

    Ok(encoded_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_certificate_info_spdu() {
        let current_i = 0;
        let request_hash = [1, 2, 3, 4, 5, 6, 7, 8];

        match make_certificate_info_spdu(current_i, request_hash) {
            Ok(encoded) => {
                assert_ne!(encoded.len(), 0);
                assert_eq!(encoded.len(), 23);
            }
            Err(_) => {
                panic!("Failed to generate encoded response");
            }
        };
    }
}
