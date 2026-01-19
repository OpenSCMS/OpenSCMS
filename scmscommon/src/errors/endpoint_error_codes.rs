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

use derive_more::Display;
use num_derive::FromPrimitive;

#[derive(Debug, Display, Eq, PartialEq, Clone, Copy)]
#[display("{}")]
pub enum Ieee1609Dot2Dot1ErrorCodes {
    #[display("400")]
    NotDefined,
    #[display("{}", _0)]
    BadRequest(BadRequestErrorCodes),
    #[display("{}", _0)]
    Forbidden(ForbiddenErrorCodes),
    #[display("401")]
    Unauthorized,
}

// If scmsV3-error = fine and the RA or ECA returns 400 or 403, the proprietary Ieee-1609.2.1-Error
// header indicates the issue. The value from Ieee-1609.2.1-Error will be 400-xxx or 403-xxx
// where xxx is the error code defined in the standard's "Table 8 - Error codes"
// and follwing values are possible:
// For 400-xxx:
#[derive(Debug, Display, Eq, PartialEq, Clone, Copy, FromPrimitive)]
pub enum BadRequestErrorCodes {
    // Error series 400-1*
    #[display("400-10")]
    AbsentEncryption = 10,
    // Error series 400-2*
    #[display("400-20")]
    BlockedEnrollmentCertificate = 20,
    #[display("400-21")]
    UnregisteredEnrollmentCertificate = 21,
    // Error series 400-3*
    #[display("400-30")]
    DisallowedAppPermissions = 30,
    #[display("400-31")]
    DisallowedCertRequestPermissions = 31,
    #[display("400-32")]
    DisallowedOtherFields = 32,
    #[display("400-33")]
    DisallowedRegion = 33,
    #[display("400-34")]
    DisallowedTypeOfCertificate = 34,
    #[display("400-35")]
    DisallowedValidityPeriod = 35,
    // Error series 400-4*
    #[display("400-40")]
    FailedCertificateChainVerification = 40,
    #[display("400-41")]
    FailedDecryption = 41,
    #[display("400-42")]
    FailedParsing = 42,
    #[display("400-43")]
    FailedSignatureVerification = 43,
    // Error series 400-5*
    #[display("400-50")]
    FutureGenerationTime = 50,
    // Error series 400-6*
    #[display("400-60")]
    InvalidAppPermissions = 60,
    #[display("400-61")]
    InvalidCertRequestPermissions = 61,
    #[display("400-62")]
    InvalidEnrollmentCertificate = 62,
    #[display("400-63")]
    InvalidRegion = 63,
    #[display("400-64")]
    InvalidVerifyKeyIndicator = 64,
    // Error series 400-7*
    #[display("400-70")]
    OutsideEcaMaxAgeGenerationTime = 70,
    #[display("400-71")]
    OutsideEcaMaxReqs = 71,
    #[display("400-72")]
    OutsideEcaMaxWait = 72,
    #[display("400-73")]
    WithinEcaMinWait = 73,
    #[display("400-74")]
    OutsideDownloadMaxAgeGenerationTime = 74,
    #[display("400-75")]
    OutsideDownloadMaxReqs = 75,
    #[display("400-76")]
    WithinDownloadMinWait = 76,
    #[display("400-77")]
    OutsideRaMaxAgeGenerationTime = 77,
    #[display("400-78")]
    OutsideRaMaxGenDelayGenerationTime = 78,
    #[display("400-79")]
    OutsideRaMaxReqs = 79,
    #[display("400-710")]
    OutsideRaMaxOverlap = 710,
    #[display("400-711")]
    WithinRaMinWait = 711,
    #[display("400-712")]
    OutsideEnrollmentCertificatesValidityPeriod = 712,
    // Error series 400-8*
    #[display("400-80")]
    WrongEca = 80,
    #[display("400-81")]
    WrongRa = 81,
    // Undefined
    #[display("400-99")]
    Undefined = 99,
}

// For 403-xxx:
#[derive(Debug, Display, Eq, PartialEq, Clone, Copy, FromPrimitive)]
pub enum ForbiddenErrorCodes {
    #[display("403-43")]
    FailedSignatureVerification = 43,
    #[display("403-711")]
    OutsideEnrollmentCertificatesValidityPeriod = 711,
    // Undefined
    #[display("403-99")]
    Undefined = 99,
}
