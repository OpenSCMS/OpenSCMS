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

use crate::{
    OscmsErrorCode, OscmsErrorCode_OSCMS_ERROR_ABSENT_ENCRYPTION,
    OscmsErrorCode_OSCMS_ERROR_BLOCKED_ENROLLMENT_CERTIFICATE,
    OscmsErrorCode_OSCMS_ERROR_DISALLOWED_APP_PERMISSIONS,
    OscmsErrorCode_OSCMS_ERROR_DISALLOWED_CERT_REQUEST_PERMISSIONS,
    OscmsErrorCode_OSCMS_ERROR_DISALLOWED_OTHER_FIELDS,
    OscmsErrorCode_OSCMS_ERROR_DISALLOWED_REGION,
    OscmsErrorCode_OSCMS_ERROR_DISALLOWED_TYPE_OF_CERTIFICATE,
    OscmsErrorCode_OSCMS_ERROR_DISALLOWED_VALIDITY_PERIOD,
    OscmsErrorCode_OSCMS_ERROR_FAILED_CERTIFICATE_CHAIN_VERIFICATION,
    OscmsErrorCode_OSCMS_ERROR_FAILED_DECRYPTION, OscmsErrorCode_OSCMS_ERROR_FAILED_PARSING,
    OscmsErrorCode_OSCMS_ERROR_FAILED_SIGNATURE_VERIFICATION,
    OscmsErrorCode_OSCMS_ERROR_FUTURE_GENERATION_TIME,
    OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR,
    OscmsErrorCode_OSCMS_ERROR_INVALID_APP_PERMISSIONS,
    OscmsErrorCode_OSCMS_ERROR_INVALID_CERT_REQUEST_PERMISSIONS,
    OscmsErrorCode_OSCMS_ERROR_INVALID_ENROLLMENT_CERTIFICATE,
    OscmsErrorCode_OSCMS_ERROR_INVALID_REGION,
    OscmsErrorCode_OSCMS_ERROR_INVALID_VERIFY_KEY_INDICATOR,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_DOWNLOAD_MAX_AGE_GENERATION_TIME,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_DOWNLOAD_MAX_REQS,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ECA_MAX_AGE_GENERATION_TIME,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ECA_MAX_REQS,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ECA_MAX_WAIT,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ENROLLMENT_CERTIFICATES_VALIDITY_PERIOD,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_AGE_GENERATION_TIME,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_GEN_DELAY_GENERATION_TIME,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_OVERLAP,
    OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_REQS,
    OscmsErrorCode_OSCMS_ERROR_UNREGISTERED_ENROLLMENT_CERTIFICATE,
    OscmsErrorCode_OSCMS_ERROR_WITHIN_DOWNLOAD_MIN_WAIT,
    OscmsErrorCode_OSCMS_ERROR_WITHIN_ECA_MIN_WAIT, OscmsErrorCode_OSCMS_ERROR_WITHIN_RA_MIN_WAIT,
    OscmsErrorCode_OSCMS_ERROR_WRONG_ECA, OscmsErrorCode_OSCMS_ERROR_WRONG_RA,
};
use scmscommon::errors::{BadRequestErrorCodes, Ieee1609Dot2Dot1ErrorCodes};

#[derive(Debug, PartialEq)]
pub enum HashIdGenerationError {
    FailedToTryInto,
}

#[derive(Debug, PartialEq)]
pub struct OscmsBridgeError {
    pub status_code: OscmsErrorCode,
}

impl OscmsBridgeError {
    pub fn new(status_code: OscmsErrorCode) -> Self {
        OscmsBridgeError { status_code }
    }
}

