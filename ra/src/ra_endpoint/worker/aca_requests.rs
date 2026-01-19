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

use crate::core_logic::cocoon_client_mapping::generate_cocoon_request;
use crate::persistence::certificate_store::store_certificate;
use crate::persistence::x_dot_info_store::store_x_dot_info;
use crate::ra_endpoint::worker::common::post_ra_to_aca;
use scmscommon::{
    Certificate, GlobalConfig,
    core_types::{
        Caterpillar, ClientRequestsMapping, ExpansionType, NonButterflyRequest, PayloadLaToRa,
        PayloadRaToAca,
    },
    errors,
};
use sea_orm::DatabaseConnection;

// include in the parameters of this function: caterpillars, payloads_la_to_ra, payload_cam_to_ra
// Maybe the store of the certificates should happen outside of the handle_aca_request
pub async fn handle_aca_request(
    caterpillars: Vec<Caterpillar>,
    payloads_la_to_ra: Vec<PayloadLaToRa>,
    non_butterfly_request: Option<NonButterflyRequest>,
    exp_type: ExpansionType,
    db: &DatabaseConnection,
) -> Result<(), errors::ScmsInternalCommError> {
    log::info!("Handling ACA request");
    let (payload_ra_to_aca, requests_mapping) = generate_payload_ra_to_aca(
        exp_type,
        caterpillars,
        payloads_la_to_ra,
        non_butterfly_request.clone(),
    )?;

    // Post to ACA
    let certificates = post_to_aca(payload_ra_to_aca).await?;

    if exp_type != ExpansionType::NonButterfly && exp_type != ExpansionType::NonButterflyEncrypted {
        // Unshuffling certificates from ACA based on mapping stored before ACA request
        log::debug!("Unshuffling certificares from ACA");
        unshuffle_and_store_certificates(requests_mapping.clone(), certificates, db).await?;

        // Generate and store x.info files
        log::debug!("Generating and storing x.info files");
        genenerate_and_store_x_dot_info(requests_mapping, db).await?;
    } else if non_butterfly_request.is_some() {
        log::debug!("Storing certificate for NonButterflyRequest");

        let certificate = certificates.first().unwrap();
        let non_butterfly_request_instance = non_butterfly_request.unwrap();
        store_certificate(
            db,
            non_butterfly_request_instance.get_hash_id().clone(),
            0,
            0,
            certificate.encoded_binary.clone(),
        )
        .await
        .map_err(|e| {
            errors::ScmsInternalCommError::new(
                format!("Failed to store certificate: {}", e).as_str(),
                errors::InternalCommWire::RaToAca,
                500,
            )
        })?;
    } else {
        return Err(errors::ScmsInternalCommError::new(
            "NonButterflyRequest is None",
            errors::InternalCommWire::RaToAca,
            500,
        ));
    }

    log::info!("Finishing ACA request handler.");
    Ok(())
}

async fn post_to_aca(
    payload_ra_to_aca: PayloadRaToAca,
) -> Result<Vec<Certificate>, errors::ScmsInternalCommError> {
    log::debug!("Sending post request to ACA");

    // Construct request address
    let aca_addr = GlobalConfig::global().aca_addr();
    let aca_port = GlobalConfig::global().aca_port;
    let aca_post_endpoint = &GlobalConfig::global().aca_post_endpoint;
    let req_addr = format!("{}:{}{}", aca_addr, aca_port, aca_post_endpoint);
    let response_result = post_ra_to_aca(req_addr, payload_ra_to_aca).await;

    match response_result {
        Ok(response) => {
            log::debug!("Received response from ACA");
            // Capture PayloadLaToRa
            if response.status().is_success() {
                let resp_body = response.text().await?;
                let certificates: Vec<Certificate> = serde_json::from_str(&resp_body)?;
                log::debug!("ACA response is ok, returning payload!");
                return Ok(certificates);
            }

            log::debug!("No success response from ACA.");
            let error_code = response.status().as_u16();
            Err(errors::ScmsInternalCommError::new(
                format!(
                    "Received response from ACA with error status code: {}",
                    error_code
                )
                .as_str(),
                errors::InternalCommWire::RaToAca,
                error_code,
            ))
        }
        Err(e) => Err(errors::ScmsInternalCommError::new(
            format!("Failed to Communicate with ACA', status code: {}", e).as_str(),
            errors::InternalCommWire::RaToAca,
            500,
        )),
    }
}

