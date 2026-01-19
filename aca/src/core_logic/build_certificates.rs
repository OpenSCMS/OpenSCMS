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

use crate::core_logic::decrypt_eplv_pair::convert_verifying_key_into_bytes_64;
use crate::core_logic::decrypt_eplv_pair::decrypt_eplv_pair;
use crate::core_logic::derive_lv::derive_lv;
use crate::core_types::LvToPlvMap;
use crate::errors::GenCertificateError;
use crate::persistence::aca_certificates::{latest_aca_certificate, latest_aca_private_key};
use scmscommon::GlobalConfig;
use scmscommon::core_types::Certificate;
use scmscommon::core_types::CertificateType;
use scmscommon::core_types::CocoonRequest;
use scmscommon::core_types::ExpansionType;
use scmscommon::core_types::NonButterflyRequest;
use scmscommon::core_types::PayloadRaToAca;
use sea_orm::DatabaseConnection;

pub async fn build_certificates(
    payload_ra_to_aca: PayloadRaToAca,
    db: &DatabaseConnection,
) -> Result<(Vec<Certificate>, Vec<LvToPlvMap>), GenCertificateError> {
    log::info!("Start to build certificate");
    let cocoon_requests = payload_ra_to_aca.cocoon_requests;
    let exp_type = payload_ra_to_aca.exp_type;
    let certificate_type = payload_ra_to_aca.certificate_type;
    let mut certificates: Vec<Certificate> = Vec::new();
    let mut lv_to_plv_maps: Vec<LvToPlvMap> = Vec::new();

    log::info!("Start to build individual certificates");
    for cocoon_request in cocoon_requests {
        let (certificate, map_pair) =
            build_individual_certificate(exp_type, certificate_type, cocoon_request, db).await?;
        certificates.push(certificate);
        lv_to_plv_maps.extend(map_pair);
    }

    Ok((certificates, lv_to_plv_maps))
}

pub async fn build_non_butterfly_certificate(
    exp_type: ExpansionType,
    non_butterfly_request: NonButterflyRequest,
    db: &DatabaseConnection,
) -> Result<Certificate, GenCertificateError> {
    log::debug!("Calculating validity");
    let certificate_validity_begin = calculate_certificate_validity_begin(0);
    let certificate_validity_duration = calculate_certificate_validity_duration();

    let req_id = 0;

    let aca_private_key = latest_aca_private_key(db)
        .await
        .map_err(|_| GenCertificateError::new("Error fetching latest ACA private key"))?;

    let aca_certificate = latest_aca_certificate(db)
        .await
        .map_err(|_| GenCertificateError::new("Error fetching latest ACA certificate chain"))?;

    let verification_key = non_butterfly_request.get_verifying_key_raw();

    let encryption_key = non_butterfly_request.get_encryption_key_raw();
    let scms_type = non_butterfly_request.get_exp_type();
    let certificate_type = non_butterfly_request.get_certificate_type();

    let certificate_binary = oscms_bridge::issue_authorization_certificate(
        aca_private_key,
        aca_certificate,
        verification_key,
        encryption_key,
        None,
        scms_type.to_owned(),
        certificate_validity_begin,
        certificate_validity_duration,
        certificate_type,
        [0u8; 3],
        0,
    )?;

    log::debug!("Certificate generated successfully");
    let certificate = Certificate::new(
        exp_type,
        req_id,
        certificate_binary,
        certificate_validity_begin,
    );

    Ok(certificate)
}

