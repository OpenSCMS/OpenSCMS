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

use futures::future::try_join_all;
use scmscommon::{
    GlobalConfig,
    core_types::{Caterpillar, PayloadLaToRa, PayloadRaToLa},
    errors,
};

use crate::core_logic::gen_plv_reqs;
use crate::ra_endpoint::worker::common::post_ra_to_la;

pub async fn handle_multiple_la_requests(
    caterpillars: &[Caterpillar],
    request_id: String,
) -> Result<Vec<PayloadLaToRa>, errors::ScmsInternalCommError> {
    log::info!("Handling multiple LA requests");
    let plv_reqs = gen_plv_reqs(caterpillars);
    let payload_ra_to_la = PayloadRaToLa::new(request_id, plv_reqs);

    let payloads_la_to_ra: Result<Vec<PayloadLaToRa>, errors::ScmsInternalCommError> =
        try_join_all(
            (1..GlobalConfig::global().la_count + 1)
                .map(|la_id| post_to_la(payload_ra_to_la.clone(), la_id)),
        )
        .await;
    log::info!(
        "handle_multiple_la_requests: payloads_la_to_ra: {:?}",
        payloads_la_to_ra
    );
    payloads_la_to_ra
}

async fn post_to_la(
    payload_ra_to_la: PayloadRaToLa,
    la_id: u16,
) -> Result<PayloadLaToRa, errors::ScmsInternalCommError> {
    log::info!("Posting to LA {}", la_id);

    // Construct request address
    let la_addr = GlobalConfig::global().la_addr(la_id);
    let la_port = GlobalConfig::global().la_port;
    let req_addr = format!("{}:{}/", la_addr, la_port);

    log::debug!("Posting to LA id: {} addr: {} ", la_id, req_addr);
    let response_result = post_ra_to_la(req_addr, payload_ra_to_la).await;
    match response_result {
        Ok(response) => {
            log::debug!("Received response from LA{}", la_id,);
            // Capture PayloadLaToRa
            if response.status().is_success() {
                let payload_la_to_ra: PayloadLaToRa = response.json().await?;
                log::debug!(
                    "LA{} response is ok, returning payload [request_id={}, plv len={}]!",
                    la_id,
                    payload_la_to_ra.request_id,
                    payload_la_to_ra.plv_payloads.len()
                );
                return Ok(payload_la_to_ra);
            }

            let error_code = response.status().as_u16();
            Err(errors::ScmsInternalCommError::new(
                format!(
                    "Received response from LA {}, but not with error status code: {}",
                    la_id, error_code
                )
                .as_str(),
                errors::InternalCommWire::RaToLa,
                error_code,
            ))
        }
        Err(e) => Err(errors::ScmsInternalCommError::new(
            format!(
                "Failed to Communicate with LA {}, status code: {}",
                la_id, e
            )
            .as_str(),
            errors::InternalCommWire::RaToLa,
            500,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_logic::gen_request_id;
    use crate::test_helper::{caterpillar_mock, la_server_mock, ra_config_mock};
    use scmscommon::setup_global_config;

    fn setup() {
        setup_global_config();
        ra_config_mock::setup_ra_conf();
    }

    #[tokio::test]
    async fn test_handle_multiple_la_requests() {
        setup();

        let (_setup, expected_responses) = match la_server_mock::setup_la_mock_servers().await {
            Ok((setup, expected_responses)) => (setup, expected_responses),
            Err(e) => {
                panic!("Failed to setup LA mock servers: {:#?}", e);
            }
        };

        let caterpillars = caterpillar_mock::mock_caterpillars();
        let request_id = gen_request_id(&caterpillars);
        let response = handle_multiple_la_requests(&caterpillars, request_id).await;
        match response {
            Ok(payloads) => {
                assert_eq!(payloads.len(), expected_responses.len());
                for (payload, expected_payload) in payloads.iter().zip(expected_responses.iter()) {
                    assert_eq!(payload.request_id, expected_payload.request_id);
                    assert_eq!(
                        payload.plv_payloads.len(),
                        expected_payload.plv_payloads.len()
                    );
                    for (plv_payload, expected_plv_payload) in payload
                        .plv_payloads
                        .iter()
                        .zip(expected_payload.plv_payloads.iter())
                    {
                        assert_eq!(plv_payload.req_id, expected_plv_payload.req_id);
                        assert_eq!(plv_payload.eplvs.len(), expected_plv_payload.eplvs.len());
                        for (eplv, expected_eplv) in plv_payload
                            .eplvs
                            .iter()
                            .zip(expected_plv_payload.eplvs.iter())
                        {
                            assert_eq!(eplv.enc_value, expected_eplv.enc_value);
                            assert_eq!(eplv.j_index, expected_eplv.j_index);
                            assert_eq!(eplv.i_index, expected_eplv.i_index);
                        }
                    }
                }
            }
            Err(e) => {
                panic!("Failed to handle multiple LA requests: {:#?}", e);
            }
        }
    }
}
