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

use crate::core_logic::create_cocoon_pub_key;
use scmscommon::ScmsInternalCommError;
use scmscommon::core_types::CocoonRequest;
use scmscommon::core_types::pre_linkage_value::Eplv;
use scmscommon::core_types::{
    Caterpillar, ClientRequestsMapping, ClientRequestsMappingEntry, ExpansionType, PayloadLaToRa,
};
use scmscommon::errors;

pub fn generate_cocoon_request(
    exp_type: ExpansionType,
    caterpillars: Vec<Caterpillar>,
    payloads_la_to_ra: Vec<PayloadLaToRa>,
) -> Result<(Vec<CocoonRequest>, Vec<ClientRequestsMapping>), errors::ScmsInternalCommError> {
    log::info!("Start generate_cocoon_request");
    let list_caterpillar_eplv = associate_caterpillar_with_eplv(caterpillars, payloads_la_to_ra);
    let (cocoon_request_list, client_requests_mapping_list) =
        generate_cocoon_request_and_client_mapping(exp_type, list_caterpillar_eplv)?;

    Ok((cocoon_request_list, client_requests_mapping_list))
}

/// There is one CocoonRequest for a Certificate. If we have one caterpillar asking
/// 10 certificates for 4 periods then we have 40 certificates.
/// Hence 40 CocoonRequest.
/// This function returns a list of CocoonRequest.
fn generate_cocoon_request_and_client_mapping(
    _exp_type: ExpansionType,
    list_caterpillar_eplv: Vec<CaterpillarEplvs>,
) -> Result<(Vec<CocoonRequest>, Vec<ClientRequestsMapping>), errors::ScmsInternalCommError> {
    let mut cocoon_requests: Vec<CocoonRequest> = Vec::new();
    let mut client_requests_mapping_list: Vec<ClientRequestsMapping> = Vec::new();
    let mut requests: Vec<ClientRequestsMappingEntry> = Vec::new();

    for caterpillar_eplv in list_caterpillar_eplv {
        // In the ClientMapping we use the hash_id to know which certificate belongs to which client.
        // The EE will use the hash_id so that we can retrieve the correct zip file.
        let hash_id = caterpillar_eplv.caterpillar.get_hash_id();
        let f_encrypt = caterpillar_eplv.caterpillar.f_encrypt();
        let f_sign = caterpillar_eplv.caterpillar.f_sign();
        let vid = caterpillar_eplv.caterpillar.vid();
        let exp_type = caterpillar_eplv.caterpillar.get_exp_type();
        let caterpillar_pub_key_sign = caterpillar_eplv.caterpillar.pub_key_sign();
        let caterpillar_pub_key_encrypt = caterpillar_eplv.caterpillar.pub_key_encrypt();
        let req_id = caterpillar_eplv.req_id;
        let eplvs_la_matrix = caterpillar_eplv.eplvs_la_matrix;

        // Create a vec[vec[eplv]] to store the values in pairs (in the case of two LAs)
        let eplv_pairs = organize_eplvs_la_matrix_into_eplv_pairs(eplvs_la_matrix);
        for eplv_pair in eplv_pairs {
            log::debug!(
                "generate_cocoon_request_and_client_mapping: eplv_pair: {:?}",
                eplv_pair
            );
            // There is one cocoon_request for each certificate that will be created by the ACA.
            let i_index = eplv_pair[0].i_index; // All the EPLVs in eplv_pair have the same i_index, that is why we get the first one.
            let j_index = eplv_pair[0].j_index;

            log::debug!(
                "generate_cocoon_request_and_client_mapping: i_index, j_index: {:?}, {:?}",
                i_index,
                j_index,
            );

            let result = create_cocoon_pub_key(
                f_encrypt,
                f_sign,
                i_index,
                j_index,
                caterpillar_pub_key_encrypt,
                caterpillar_pub_key_sign,
            );
            if result.is_err() {
                return Err(ScmsInternalCommError::new(
                    format!(
                        "Error in expansion function in create_cocoon_pub_key: {:?}",
                        result.err()
                    )
                    .as_str(),
                    errors::InternalCommWire::RaToAca,
                    500,
                ));
            }
            let (cocoon_pub_key_encrypt, cocoon_pub_key_sign, private_key_info) = result.unwrap();

            let cocoon_request = CocoonRequest::new(
                req_id,
                cocoon_pub_key_sign,
                cocoon_pub_key_encrypt,
                private_key_info,
                eplv_pair,
                i_index,
            );
            let client_entry = ClientRequestsMappingEntry::new(i_index, j_index);
            requests.push(client_entry);
            cocoon_requests.push(cocoon_request);
        }
        let client_mapping =
            ClientRequestsMapping::new(requests.clone(), vid, hash_id, *exp_type, req_id);
        client_requests_mapping_list.push(client_mapping);
    }

    Ok((cocoon_requests, client_requests_mapping_list))
}

