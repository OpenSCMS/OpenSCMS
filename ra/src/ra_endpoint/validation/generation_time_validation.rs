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

use actix_web::http::StatusCode;
use scmscommon::GlobalConfig;
use scmscommon::errors::{BadRequestErrorCodes, HandleResponseError, Ieee1609Dot2Dot1ErrorCodes};

pub fn validate_generation_time(
    current_time: u32,
    generation_time: u32,
    time_request_received: u32,
    is_post_request: bool,
) -> Result<(), HandleResponseError> {
    log::debug!(
        "validate_generation_time: is_post_request: {} generation_time: {}, time_request_received: {}",
        is_post_request,
        generation_time,
        time_request_received
    );
    // Getting global config object
    let global_config = GlobalConfig::global();
    let mut max_age = global_config.param_ra_max_age as u32;
    let mut min_wait = global_config.param_ra_min_wait as u32;
    // /POST Throw Error- 400-711 within ra-minWait.
    // Minimum time that an EE shall wait before retrying the
    // request for a particular set of certificate permissions
    // (PSID/SSP) authorized by the same enrollment certificate.
    let mut within_min_wait_code = BadRequestErrorCodes::WithinRaMinWait;
    // Throw /POST Throw Error- 400-77: Outside ra-maxAge generationTime.
    // Maximum age of the authorization certificate request for a
    // particular set of certificate permissions (PSID/SSP) when
    // received by the RA, that is, the maximum amount of time in
    // the past that the generationTime field in the
    // EeRaCertRequest can represent relative to the time that
    // the RA receives the request.
    let mut outside_max_age_generation_time_code =
        BadRequestErrorCodes::OutsideRaMaxAgeGenerationTime;

    if !is_post_request {
        max_age = global_config.param_download_max_age as u32;
        min_wait = global_config.param_download_min_wait as u32;
        // Throw /GET Error-Code 400-76 within download-minWait.
        // Minimum time that an EE shall wait before retrying the request.
        within_min_wait_code = BadRequestErrorCodes::WithinDownloadMinWait;
        // Throw /POST Error-Code 400-74 outside download-maxAge generationTime.
        // Maximum age of a request for a particular set of certificate permissions
        // (PSID/SSP) when received by the RA, that is, the maximum amount of time
        // in the past that the generationTime field in the
        // EeRaDownloadRequest can represent relative to the time that the RA
        // receives the request.
        outside_max_age_generation_time_code =
            BadRequestErrorCodes::OutsideDownloadMaxAgeGenerationTime;
    }

    validate(
        current_time,
        generation_time,
        time_request_received,
        max_age,
        min_wait,
        within_min_wait_code,
        outside_max_age_generation_time_code,
    )
}

fn validate(
    current_time: u32,
    generation_time: u32,
    time_request_received: u32,
    max_age: u32,
    min_wait: u32,
    within_min_wait_code: BadRequestErrorCodes,
    outside_max_age_generation_time_code: BadRequestErrorCodes,
) -> Result<(), HandleResponseError> {
    // Throw Error- 400-50: Future generationTime
    // when reading the generation time compare with current time to check and throw error if future time
    if generation_time > current_time {
        log::debug!(
            "Error- 400-50: Future generationTime: generation_time: {}, current_time: {}",
            generation_time,
            current_time
        );
        return Err(HandleResponseError::new(
            "Future generation time",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::FutureGenerationTime),
        ));
    }

    // Throw /POST Error-Code 400-XX outside xxx-maxAge generationTime.
    // we need to maintain timer at this point to calculate between generation time and time that the RA receives the request.
    if generation_time + max_age < time_request_received {
        log::debug!(
            "Error-Code 400-XX outside ra-maxAge generationTime: generation_time: {}, time_request_received: {}, max_age: {}",
            generation_time,
            time_request_received,
            max_age
        );
        return Err(HandleResponseError::new(
            "Outside ra-maxAge generationTime",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(outside_max_age_generation_time_code),
        ));
    }

    // Throw /POST Error-Code 400-XX within XX-minWait.
    // check if generation time > current time + minwaittime holds, else throw this error
    // TODO check this, is it the oposite
    if current_time < generation_time + min_wait {
        log::debug!(
            "Error-Code 400-XX within ra-minWait: generation_time: {}, current_time: {}, min_wait: {}",
            generation_time,
            current_time,
            min_wait
        );
        return Err(HandleResponseError::new(
            "Within ra-minWait",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(within_min_wait_code),
        ));
    }

    log::debug!("Generation time validated successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmscommon::setup_global_config;

    fn get_current_time_seconds() -> u32 {
        12345680
    }

    fn test_future_generation_time_failure() {
        // POST future generation time failure
        let post_and_get = vec![true, false];
        for mode in post_and_get {
            let current_time = get_current_time_seconds();
            let generation_time = current_time + 1000;
            let time_request_received = get_current_time_seconds();
            match validate_generation_time(
                current_time,
                generation_time,
                time_request_received,
                mode,
            ) {
                Ok(_) => panic!("Expected error"),
                Err(e) => {
                    assert_eq!(
                        e.error_code,
                        Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                            BadRequestErrorCodes::FutureGenerationTime
                        )
                    );
                }
            }
        }
    }

    fn test_outside_max_age() {
        // Test 3: POST outside ra-maxAge generationTime failure
        let bad_request_expected_error = vec![
            (true, BadRequestErrorCodes::OutsideRaMaxAgeGenerationTime),
            (
                false,
                BadRequestErrorCodes::OutsideDownloadMaxAgeGenerationTime,
            ),
        ];

        for expected in bad_request_expected_error {
            let current_time = 100;
            let generation_time = 100;
            let time_request_received = 1200;
            match validate_generation_time(
                current_time,
                generation_time,
                time_request_received,
                expected.0,
            ) {
                Ok(_) => panic!("Expected error"),
                Err(e) => {
                    assert_eq!(
                        e.error_code,
                        Ieee1609Dot2Dot1ErrorCodes::BadRequest(expected.1)
                    );
                }
            }
        }
    }

    fn test_within_min_wait() {
        // Test: POST within ra-minWait.
        let bad_request_expected_error = vec![
            (true, BadRequestErrorCodes::WithinRaMinWait),
            (false, BadRequestErrorCodes::WithinDownloadMinWait),
        ];

        for expected in bad_request_expected_error {
            let current_time = 100;
            let generation_time = 100;
            let time_request_received = 100;
            match validate_generation_time(
                current_time,
                generation_time,
                time_request_received,
                expected.0,
            ) {
                Ok(_) => panic!("Expected error"),
                Err(e) => {
                    assert_eq!(
                        e.error_code,
                        Ieee1609Dot2Dot1ErrorCodes::BadRequest(expected.1)
                    );
                }
            }
        }
    }

    fn test_validation_ok() {
        // Test Ok
        let post_and_get = vec![true, false];
        let current_time = get_current_time_seconds();
        let generation_time = current_time - 100;
        let time_request_received = generation_time - 108;
        for mode in post_and_get {
            match validate_generation_time(
                current_time,
                generation_time,
                time_request_received,
                mode,
            ) {
                Ok(_) => {}
                Err(e) => {
                    panic!("No error Expecte at POST but got {:?}", e);
                }
            }
        }
    }

    #[test]
    fn test_validation_generation_time() {
        // Setup test env vars
        setup_global_config();

        test_future_generation_time_failure();
        test_outside_max_age();
        test_within_min_wait();
        test_validation_ok();
    }
}