async fn build_individual_certificate(
    exp_type: ExpansionType,
    certificate_type: CertificateType,
    cocoon_request: CocoonRequest,
    db: &DatabaseConnection,
) -> Result<(Certificate, Vec<LvToPlvMap>), GenCertificateError> {
    let eplv_pair = cocoon_request.eplv_pair;
    let plv_pair = decrypt_eplv_pair(eplv_pair, db).await?;
    let i_index = plv_pair[0].i_index;
    let plvs: Vec<[u8; 16]> = plv_pair.iter().map(|x| x.value).collect();
    let lv = derive_lv(plvs.clone());

    let mut lv_to_plv_maps: Vec<LvToPlvMap> = vec![];
    for plv in plvs {
        lv_to_plv_maps.push(LvToPlvMap::new(lv, plv, i_index));
    }

    log::debug!(
        "Generating individual certificate for req_id: {}",
        cocoon_request.req_id
    );

    let i_index = cocoon_request.i_index;
    let pub_key_encrypt = cocoon_request.pub_key_encrypt;
    let pub_key_sign = cocoon_request.pub_key_sign;
    let private_key_info = cocoon_request.private_key_info;
    let req_id = cocoon_request.req_id;

    let aca_private_key = latest_aca_private_key(db)
        .await
        .map_err(|_| GenCertificateError::new("Error fetching latest ACA private key"))?;

    let aca_certificate = latest_aca_certificate(db)
        .await
        .map_err(|_| GenCertificateError::new("Error fetching latest ACA certificate chain"))?;

    // In the case of UBK there is only one key, the E. This is going to be used for both signing and encryption.
    // In the case of OBK we have two distinct keys, E and S.
    log::debug!("Converting Verifying Key to vec 64");
    let (param_e, param_s) = convert_verifying_key_into_bytes_64(pub_key_encrypt, pub_key_sign)?;
    let scms_type = exp_type;

    log::debug!("Calculating validity");
    let certificate_validity_begin = calculate_certificate_validity_begin(i_index);
    let certificate_validity_duration = calculate_certificate_validity_duration();

    // Calling the lib1609 function
    log::debug!(
        "Calling issue_authorization_certificate: scms_type {}. certificate_validity_begin {}, certificate_validity_duration {}, certificate_type {}.",
        scms_type,
        certificate_validity_begin,
        certificate_validity_duration,
        certificate_type
    );
    let certificate_binary = oscms_bridge::issue_authorization_certificate(
        aca_private_key,
        aca_certificate,
        param_s.to_vec(),
        param_e.to_vec(),
        private_key_info,
        scms_type,
        certificate_validity_begin,
        certificate_validity_duration,
        certificate_type,
        [0u8; 3],
        0,
    )?;

    log::debug!("Certificate generated successfully");
    let certificate = Certificate::new(
        exp_type,
        req_id,
        certificate_binary,
        certificate_validity_begin,
    );
    Ok((certificate, lv_to_plv_maps))
}

/// capacity of 16 bits | duration: 7 years | measured in hours| i-value is 7 days, this value needs to be set in hours
fn calculate_certificate_validity_duration() -> u16 {
    // Define the number of days
    let days: u16 = GlobalConfig::global().period_length_days;

    // Calculate the number of hours (days * hours per day)
    let hours: u16 = days * 24;

    hours
}