/// This struct is used for keeping track which Caterpillar is associated with which
/// req_id and which eplvs.
/// Each Caterpillar has a list of Eplv one from each LA, if there are two LAs
/// then we have two lists of eplvs.
#[derive(Debug)]
struct CaterpillarEplvs {
    pub caterpillar: Caterpillar,
    pub req_id: u64,
    pub eplvs_la_matrix: Vec<Vec<Eplv>>, // The line here is an LA, and it has all the eplvs from that LA which are the columns
}

/// Each Caterpillar has associated with it two lists of EPLVs each from an LA, if we have two LAs.
/// This function is used to combine the `caterpillar` and `payloads_la_to_ra` so we can create a simpler
/// object which contains a caterpillar and associated EPLVs of that caterpillar,
/// in an easy to access form.
fn associate_caterpillar_with_eplv(
    caterpillars: Vec<Caterpillar>,
    payloads_la_to_ra: Vec<PayloadLaToRa>, // In the case of two LAs there are two items in this list
) -> Vec<CaterpillarEplvs> {
    log::debug!(
        "associate_caterpillar_with_eplv: payloads_la_to_ra: {:?}",
        payloads_la_to_ra
    );
    let mut list_caterpillar_eplv: Vec<CaterpillarEplvs> = Vec::new();

    for (req_id, caterpillar) in caterpillars.iter().enumerate() {
        // Loop on the caterpillars, each EE request that arrived
        let mut eplvs_la_matrix: Vec<Vec<Eplv>> = Vec::new();
        let req_id = req_id as u64; // The specific EPLV value that comes from the LA and the caterpillar is not important. We are associating them now in this function. From now on the association created here will be important.
        for payload_la_to_ra in payloads_la_to_ra.clone() {
            // Loop on LA index. In the case of two LAs, this loop will run two times.
            let plv_payloads = payload_la_to_ra.plv_payloads.clone();
            for plv_payload in plv_payloads {
                // Loop on the req_id, the id connecting which caterpillar is associated with.
                if plv_payload.req_id == req_id {
                    eplvs_la_matrix.push(plv_payload.eplvs);
                    // This Matrix has a format of like:
                    // [
                    //    [eplv1, eplv2,...],
                    //    [eplv1, eplv2,...],
                    // ]
                }
            }
        }
        let item = CaterpillarEplvs {
            caterpillar: caterpillar.clone(),
            req_id,
            eplvs_la_matrix,
        };

        list_caterpillar_eplv.push(item);
    }
    // Each caterpillar needs two lists of EPLVs (if there are two LAs).
    // Those lists need to be sent to ACA along side the caterpillar.
    log::debug!(
        "associate_caterpillar_with_eplv: list_caterpillar_eplv: {:?}",
        list_caterpillar_eplv
    );
    list_caterpillar_eplv
}

