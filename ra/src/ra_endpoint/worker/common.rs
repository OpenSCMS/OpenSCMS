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

use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::{RetryTransientMiddleware, policies::ExponentialBackoff};
use scmscommon::GlobalConfig;
use scmscommon::core_types::{PayloadRaToAca, PayloadRaToLa};

fn client_with_middleware() -> ClientWithMiddleware {
    // Retry up to ra_max_requests times with ra_min_wait second delay between each attempt
    let ra_max_requests = GlobalConfig::global().param_ra_max_reqs as u32;
    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(ra_max_requests);

    ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build()
}

pub async fn post_ra_to_la(
    req_addr: String,
    payload: PayloadRaToLa,
) -> Result<reqwest::Response, reqwest_middleware::Error> {
    let client = client_with_middleware();
    client.post(req_addr).json(&payload).send().await
}

pub async fn post_ra_to_aca(
    req_addr: String,
    payload: PayloadRaToAca,
) -> Result<reqwest::Response, reqwest_middleware::Error> {
    let client = client_with_middleware();
    client.post(req_addr).json(&payload).send().await
}

pub async fn post_ra_to_eca(
    req_addr: String,
    payload: Vec<u8>,
    request_hash_id: String,
) -> Result<reqwest::Response, reqwest_middleware::Error> {
    let client = client_with_middleware();
    client
        .post(req_addr)
        .body(payload)
        .header("request_hash_id", request_hash_id)
        .send()
        .await
}

pub async fn get_ra_to_eca(
    req_addr: String,
) -> Result<reqwest::Response, reqwest_middleware::Error> {
    let client = client_with_middleware();
    client.get(req_addr).send().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct PostPayload {
        req_field1: String,
        req_field2: u32,
    }

    #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
    struct PostPayloadResponse {
        status: u64,
        message: String,
        error: String,
    }

    #[tokio::test]
    async fn test_post_ra_to_component_200() {
        // Mock Response
        let source_response_obj = PostPayloadResponse {
            status: 200,
            message: "OK".to_string(),
            error: "".to_string(),
        };

        // Mock the server
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/v3/authorization-certificate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(serde_json::to_string(&source_response_obj).unwrap())
            .create();

        // Prepare request
        let req_addr = format!("{}/v3/authorization-certificate", server.url());
        let payload_ra_to_la = PayloadRaToLa::new("sample_request_id".to_string(), vec![]);

        // Call endpoint
        let response = post_ra_to_la(req_addr, payload_ra_to_la).await.unwrap();

        // Check response
        assert_eq!(response.status(), 200);
        let response_body = response.text().await.unwrap();
        let response_obj: PostPayloadResponse = serde_json::from_str(&response_body).unwrap();
        assert_eq!(response_obj, source_response_obj);
    }

    #[tokio::test]
    async fn test_post_ra_to_component_timeout() {
        // Mock the server
        let mut server = mockito::Server::new_async().await;
        let _m = server
            .mock("POST", "/v3/authorization-certificate")
            .with_status(503)
            .with_header("content-type", "application/json")
            .with_body("TIMEOUT")
            .create();

        // Prepare request
        let req_addr = format!("{}/v3/authorization-certificate", server.url());
        let payload_ra_to_la = PayloadRaToLa::new("sample_request_id".to_string(), vec![]);

        // Call endpoint
        let response = post_ra_to_la(req_addr, payload_ra_to_la).await;
        // Check response
        assert_eq!(response.unwrap().status(), 503);
    }
}