/// capacity of 32 bits | measured in seconds, since 2004-01-01: 00:00.00000 UTC| the i-value is measured as 1 day, this values needs to be set in seconds
fn calculate_certificate_validity_begin(i_index: u64) -> u32 {
    // Calculate the validity begin time in seconds (i_index days * seconds per day)
    let seconds: u64 = i_index * 24 * 60 * 60;

    // Convert the result to u32, handling potential overflow
    // Using `u32::MAX` ensures that any overflow is capped at the maximum value of u32
    seconds.min(u32::MAX as u64) as u32
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::aca_certificates::Model as AcaCertificatesModel;
    use crate::test_helper::setup;
    use chrono::Utc;
    use p256::ecdsa::{SigningKey, VerifyingKey};
    use rand::rngs::OsRng;
    use scmscommon::core_types::pre_linkage_value::Eplv;
    use sea_orm::{DatabaseBackend, DatabaseConnection, MockDatabase};

    fn mock_db() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([
                [AcaCertificatesModel {
                    id: 1,
                    name: "aca_private_key".to_string(),
                    insert_date: Utc::now().naive_utc(),
                    file: vec![
                        0x98, 0xF1, 0xC8, 0xE3, 0xED, 0xA3, 0xD3, 0xF8, 0xE7, 0xFC, 0x6E, 0x71,
                        0x2E, 0x87, 0x0A, 0xAE, 0xAC, 0x79, 0x5C, 0xA0, 0x10, 0x0A, 0xB6, 0x00,
                        0xD8, 0x7A, 0x84, 0x5D, 0x62, 0x5A, 0xC2, 0xDD,
                    ],
                }],
                [AcaCertificatesModel {
                    id: 2,
                    name: "aca_public_uncompressed".to_string(),
                    insert_date: Utc::now().naive_utc(),
                    file: vec![
                        0x2B, 0xEC, 0x66, 0x08, 0x0C, 0x3B, 0xC2, 0x59, 0xE5, 0xE9, 0x64, 0xE3,
                        0xA2, 0x0E, 0x41, 0x07, 0x20, 0x05, 0x25, 0xB9, 0xE9, 0x1B, 0xFE, 0xBF,
                        0x34, 0x07, 0x15, 0x61, 0x8E, 0xD5, 0x0D, 0x4F, 0x72, 0x35, 0xF2, 0xEB,
                        0x93, 0xB9, 0x10, 0x22, 0x05, 0xE6, 0xFA, 0x8E, 0xF5, 0xFA, 0x9C, 0x07,
                        0xAA, 0x6B, 0x83, 0x68, 0x21, 0x38, 0x49, 0x34, 0xB8, 0x5B, 0x59, 0xF8,
                        0xE5, 0x78, 0x31, 0xAF,
                    ],
                }],
                [AcaCertificatesModel {
                    id: 3,
                    name: "aca_certificate".to_string(),
                    insert_date: Utc::now().naive_utc(),
                    file: vec![
                        0x80, 0x03, 0x00, 0x80, 0xBA, 0xC7, 0x00, 0x87, 0xDC, 0x64, 0xF2, 0x90,
                        0x00, 0x81, 0x0F, 0x61, 0x63, 0x61, 0x2E, 0x6C, 0x67, 0x73, 0x63, 0x6D,
                        0x73, 0x2E, 0x74, 0x65, 0x73, 0x74, 0x01, 0x02, 0x03, 0x00, 0x00, 0x26,
                        0xF7, 0x28, 0x3C, 0x84, 0x00, 0x0A, 0x80, 0x80, 0x83, 0x2B, 0xEC, 0x66,
                        0x08, 0x0C, 0x3B, 0xC2, 0x59, 0xE5, 0xE9, 0x64, 0xE3, 0xA2, 0x0E, 0x41,
                        0x07, 0x20, 0x05, 0x25, 0xB9, 0xE9, 0x1B, 0xFE, 0xBF, 0x34, 0x07, 0x15,
                        0x61, 0x8E, 0xD5, 0x0D, 0x4F, 0x80, 0x80, 0xDD, 0x00, 0x9B, 0x18, 0xF0,
                        0x3D, 0x62, 0xC9, 0x70, 0x05, 0xCD, 0x96, 0xED, 0x00, 0xD0, 0x3D, 0x55,
                        0x34, 0xAA, 0x9E, 0xA3, 0x09, 0x10, 0xAA, 0x3C, 0xA9, 0x72, 0x56, 0x02,
                        0xC6, 0xBF, 0x94, 0xD9, 0x55, 0xAB, 0x3B, 0xE8, 0x1E, 0x3F, 0x11, 0xC4,
                        0xA1, 0x04, 0x25, 0x5B, 0x9E, 0x1D, 0xB5, 0x64, 0xF6, 0xDF, 0x24, 0x4A,
                        0x9F, 0x83, 0x72, 0x8F, 0x6B, 0xCC, 0x74, 0x6F, 0xF8, 0x8B, 0xC3,
                    ],
                }],
                [AcaCertificatesModel {
                    id: 4,
                    name: "aca_ca_cert".to_string(),
                    insert_date: Utc::now().naive_utc(),
                    file: vec![
                        0x80, 0x03, 0x00, 0x80, 0x31, 0x0B, 0xF2, 0xFD, 0xE7, 0xFE, 0x81, 0xC1,
                        0x00, 0x82, 0x10, 0xD7, 0xBC, 0x90, 0xB9, 0x57, 0x9E, 0x98, 0xED, 0x68,
                        0x34, 0x83, 0xE3, 0x3B, 0xF6, 0xFA, 0x24, 0x01, 0x02, 0x03, 0x00, 0x00,
                        0x26, 0xD2, 0x73, 0xD0, 0x84, 0x00, 0x18, 0x80, 0x80, 0x84, 0x03, 0xA6,
                        0x56, 0xE1, 0x30, 0x13, 0xCE, 0xE0, 0x82, 0x46, 0x5B, 0xD5, 0x52, 0x97,
                        0x83, 0x33, 0x96, 0x29, 0xFC, 0x91, 0x8E, 0x3F, 0x56, 0xF2, 0xA2, 0x35,
                        0x64, 0x55, 0x84, 0x8F, 0x40, 0xE3, 0xF8, 0x10, 0xC5, 0xB1, 0x2A, 0x7E,
                        0x8B, 0x90, 0x63, 0xDC, 0x37, 0x38, 0xC2, 0xFE, 0x9D, 0x67, 0x67, 0x76,
                        0x55, 0x32, 0xB1, 0x6E, 0xAD, 0x10, 0x64, 0x6E, 0x2D, 0xF1, 0xE3, 0x4F,
                        0x6E, 0x7C, 0x80, 0x80, 0xE5, 0xD2, 0x63, 0xEE, 0x90, 0x1B, 0x81, 0x79,
                        0xC9, 0x84, 0xC9, 0x1C, 0xB4, 0x9C, 0x55, 0x29, 0x59, 0x59, 0xDF, 0x9A,
                        0x62, 0x96, 0x2F, 0x3F, 0x4B, 0x91, 0x00, 0x2B, 0x7E, 0x10, 0x4E, 0x96,
                        0xDD, 0xB9, 0xFF, 0xE3, 0x0C, 0x0F, 0xB3, 0x19, 0x6B, 0xC0, 0x23, 0x29,
                        0x8A, 0x6B, 0x70, 0x74, 0x3F, 0xAC, 0xA7, 0x3D, 0x1B, 0x11, 0x90, 0xA2,
                        0x5B, 0x0B, 0x78, 0xA4, 0x77, 0x16, 0x4C, 0x8F,
                    ],
                }],
                [AcaCertificatesModel {
                    id: 5,
                    name: "root_ca_cert".to_string(),
                    insert_date: Utc::now().naive_utc(),
                    file: vec![
                        0x80, 0x03, 0x00, 0x81, 0x00, 0x00, 0x82, 0x10, 0x1B, 0x58, 0x98, 0xFD,
                        0x46, 0x4B, 0x83, 0x66, 0xB3, 0x9E, 0x88, 0xC9, 0x33, 0x5E, 0xC5, 0x4D,
                        0x01, 0x02, 0x03, 0x00, 0x00, 0x26, 0xD2, 0x73, 0x3A, 0x84, 0x00, 0x18,
                        0x80, 0x80, 0x84, 0xE8, 0x3B, 0xAE, 0x6A, 0x83, 0xC4, 0xF8, 0x60, 0xFC,
                        0xFC, 0xDC, 0xF5, 0xAE, 0x6A, 0x0B, 0xF4, 0x09, 0x52, 0xDA, 0x4C, 0x89,
                        0xA8, 0xB2, 0xDC, 0x70, 0x05, 0x13, 0x06, 0x06, 0xAB, 0x10, 0xB3, 0x62,
                        0xC2, 0x64, 0x49, 0x86, 0xEE, 0x7A, 0x7F, 0x6B, 0x53, 0xA8, 0x20, 0x24,
                        0x1E, 0x37, 0xAB, 0x0D, 0x9C, 0x09, 0xE8, 0x7C, 0xCE, 0x4B, 0x17, 0xCD,
                        0x5B, 0x59, 0x7A, 0x83, 0x12, 0x00, 0x0B, 0x80, 0x80, 0xEF, 0x5E, 0x5E,
                        0xD5, 0x7F, 0x8E, 0xB4, 0x35, 0x89, 0x54, 0x04, 0xBA, 0x45, 0xC4, 0x44,
                        0xDC, 0x23, 0x89, 0x5D, 0x87, 0x94, 0x77, 0xB8, 0xDB, 0x32, 0x40, 0x91,
                        0xF2, 0xED, 0x34, 0xE3, 0x6D, 0xE4, 0xC1, 0x5E, 0xD0, 0xF4, 0xD7, 0x31,
                        0x6C, 0x02, 0xA8, 0x36, 0x6D, 0xF3, 0xC7, 0x68, 0xE0, 0xBD, 0x03, 0x23,
                        0x8E, 0x3C, 0xC8, 0x73, 0xCB, 0xF2, 0xAB, 0xE9, 0xE5, 0x70, 0xF2, 0x8D,
                        0xED,
                    ],
                }],
            ])
            .into_connection()
    }

    #[test]
    fn test_build_individual_certificate() {
        setup();
        let req_id: u64 = 1;
        let signing_key = SigningKey::random(&mut OsRng);
        let private_key_info = Some(signing_key.to_bytes().to_vec());
        let verifying_key = VerifyingKey::from(&signing_key);
        let pub_key_sign = Some(verifying_key);
        let pub_key_encrypt = Some(verifying_key);

        let enc_value = [1u8; 32];
        let ephemeral_pub_key = vec![2u8; 32];
        let nonce = [5u8; 12];
        let i_index = 123;
        let j_index = 456;

        let eplv = Eplv::new(
            enc_value,
            ephemeral_pub_key.clone(),
            nonce,
            i_index,
            j_index,
        );

        let eplv_pair = vec![eplv.clone(), eplv.clone()];
        let i_index: u64 = 10;

        // Create a new CocoonRequest using the new method
        let cocoon_request = CocoonRequest::new(
            req_id,
            pub_key_sign,
            pub_key_encrypt,
            private_key_info,
            eplv_pair.clone(),
            i_index,
        );

        let db = mock_db();
        let certificate_type = CertificateType::Explicit;
        std::mem::drop(build_individual_certificate(
            ExpansionType::Compact,
            certificate_type,
            cocoon_request,
            &db,
        ));
    }
}
