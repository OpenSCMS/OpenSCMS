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
use crate::{
    CertificateManagementInformationStatusSpduArgs,
    create_certificate_management_information_status_spdu,
};
use crate::{OscmsOctetBuffer, oscms_empty_octet_buffer, oscms_octet_buffer_init_from_buffer};

pub fn make_certificate_mgmt_info_status_file_encoded(
    ra_hostname: String,
    ma_psid_list: Vec<Vec<u64>>,
    mut ma_updated_time_list: Vec<u32>,
    mut ctl_sequence_number_list: Vec<u16>,
    ctl_series_id_list: Vec<Vec<u8>>,
    mut ctl_updated_time_list: Vec<u32>,
    crl_craca_id_list: Vec<Vec<u8>>,
    mut crl_series_id_list: Vec<u16>,
    mut crl_issue_date_list: Vec<u32>,
    ca_ccf_updated_time: u32,
    ra_updated_time: u32,
    mut ra_private: Vec<u8>,
    mut ra_certificate: Vec<u8>,
) -> Result<(Vec<u8>, String), OscmsBridgeError> {
    log::debug!(
        "OSCMS-BRIDGE - Making Certificate Management Info Status file encoded for RA hostname: {}",
        ra_hostname
    );

    // Preparing MA info
    if ma_psid_list.len() != ma_updated_time_list.len() {
        log::debug!(
            "PSID list and updated time list have different sizes: {} and {}",
            ma_psid_list.len(),
            ma_updated_time_list.len()
        );
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }

    let mut sequence_of_ma_psid_list_sizes: Vec<usize> = ma_psid_list
        .iter()
        .map(|psid_list| psid_list.len())
        .collect();

    let mut sequence_of_ma_psid_list: Vec<*mut u64> = ma_psid_list
        .iter()
        .map(|v| v.as_ptr() as *mut u64)
        .collect();

    // Prepare CTL info
    if ctl_sequence_number_list.len() != ctl_updated_time_list.len() {
        log::debug!(
            "Sequence number list and updated time list have different sizes: {} and {}",
            ctl_sequence_number_list.len(),
            ctl_updated_time_list.len()
        );
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }

    let mut sequence_of_ctl_series_id_list: Vec<*mut u8> = ctl_series_id_list
        .iter()
        .map(|v| v.as_ptr() as *mut u8)
        .collect();

    // Prepare CRL info
    if crl_craca_id_list.len() != crl_series_id_list.len()
        || crl_series_id_list.len() != crl_issue_date_list.len()
    {
        log::debug!(
            "CRL lists have different sizes: {}, {} and {}",
            crl_craca_id_list.len(),
            crl_series_id_list.len(),
            crl_issue_date_list.len()
        );
        return Err(OscmsBridgeError::new(
            OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
        ));
    }

    let mut sequence_of_crl_craca_id_list: Vec<*mut u8> = crl_craca_id_list
        .iter()
        .map(|v| v.as_ptr() as *mut u8)
        .collect();

    let mut oscms_signer_private_key_buffer = OscmsOctetBuffer {
        length: 0,
        data: std::ptr::null_mut(),
    };
    let mut oscms_signer_certificate_buffer = OscmsOctetBuffer {
        length: 0,
        data: std::ptr::null_mut(),
    };

    let mut oscms_output_buffer = OscmsOctetBuffer {
        length: 0,
        data: std::ptr::null_mut(),
    };

    unsafe {
        let mut result = oscms_octet_buffer_init_from_buffer(
            &mut oscms_signer_private_key_buffer,
            ra_private.as_mut_ptr(),
            ra_private.len(),
        );

        if result != 0 {
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }

        result = oscms_octet_buffer_init_from_buffer(
            &mut oscms_signer_certificate_buffer,
            ra_certificate.as_mut_ptr(),
            ra_certificate.len(),
        );

        if result != 0 {
            oscms_empty_octet_buffer(&mut oscms_signer_private_key_buffer);
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }
    }

    let mut args = CertificateManagementInformationStatusSpduArgs {
        sequence_of_ma_psid_list: sequence_of_ma_psid_list.as_mut_ptr(),
        sequence_of_ma_psid_list_sizes: sequence_of_ma_psid_list_sizes.as_mut_ptr(),
        sequence_of_ma_updated_time: ma_updated_time_list.as_mut_ptr(),
        sequence_of_ma_size: ma_psid_list.len(),
        sequence_of_ctl_sequence_number: ctl_sequence_number_list.as_mut_ptr(),
        sequence_of_ctl_series_id: sequence_of_ctl_series_id_list.as_mut_ptr(),
        sequence_of_ctl_last_update: ctl_updated_time_list.as_mut_ptr(),
        sequence_of_ctl_size: ctl_sequence_number_list.len(),
        sequence_of_crl_craca_id: sequence_of_crl_craca_id_list.as_mut_ptr(),
        sequence_of_crl_series: crl_series_id_list.as_mut_ptr(),
        sequence_of_crl_issue_date: crl_issue_date_list.as_mut_ptr(),
        sequence_of_crl_size: crl_craca_id_list.len(),
        ca_ccf_updated_time: ca_ccf_updated_time,
        ra_updated_time: ra_updated_time,
        signer_certificate: oscms_signer_certificate_buffer,
        signer_private_key: oscms_signer_private_key_buffer,
        output_spdu: oscms_output_buffer,
    };

    log::debug!("OSCMS-BRIDGE - Calling create_certificate_management_information_status_spdu");

    unsafe {
        let result = create_certificate_management_information_status_spdu(&mut args);
        if result != 0 {
            oscms_empty_octet_buffer(&mut oscms_signer_private_key_buffer);
            oscms_empty_octet_buffer(&mut oscms_signer_certificate_buffer);
            oscms_empty_octet_buffer(&mut oscms_output_buffer);
            return Err(OscmsBridgeError::new(result));
        }

        // 3. Copy the output buffer from args to the output data (encoded).
        //    In this case, the encoded buffer is a Vec<u8> so we need to
        //    iterate over the output buffer and copy the data to the encoded buffer
        if args.output_spdu.data.is_null() {
            oscms_empty_octet_buffer(&mut oscms_signer_private_key_buffer);
            oscms_empty_octet_buffer(&mut oscms_signer_certificate_buffer);
            oscms_empty_octet_buffer(&mut oscms_output_buffer);
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }

        let data_ptr = args.output_spdu.data;
        let length = args.output_spdu.length;
        let encoded = std::slice::from_raw_parts(data_ptr, length).to_vec();

        // Free buffers
        oscms_empty_octet_buffer(&mut oscms_signer_private_key_buffer);
        oscms_empty_octet_buffer(&mut oscms_signer_certificate_buffer);
        oscms_empty_octet_buffer(&mut oscms_output_buffer);

        // Filename as specified
        // The filename is scms-management-infostatus_RA-or-DC-Hostname_lastUpdateDate.oer.
        // The last update date is expressed as a Coordinated
        // Universal Time (UTC) time in ISO 8601-1 format with hours, minutes, and seconds given as follows: YYYYMM-
        // DDTHH:MM:SSZ.
        let formatted_last_update_date =
            chrono::DateTime::from_timestamp(ra_updated_time as i64, 0);
        if formatted_last_update_date.is_none() {
            return Err(OscmsBridgeError::new(
                OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
            ));
        }
        let formatted_last_update_date = formatted_last_update_date.unwrap();
        let formatted_last_update_date = formatted_last_update_date.format("%Y%m%dT%H:%M:%SZ");
        let filename = format!(
            "scms-management-infostatus_{}_{}.oer",
            ra_hostname, formatted_last_update_date
        );

        log::debug!(
            "OSCMS-BRIDGE - Created Certificate Management Info Status file encoded with filename: {} and size: {} bytes",
            filename,
            encoded.len()
        );

        // 4. Return Ok if everything went well
        Ok((encoded, filename))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ra_certificate() -> Vec<u8> {
        vec![
            0x80, 0x03, 0x00, 0x80, 0xA5, 0xE9, 0x44, 0x21, 0x4B, 0xE5, 0xB9, 0xE6, 0x01, 0x82,
            0x10, 0xB0, 0x95, 0x48, 0xE7, 0xBC, 0x68, 0x58, 0x96, 0x4C, 0x8C, 0xA1, 0xD3, 0x0E,
            0x8D, 0x56, 0xCF, 0x01, 0x02, 0x03, 0x00, 0x00, 0x27, 0x27, 0x38, 0x80, 0x86, 0x00,
            0x0A, 0x00, 0x80, 0x83, 0xD3, 0xD6, 0xD4, 0x1F, 0x3B, 0x4E, 0x81, 0x20, 0x9E, 0x2C,
            0x06, 0x8E, 0x5D, 0xD0, 0x5C, 0x07, 0xA9, 0x7C, 0xC1, 0x05, 0x5E, 0xB9, 0x47, 0x32,
            0x0D, 0xC9, 0x9F, 0xF7, 0xF2, 0x67, 0xDD, 0x19, 0x80, 0x80, 0x82, 0xC9, 0x17, 0x9C,
            0x9F, 0x36, 0xD7, 0xF0, 0xFF, 0xF4, 0x25, 0x22, 0x41, 0x55, 0xDE, 0xE3, 0x4E, 0x27,
            0x6D, 0xD3, 0x6D, 0xAC, 0x15, 0x37, 0xA8, 0x56, 0x91, 0x2E, 0x79, 0x25, 0xC0, 0x4A,
            0x58, 0x80, 0x80, 0xE6, 0x85, 0x92, 0xDB, 0x43, 0xEF, 0xC1, 0xD1, 0x93, 0x84, 0x20,
            0xDA, 0x21, 0x5B, 0x82, 0xF0, 0x60, 0x70, 0x09, 0x74, 0x6E, 0x04, 0x3E, 0x41, 0xE8,
            0x1F, 0xA0, 0xE0, 0xAE, 0x83, 0xE4, 0xBF, 0xF2, 0x5A, 0x59, 0x56, 0x6C, 0xA0, 0x4D,
            0x28, 0xA7, 0x1A, 0x61, 0x19, 0x1F, 0x36, 0xA1, 0xCA, 0x3E, 0xB5, 0x19, 0x51, 0xEF,
            0x8B, 0x07, 0x21, 0xF8, 0x8F, 0x5A, 0xEC, 0xA7, 0x6C, 0xD0, 0xF5,
        ]
    }

    fn ra_enc_private_key() -> Vec<u8> {
        vec![
            0xEB, 0xCE, 0xC0, 0x4E, 0xD3, 0xAC, 0x08, 0xF0, 0xDF, 0x22, 0x1C, 0x00, 0xDC, 0x5C,
            0x75, 0x1B, 0x91, 0x0E, 0x2C, 0x6B, 0x5E, 0xD7, 0xD1, 0x87, 0x96, 0xFE, 0x33, 0x04,
            0x11, 0x47, 0xE8, 0x59,
        ]
    }

    #[test]
    fn test_make_certificate_mgmt_info_status_file_encoded() {
        let ra_hostname = "ra_hostname".to_string();
        let ma_psid_list = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let ma_updated_time_list = vec![1, 2];
        let ctl_sequence_number_list = vec![1, 2];
        let ctl_series_id_list = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let ctl_updated_time_list = vec![1, 2];
        let crl_craca_id_list = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let crl_series_id_list = vec![1, 2];
        let crl_issue_date_list = vec![1, 2];
        let ca_ccf_updated_time = 1;
        let ra_updated_time = 1;
        let ra_private = ra_enc_private_key();
        let ra_certificate = ra_certificate();

        let result = make_certificate_mgmt_info_status_file_encoded(
            ra_hostname,
            ma_psid_list,
            ma_updated_time_list,
            ctl_sequence_number_list,
            ctl_series_id_list,
            ctl_updated_time_list,
            crl_craca_id_list,
            crl_series_id_list,
            crl_issue_date_list,
            ca_ccf_updated_time,
            ra_updated_time,
            ra_private,
            ra_certificate,
        );

        assert!(result.is_ok());
        let result_values = result.unwrap();
        let encoded_output = result_values.0;
        assert!(!encoded_output.is_empty());
        let filename = result_values.1;
        assert_eq!(
            filename,
            "scms-management-infostatus_ra_hostname_19700101T00:00:01Z.oer"
        );
    }
}