fn generate_payload_ra_to_aca(
    exp_type: ExpansionType,
    caterpillars: Vec<Caterpillar>,
    payloads_la_to_ra: Vec<PayloadLaToRa>,
    non_butterfly_request: Option<NonButterflyRequest>,
) -> Result<(PayloadRaToAca, Vec<ClientRequestsMapping>), errors::ScmsInternalCommError> {
    // Generate cocoon request based on expansion type
    log::info!("Starting generate_payload_ra_to_aca");
    let mut cocoon_request_list = Vec::new();
    let mut cocoon_request_mapping = Vec::new();

    let certificate_type;

    if exp_type != ExpansionType::NonButterfly && exp_type != ExpansionType::NonButterflyEncrypted {
        log::debug!("Generating cocoon request for caterpillars");
        (cocoon_request_list, cocoon_request_mapping) =
            generate_cocoon_request(exp_type, caterpillars.clone(), payloads_la_to_ra)?;

        // Sort cocoon request based on request id
        cocoon_request_list.sort_by(|a, b| a.req_id.cmp(&b.req_id));

        // Certificate Type will be the same for all cocoon
        if let Some(first_caterpillar) = caterpillars.first() {
            certificate_type = first_caterpillar.get_certificate_type();
        } else {
            return Err(errors::ScmsInternalCommError::new(
                "Caterpillars list is empty",
                errors::InternalCommWire::RaToAca,
                500,
            ));
        }
    } else {
        log::debug!("Generating cocoon request for NonButterflyRequest");

        let non_butterfly_request = non_butterfly_request.as_ref().ok_or_else(|| {
            errors::ScmsInternalCommError::new(
                "NonButterflyRequest is None",
                errors::InternalCommWire::RaToAca,
                500,
            )
        })?;

        log::debug!(
            "Generating cocoon request for NonButterflyRequest {:?}",
            non_butterfly_request
        );

        certificate_type = non_butterfly_request.certificate_type;
    }

    // Create PayloadRaToAca
    Ok((
        PayloadRaToAca::new(
            exp_type,
            1,
            certificate_type,
            cocoon_request_list,
            non_butterfly_request,
        ),
        cocoon_request_mapping,
    ))
}

async fn unshuffle_and_store_certificates(
    client_mapping: Vec<ClientRequestsMapping>,
    certificates: Vec<Certificate>,
    db: &DatabaseConnection,
) -> Result<(), errors::ScmsInternalCommError> {
    log::info!("Starting unshuffling certificates from ACA");

    for client in client_mapping.iter() {
        let req_id = client.req_id;

        let certificates_filtered: Vec<&Certificate> = certificates
            .iter()
            .filter(|certificate| req_id == certificate.req_id)
            .collect();

        for (certificate, req_entry) in certificates_filtered.iter().zip(client.requests.iter()) {
            log::debug!(
                "Unshuffling and storing certificate for client {} with i: {} and j: {}, certificate {:?}",
                client.vid,
                req_entry.i_index,
                req_entry.j_index,
                certificate.encoded_binary,
            );
            // Using database
            store_certificate(
                db,
                client.hash_id.clone(),
                req_entry.i_index,
                req_entry.j_index,
                certificate.encoded_binary.clone(),
            )
            .await
            .map_err(|e| {
                errors::ScmsInternalCommError::new(
                    format!("Failed to store certificate: {}", e).as_str(),
                    errors::InternalCommWire::RaToAca,
                    500,
                )
            })?;
        }

        log::debug!(
            "Unshuffled and stored {} certificates for client {}",
            certificates_filtered.len(),
            client.vid
        );
    }

    log::debug!("Unshuffled and stored all certificates from ACA.");
    Ok(())
}

