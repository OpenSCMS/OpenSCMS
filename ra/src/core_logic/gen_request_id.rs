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

use scmscommon::Caterpillar;
use sha2::{Digest, Sha256};

/// For each processing session we'll build a single request_id.
/// If there where e.g. 12 requests from EEs then for this bundle we'll create a single
/// request_id.
pub fn gen_request_id(caterpillars: &Vec<Caterpillar>) -> String {
    let mut hash_id_out_int: Vec<u8> = vec![0; 32];

    for caterpillar in caterpillars {
        let hash_id = caterpillar.get_hash_id();
        let hashed_bin_1 = Sha256::digest(hash_id.as_bytes());
        let hashed_bin_2 = hashed_bin_1.as_slice();

        hash_id_out_int
            .iter_mut()
            .zip(hashed_bin_2.iter())
            .for_each(|(x, &y)| {
                *x ^= y;
            });
    }

    hex::encode(hash_id_out_int)
}

#[test]
fn test_gen_request_id() {
    use p256::ecdsa::{SigningKey, VerifyingKey};
    use rand::{RngCore, rngs::OsRng};
    use scmscommon::CertificateType;
    use scmscommon::{Caterpillar, CaterpillarUbk};

    // Generate random values
    let mut rng = OsRng;

    // Generate random values
    let signing_key_sign = SigningKey::random(&mut rng);
    let pub_key_sign = VerifyingKey::from(&signing_key_sign);
    let mut f_sign = [0u8; 16];
    rng.fill_bytes(&mut f_sign);
    let certificate_type = CertificateType::Explicit;
    let exp_type = scmscommon::ExpansionType::Unified;

    let caterpillars = vec![
        Caterpillar::Ubk(CaterpillarUbk::new(
            1,
            pub_key_sign,
            f_sign,
            "123".to_owned(),
            certificate_type,
            exp_type,
        )),
        Caterpillar::Ubk(CaterpillarUbk::new(
            2,
            pub_key_sign,
            f_sign,
            "456".to_owned(),
            certificate_type,
            exp_type,
        )),
        Caterpillar::Ubk(CaterpillarUbk::new(
            3,
            pub_key_sign,
            f_sign,
            "789".to_owned(),
            certificate_type,
            exp_type,
        )),
    ];
    let request_id = gen_request_id(&caterpillars);
    assert_eq!(
        request_id,
        "2064a739684b41042fd3e5dc76d284256dffce3dc8732c259f3c2857519a3db2"
    );
}