pub fn oscms_bridge_error_to_ieee_1609_error_codes(
    oscms_error: OscmsBridgeError,
) -> (String, Ieee1609Dot2Dot1ErrorCodes) {
    match oscms_error.status_code {
        OscmsErrorCode_OSCMS_ERROR_INTERNAL_SERVER_ERROR => (
            format!(
                "Internal server error code: ErrorCode1609dot2({:?})",
                oscms_error
            ),
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ),
        OscmsErrorCode_OSCMS_ERROR_ABSENT_ENCRYPTION => (
            "Absent encryption".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::AbsentEncryption),
        ),
        OscmsErrorCode_OSCMS_ERROR_BLOCKED_ENROLLMENT_CERTIFICATE => (
            "Blocked enrollment certificate".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::BlockedEnrollmentCertificate,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_UNREGISTERED_ENROLLMENT_CERTIFICATE => (
            "Unregistered enrollment certificate".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::UnregisteredEnrollmentCertificate,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_DISALLOWED_APP_PERMISSIONS => (
            "Disallowed app permissions".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::DisallowedAppPermissions),
        ),
        OscmsErrorCode_OSCMS_ERROR_DISALLOWED_CERT_REQUEST_PERMISSIONS => (
            "Disallowed cert request permissions".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::DisallowedCertRequestPermissions,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_DISALLOWED_OTHER_FIELDS => (
            "Disallowed other fields".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::DisallowedOtherFields),
        ),
        OscmsErrorCode_OSCMS_ERROR_DISALLOWED_REGION => (
            "Disallowed region".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::DisallowedRegion),
        ),
        OscmsErrorCode_OSCMS_ERROR_DISALLOWED_TYPE_OF_CERTIFICATE => (
            "Disallowed type of certificate".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::DisallowedTypeOfCertificate,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_DISALLOWED_VALIDITY_PERIOD => (
            "Disallowed validity period".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::DisallowedValidityPeriod),
        ),
        OscmsErrorCode_OSCMS_ERROR_FAILED_CERTIFICATE_CHAIN_VERIFICATION => (
            "Failed certificate chain verification".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::FailedCertificateChainVerification,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_FAILED_DECRYPTION => (
            "Failed decryption".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::FailedDecryption),
        ),
        OscmsErrorCode_OSCMS_ERROR_FAILED_PARSING => (
            "Failed parsing".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::FailedParsing),
        ),
        OscmsErrorCode_OSCMS_ERROR_FAILED_SIGNATURE_VERIFICATION => (
            "Failed signature verification".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::FailedSignatureVerification,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_FUTURE_GENERATION_TIME => (
            "Future generation time".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::FutureGenerationTime),
        ),
        OscmsErrorCode_OSCMS_ERROR_INVALID_APP_PERMISSIONS => (
            "Invalid app permissions".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::InvalidAppPermissions),
        ),
        OscmsErrorCode_OSCMS_ERROR_INVALID_CERT_REQUEST_PERMISSIONS => (
            "Invalid cert request permissions".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::InvalidCertRequestPermissions,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_INVALID_ENROLLMENT_CERTIFICATE => (
            "Invalid enrollment certificate".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::InvalidEnrollmentCertificate,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_INVALID_REGION => (
            "Invalid region".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::InvalidRegion),
        ),
        OscmsErrorCode_OSCMS_ERROR_INVALID_VERIFY_KEY_INDICATOR => (
            "Invalid verify key indicator".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::InvalidVerifyKeyIndicator),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ECA_MAX_AGE_GENERATION_TIME => (
            "Outside ECA max age generation time".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::OutsideEcaMaxAgeGenerationTime,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ECA_MAX_REQS => (
            "Outside ECA max reqs".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::OutsideEcaMaxReqs),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ECA_MAX_WAIT => (
            "Outside ECA max wait".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::OutsideEcaMaxWait),
        ),
        OscmsErrorCode_OSCMS_ERROR_WITHIN_ECA_MIN_WAIT => (
            "Within ECA min wait".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::WithinEcaMinWait),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_DOWNLOAD_MAX_AGE_GENERATION_TIME => (
            "Outside download max age generation time".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::OutsideDownloadMaxAgeGenerationTime,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_DOWNLOAD_MAX_REQS => (
            "Outside download max reqs".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::OutsideDownloadMaxReqs),
        ),
        OscmsErrorCode_OSCMS_ERROR_WITHIN_DOWNLOAD_MIN_WAIT => (
            "Within download min wait".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::WithinDownloadMinWait),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_AGE_GENERATION_TIME => (
            "Outside RA max age generation time".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::OutsideRaMaxAgeGenerationTime,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_GEN_DELAY_GENERATION_TIME => (
            "Outside RA max gen delay generation time".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::OutsideRaMaxGenDelayGenerationTime,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_REQS => (
            "Outside RA max reqs".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::OutsideRaMaxReqs),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_RA_MAX_OVERLAP => (
            "Outside RA max overlap".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::OutsideRaMaxOverlap),
        ),
        OscmsErrorCode_OSCMS_ERROR_WITHIN_RA_MIN_WAIT => (
            "Within RA min wait".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::WithinRaMinWait),
        ),
        OscmsErrorCode_OSCMS_ERROR_OUTSIDE_ENROLLMENT_CERTIFICATES_VALIDITY_PERIOD => (
            "Outside enrollment certificates validity period".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                BadRequestErrorCodes::OutsideEnrollmentCertificatesValidityPeriod,
            ),
        ),
        OscmsErrorCode_OSCMS_ERROR_WRONG_ECA => (
            "Wrong ECA".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::WrongEca),
        ),
        OscmsErrorCode_OSCMS_ERROR_WRONG_RA => (
            "Wrong RA".to_string(),
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(BadRequestErrorCodes::WrongRa),
        ),
        _ => (
            format!("Internal server error code: Undefined"),
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        ),
    }
}