// This method is used to carry info about the stream of ASN bytes of the
// encoded x.info file.
//
// This file is specified in the IEEE 1609.2.1 standard.
// Make a search with the word "x.info", you'll find more information about it.
//
// "The first file in the root directory is the .info file with name x.info,
// where x is the minimal length hex encoding of the i-value for this
// certificate batch. The contents of the .info file is a C-OER encoded
// RaEeCertInfoSpdu if the file does not contain ACPC information, and a C-OER
// encoded RaEeCertAndAcpcInfoSpdu if the file contains ACPC information
// (specifically, an AcpcTreeId)."
async fn genenerate_and_store_x_dot_info(
    requests_mapping: Vec<ClientRequestsMapping>,
    db: &DatabaseConnection,
) -> Result<(), errors::ScmsInternalCommError> {
    log::info!("Generating and storing x.info files");

    for client in requests_mapping.iter() {
        // All req_entry.i from the client.request are the same so we can take
        // just the first one.
        let request = client.requests.first().unwrap();
        let current_i = request.i_index;
        let hash_id = &client.hash_id;

        // hash_id = "83ddacb72a977c2f" an example of a hash_id
        if hash_id.len() != 16 {
            return Err(errors::ScmsInternalCommError::new(
                "Invalid hash_id length",
                errors::InternalCommWire::RaToAca,
                500,
            ));
        }

        let mut request_hash: [u8; 8] = [0; 8];
        for i in (0..hash_id.len()).step_by(2) {
            let hex_str = &hash_id[i..i + 2];
            let hex_int = u8::from_str_radix(hex_str, 16).expect("Invalid hex string");
            request_hash[i / 2] = hex_int;
        }

        let encoded_response =
            match oscms_bridge::make_certificate_info_spdu(current_i as u16, request_hash) {
                Ok(encoded) => encoded,
                Err(_) => {
                    return Err(errors::ScmsInternalCommError::new(
                        "Failed to generate encoded response",
                        errors::InternalCommWire::RaToAca,
                        500,
                    ));
                }
            };

        store_x_dot_info(db, hash_id.clone(), current_i, encoded_response)
            .await
            .map_err(|e| {
                errors::ScmsInternalCommError::new(
                    format!("Failed to store certificate: {}", e).as_str(),
                    errors::InternalCommWire::RaToAca,
                    500,
                )
            })?;

        log::debug!("Generated x.info files for vid: {}", client.vid);
    }

    log::debug!("Generated all x.info files.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::certificate_store::Model as CertificateStoreModel;
    use crate::entities::x_dot_info_store::Model as XDotInfoModel;
    use crate::test_helper::ra_config_mock::setup_ra_conf;
    use crate::test_helper::{aca_server_mock, caterpillar_mock};
    use scmscommon::core_types::{
        Certificate, ClientRequestsMapping, ClientRequestsMappingEntry, ExpansionType,
    };
    use scmscommon::plv_payload::PlvPayload;
    use scmscommon::pre_linkage_value::Eplv;
    use scmscommon::setup_global_config;
    use sea_orm::{DatabaseBackend, MockDatabase, MockExecResult};

    fn setup() {
        setup_global_config();
        setup_ra_conf();
    }

    fn mock_db() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([
                [CertificateStoreModel {
                    id: 1,
                    hash_id: "81ddacb72a977c2f".to_string(),
                    index_i: 3,
                    index_j: 3,
                    file: vec![3; 16],
                    downloaded: 0,
                }],
                [CertificateStoreModel {
                    id: 2,
                    hash_id: "81ddacb72a977c2f".to_string(),
                    index_i: 4,
                    index_j: 4,
                    file: vec![4; 16],
                    downloaded: 0,
                }],
                [CertificateStoreModel {
                    id: 3,
                    hash_id: "82ddacb72a977c2f".to_string(),
                    index_i: 5,
                    index_j: 5,
                    file: vec![5; 16],
                    downloaded: 0,
                }],
                [CertificateStoreModel {
                    id: 4,
                    hash_id: "82ddacb72a977c2f".to_string(),
                    index_i: 6,
                    index_j: 6,
                    file: vec![6; 16],
                    downloaded: 0,
                }],
                [CertificateStoreModel {
                    id: 5,
                    hash_id: "83ddacb72a977c2f".to_string(),
                    index_i: 1,
                    index_j: 1,
                    file: vec![1; 16],
                    downloaded: 0,
                }],
                [CertificateStoreModel {
                    id: 6,
                    hash_id: "83ddacb72a977c2f".to_string(),
                    index_i: 2,
                    index_j: 2,
                    file: vec![2; 16],
                    downloaded: 0,
                }],
            ])
            .append_query_results([
                // For x.info
                [XDotInfoModel {
                    id: 1,
                    hash_id: "81ddacb72a977c2f".to_string(),
                    current_i: 3,
                    file: vec![],
                    downloaded: 0,
                }],
                [XDotInfoModel {
                    id: 2,
                    hash_id: "82ddacb72a977c2f".to_string(),
                    current_i: 5,
                    file: vec![],
                    downloaded: 0,
                }],
                [XDotInfoModel {
                    id: 3,
                    hash_id: "83ddacb72a977c2f".to_string(),
                    current_i: 1,
                    file: vec![],
                    downloaded: 0,
                }],
            ])
            .append_exec_results([
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 2,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 3,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 4,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 5,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 6,
                    rows_affected: 1,
                },
            ])
            .append_exec_results([
                // For x.files
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 2,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 3,
                    rows_affected: 1,
                },
            ])
            .into_connection()
    }

    fn mock_db2() -> DatabaseConnection {
        MockDatabase::new(DatabaseBackend::MySql)
            .append_query_results([
                [CertificateStoreModel {
                    id: 1,
                    hash_id: "c65fc8b49de32b6a".to_string(),
                    index_i: 3,
                    index_j: 3,
                    file: vec![3; 16],
                    downloaded: 0,
                }],
                [CertificateStoreModel {
                    id: 2,
                    hash_id: "35320f14a4bc2f0f".to_string(),
                    index_i: 4,
                    index_j: 4,
                    file: vec![4; 16],
                    downloaded: 0,
                }],
            ])
            .append_query_results([
                // For x.info
                [XDotInfoModel {
                    id: 1,
                    hash_id: "c65fc8b49de32b6a".to_string(),
                    current_i: 3,
                    file: vec![],
                    downloaded: 0,
                }],
                [XDotInfoModel {
                    id: 2,
                    hash_id: "35320f14a4bc2f0f".to_string(),
                    current_i: 5,
                    file: vec![],
                    downloaded: 0,
                }],
            ])
            .append_exec_results([
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 2,
                    rows_affected: 1,
                },
            ])
            .append_exec_results([
                // For x.files
                MockExecResult {
                    last_insert_id: 1,
                    rows_affected: 1,
                },
                MockExecResult {
                    last_insert_id: 2,
                    rows_affected: 1,
                },
            ])
            .into_connection()
    }

    #[tokio::test]
    async fn test_all_aca_handler() {
        setup();

        let db = mock_db();
        test_unshuffle_and_store_certificates(&db).await;
        test_genenerate_and_store_x_dot_info(&db).await;
        test_handle_aca_request().await;
    }

    async fn test_unshuffle_and_store_certificates(db: &DatabaseConnection) {
        let client_mapping: Vec<ClientRequestsMapping> = vec![
            ClientRequestsMapping {
                vid: 1,
                req_id: 1,
                exp_type: ExpansionType::Original,
                hash_id: "81ddacb72a977c2f".to_string(),
                requests: vec![
                    ClientRequestsMappingEntry {
                        i_index: 3,
                        j_index: 3,
                    },
                    ClientRequestsMappingEntry {
                        i_index: 4,
                        j_index: 4,
                    },
                ],
            },
            ClientRequestsMapping {
                vid: 2,
                req_id: 2,
                exp_type: ExpansionType::Original,
                hash_id: "82ddacb72a977c2f".to_string(),
                requests: vec![
                    ClientRequestsMappingEntry {
                        i_index: 5,
                        j_index: 5,
                    },
                    ClientRequestsMappingEntry {
                        i_index: 6,
                        j_index: 6,
                    },
                ],
            },
            ClientRequestsMapping {
                vid: 3,
                req_id: 3,
                exp_type: ExpansionType::Original,
                hash_id: "83ddacb72a977c2f".to_string(),
                requests: vec![
                    ClientRequestsMappingEntry {
                        i_index: 1,
                        j_index: 1,
                    },
                    ClientRequestsMappingEntry {
                        i_index: 2,
                        j_index: 2,
                    },
                ],
            },
        ];

        let certificates = vec![
            Certificate {
                req_id: 1,
                exp_type: ExpansionType::Original,
                certificate_validity_begin: 1,
                encoded_binary: vec![1, 2, 3],
            },
            Certificate {
                req_id: 1,
                exp_type: ExpansionType::Original,
                certificate_validity_begin: 1,
                encoded_binary: vec![4, 5, 6],
            },
            Certificate {
                req_id: 2,
                exp_type: ExpansionType::Original,
                certificate_validity_begin: 1,
                encoded_binary: vec![7, 8, 9],
            },
            Certificate {
                req_id: 2,
                exp_type: ExpansionType::Original,
                certificate_validity_begin: 1,
                encoded_binary: vec![10, 11, 12],
            },
            Certificate {
                req_id: 3,
                exp_type: ExpansionType::Original,
                certificate_validity_begin: 1,
                encoded_binary: vec![13, 14, 15],
            },
            Certificate {
                req_id: 3,
                exp_type: ExpansionType::Original,
                certificate_validity_begin: 1,
                encoded_binary: vec![16, 17, 18],
            },
        ];

        let result =
            unshuffle_and_store_certificates(client_mapping.clone(), certificates.clone(), db)
                .await
                .map_err(|e| {
                    panic!("Failed unshuffle_and_store_certificates: {:#?}", e);
                });
        assert!(result.is_ok());
    }

    async fn test_genenerate_and_store_x_dot_info(db: &DatabaseConnection) {
        let client_mapping: Vec<ClientRequestsMapping> = vec![
            ClientRequestsMapping {
                vid: 1,
                req_id: 1,
                exp_type: ExpansionType::Original,
                hash_id: "81ddacb72a977c2f".to_string(),
                requests: vec![
                    ClientRequestsMappingEntry {
                        i_index: 3,
                        j_index: 3,
                    },
                    ClientRequestsMappingEntry {
                        i_index: 4,
                        j_index: 4,
                    },
                ],
            },
            ClientRequestsMapping {
                vid: 2,
                req_id: 2,
                exp_type: ExpansionType::Original,
                hash_id: "82ddacb72a977c2f".to_string(),
                requests: vec![
                    ClientRequestsMappingEntry {
                        i_index: 5,
                        j_index: 5,
                    },
                    ClientRequestsMappingEntry {
                        i_index: 6,
                        j_index: 6,
                    },
                ],
            },
            ClientRequestsMapping {
                vid: 3,
                req_id: 3,
                exp_type: ExpansionType::Original,
                hash_id: "83ddacb72a977c2f".to_string(),
                requests: vec![
                    ClientRequestsMappingEntry {
                        i_index: 1,
                        j_index: 1,
                    },
                    ClientRequestsMappingEntry {
                        i_index: 2,
                        j_index: 2,
                    },
                ],
            },
        ];

        let result = genenerate_and_store_x_dot_info(client_mapping.clone(), db)
            .await
            .map_err(|e| {
                panic!("Failed genenerate_and_store_x_dot_info: {:#?}", e);
            });
        assert!(result.is_ok());
    }

    async fn test_handle_aca_request() {
        let (_setup, _expected_responses) = match aca_server_mock::setup_aca_mock_servers().await {
            Ok((setup, _expected_responses)) => (setup, _expected_responses),
            Err(e) => {
                panic!("Failed to setup ACA mock servers: {:#?}", e);
            }
        };

        let caterpillars = caterpillar_mock::mock_caterpillars();

        // Create a new PayloadLaToRa instance using the new method
        let payloads_la_to_ra = vec![PayloadLaToRa::new(
            "sample_request_id".to_owned(),
            vec![
                PlvPayload {
                    req_id: 0,
                    eplvs: vec![Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1)],
                },
                PlvPayload {
                    req_id: 1,
                    eplvs: vec![Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1)],
                },
            ],
        )];

        let db: DatabaseConnection = mock_db2();
        let result = handle_aca_request(
            caterpillars,
            payloads_la_to_ra,
            None,
            ExpansionType::Original,
            &db,
        )
        .await
        .map_err(|e| {
            panic!("Failed unshuffle_and_store_certificates: {:#?}", e);
        });
        assert!(result.is_ok());
    }
}