/// This is an auxiliary function, each CaterpillarEplv has inside of it a field called
/// eplvs_la_matrix, which is a matrix with two lines (in the case of two LAs).
/// Each line has inside of it a `Vec<Eplv>`.
/// One caterpillar is associated with one EE request.
/// If the EE is asking certificates for 4 periods and in each period we have 10 certificates.
/// We'll have to send 40 certificates in total, for each EE request.
/// If we have 2 EE requests we'll send 80 certificates e.g.
/// In the CaterpillarEplv, we have a HashMap as a field which contains 2 keys 0,1 (in the case of 2 LAs)
/// Each key contains a `Vec<Eplv>` as its value, so we have two of those lists.
/// Each list contains 40 Eplvs, so inside each CaterpillarEplv we have 80 Eplvs.
/// Each Caterpillar needs a pair (of two, with 2 LAs), then inside the ACA it will xor those two PLVs.
fn organize_eplvs_la_matrix_into_eplv_pairs(eplvs_la_matrix: Vec<Vec<Eplv>>) -> Vec<Vec<Eplv>> {
    // This Matrix has a format of like:
    // [
    //    [eplv1, eplv2,...],
    //    [eplv1, eplv2,...],
    // ]

    let num_of_la = eplvs_la_matrix.len();
    let len_list = eplvs_la_matrix[0].len();

    // Create a vec[vec[eplv]] to store the values in pairs (in the case of two LAs)
    // Example of
    // eplvs_pairs = [
    //      [eplv1, eplv4],
    //      [eplv2, eplv5],
    //      [eplv3, eplv6],
    // ]
    // eplv 1, 2, 3 came from LA1 and eplv 4, 5, 6 came from LA2
    // e.g. eplv 1 and 4 need to have the same i and j index values

    // TODO: We'll suppose that the eplvs are in the correct order each eplv pair need to have the same i and j values
    // this may not be true, so I need to think about a way to assure this
    (0..len_list)
        .map(|i| {
            (0..num_of_la)
                .map(|j| eplvs_la_matrix[j][i].clone())
                .collect::<Vec<Eplv>>()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmscommon::core_types::plv_payload::PlvPayload;
    use scmscommon::core_types::pre_linkage_value::Eplv;
    use scmscommon::core_types::{Caterpillar, CaterpillarUbk};

    #[test]
    fn test_associate_caterpillar_with_eplv() {
        // Prepare test data

        let caterpillars = vec![
            Caterpillar::Ubk(CaterpillarUbk::gen_random_example()),
            Caterpillar::Ubk(CaterpillarUbk::gen_random_example()),
        ];

        let plv_payload1 = PlvPayload {
            req_id: 1,
            eplvs: vec![
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 2),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 3),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 4),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 5),
            ],
        };

        let plv_payload2 = PlvPayload {
            req_id: 2,
            eplvs: vec![
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 2),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 3),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 4),
                Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 5),
            ],
        };

        // Create a new PayloadLaToRa instance using the new method
        let payload_la_to_ra = PayloadLaToRa::new(
            "sample_request_id".to_owned(),
            vec![plv_payload1.clone(), plv_payload2.clone()],
        );

        let payloads_la_to_ra = vec![payload_la_to_ra.clone(), payload_la_to_ra.clone()];

        // Run the function
        let _result = associate_caterpillar_with_eplv(caterpillars, payloads_la_to_ra);
    }

    #[test]
    fn test_organize_eplvs_la_matrix_into_eplvs_pairs() {
        // Suppose that we have 2 certificates per period and just one period and 2 LAs.

        // Create some example Eplv instances
        let eplv1 = Eplv::new([1u8; 32], vec![2u8; 32], [5u8; 12], 1, 1);
        let eplv2 = Eplv::new([2u8; 32], vec![2u8; 32], [5u8; 12], 1, 2);
        let eplv3 = Eplv::new([3u8; 32], vec![2u8; 32], [5u8; 12], 1, 3);
        let eplv4 = Eplv::new([4u8; 32], vec![2u8; 32], [5u8; 12], 1, 1);
        let eplv5 = Eplv::new([5u8; 32], vec![2u8; 32], [5u8; 12], 1, 2);
        let eplv6 = Eplv::new([6u8; 32], vec![2u8; 32], [5u8; 12], 1, 3);

        // Create the eplvs_la_matrix
        let eplvs_la_matrix: Vec<Vec<Eplv>> = vec![
            vec![eplv1.clone(), eplv2.clone(), eplv3.clone()], // from LA 1
            vec![eplv4.clone(), eplv5.clone(), eplv6.clone()], // from LA 2
        ];

        // Call the function
        let result = organize_eplvs_la_matrix_into_eplv_pairs(eplvs_la_matrix);

        // Expected result
        let expected_result = vec![
            vec![eplv1.clone(), eplv4.clone()],
            vec![eplv2.clone(), eplv5.clone()],
            vec![eplv3.clone(), eplv6.clone()],
        ];

        // Assert that the result matches the expected result
        assert_eq!(result, expected_result);
        assert_eq!(result[0][0].i_index, expected_result[0][0].i_index);
        assert_eq!(result[0][0].j_index, expected_result[0][0].j_index);
        assert_eq!(result[0][1].i_index, expected_result[0][1].i_index);
        assert_eq!(result[0][1].j_index, expected_result[0][1].j_index);
    }
}
