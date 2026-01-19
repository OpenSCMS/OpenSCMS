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

use scmscommon::core_types::Certificate;
use scmscommon::{ExpansionType, GlobalConfig};

#[allow(dead_code)]
pub async fn setup_aca_mock_servers()
-> Result<((mockito::Mock, mockito::Server), Vec<Certificate>), Box<dyn std::error::Error>> {
    let aca_port = GlobalConfig::global().aca_port;
    let aca_prefix = &GlobalConfig::global().aca_prefix;

    let mut aca_server = mockito::Server::new_with_opts_async(mockito::ServerOpts {
        port: aca_port,
        host: aca_prefix,
        assert_on_drop: false,
    })
    .await;

    // Create a new Certificate instance using the new method
    let payload_aca_to_ra = vec![
        Certificate {
            req_id: 0,
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
    ];

    let aca_mock = aca_server
        .mock("POST", "/")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(serde_json::to_string(&payload_aca_to_ra).unwrap())
        .create();

    Ok(((aca_mock, aca_server), payload_aca_to_ra))
}
