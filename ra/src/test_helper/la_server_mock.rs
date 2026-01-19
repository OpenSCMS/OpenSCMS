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

use rand::{self, Rng};
use scmscommon::GlobalConfig;
use scmscommon::core_types::PayloadLaToRa;
use scmscommon::plv_payload::PlvPayload;
use scmscommon::pre_linkage_value::Eplv;

#[allow(dead_code)]
pub async fn setup_la_mock_servers() -> Result<
    (
        (Vec<mockito::Mock>, Vec<mockito::Server>),
        Vec<PayloadLaToRa>,
    ),
    Box<dyn std::error::Error>,
> {
    let la_count = GlobalConfig::global().la_count;
    let la_port = GlobalConfig::global().la_port;
    let la_prefix = &GlobalConfig::global().la_prefix;

    let mut la_servers: Vec<mockito::Server> = Vec::new();
    for i in 0..la_count {
        let la_address = format!("{}{}", la_prefix, i + 1);
        let la = mockito::Server::new_with_opts_async(mockito::ServerOpts {
            port: la_port,
            host: Box::leak(la_address.into_boxed_str()),
            assert_on_drop: false,
        })
        .await;
        la_servers.push(la);
    }

    let mut rng = rand::thread_rng();
    let mut la_random_payloads: Vec<PayloadLaToRa> = Vec::new();

    let enc_value = [1u8; 32];
    let ephemeral_pub_key = vec![2u8; 32];
    let nonce = [5u8; 12];

    for _i in 0..la_count {
        let la_response = PayloadLaToRa {
            request_id: "sample_request_id".to_owned(),
            plv_payloads: vec![PlvPayload {
                req_id: rng.r#gen(),
                eplvs: vec![
                    Eplv::new(
                        enc_value,
                        ephemeral_pub_key.clone(),
                        nonce,
                        rng.r#gen(),
                        rng.r#gen(),
                    ),
                    Eplv::new(
                        enc_value,
                        ephemeral_pub_key.clone(),
                        nonce,
                        rng.r#gen(),
                        rng.r#gen(),
                    ),
                ],
            }],
        };
        la_random_payloads.push(la_response);
    }

    let mut la_mocks: Vec<mockito::Mock> = Vec::new();
    for i in 0..la_count {
        let m = la_servers[i as usize]
            .mock("POST", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&la_random_payloads[i as usize]).unwrap())
            .create();
        la_mocks.push(m);
    }

    Ok(((la_mocks, la_servers), la_random_payloads))
}
