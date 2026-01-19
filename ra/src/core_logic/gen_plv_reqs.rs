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

use crate::raconfig::RaConfig;
use scmscommon::Caterpillar;
use scmscommon::GlobalConfig;
use scmscommon::PlvReq;
use scmscommon::get_current_time_period_days;

/// Generates PlvReq using caterpillars, requests for PLV (Pre Linkage Values) to LA.
/// There is one PlvReq for each Caterpillar, and there is one Caterpillar per EE request.
pub fn gen_plv_reqs(caterpillars: &[Caterpillar]) -> Vec<PlvReq> {
    let ra_id = RaConfig::global().ra_id;

    let i_min = get_current_time_period_days() as u64;
    let i_max = i_min + GlobalConfig::global().number_cert_batches;
    let j_max = GlobalConfig::global().certificates_per_batch;

    let mut plv_reqs = Vec::new();
    for (req_id, _) in caterpillars.iter().enumerate() {
        let req_id = req_id as u64;
        let plv_req = PlvReq::new(req_id, ra_id, i_min, i_max, j_max);
        plv_reqs.push(plv_req);
    }

    plv_reqs
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_helper::ra_config_mock;
    use p256::ecdsa::{SigningKey, VerifyingKey};
    use rand::{RngCore, rngs::OsRng};
    use scmscommon::Caterpillar;
    use scmscommon::CaterpillarUbk;
    use scmscommon::CertificateType;
    use scmscommon::setup_global_config;

    #[test]
    fn test_gen_plv_reqs() {
        setup_global_config();
        ra_config_mock::setup_ra_conf();
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

        let plv_reqs = gen_plv_reqs(&caterpillars);
        assert_eq!(plv_reqs.len(), 3);
    }
}
