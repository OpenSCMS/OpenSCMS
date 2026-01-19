# Parameters per Section 4.3 of IEEE1609.2.1-2022

## Copyright and License Information

Unless otherwise specified, all content, including all source code files and
documentation files in this repository are:

Copyright (c) 2025 LG Electronics, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

<http://www.apache.org/licenses/LICENSE-2.0>

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

SPDX-License-Identifier: Apache-2.0

## Time based

Table 1, p40

| Parameter                | Value                                                                               |
| ------------------------ | ----------------------------------------------------------------------------------- |
| time-iPeriod             | integer - dynamic                                                                   |
| time-iPeriodEpoch        | date - danymic                                                                      |
| time-iPeriodInit         | integer - dynamic                                                                   |
| time-iPeriodLength       | 1 week                                                                              |
| time-overlap             | 0, we are not using overlap                                                         |
| time-baseCertNumber      | 10 (but it should be more)                                                          |
| time-overlapCertNumber   | 0, as explained before we are not using overlap                                     |
| time-extraBands          | 0, there are no extra bands                                                         |
|                          | Given that we are only using band 0 we’ll not have values for the parameters below. |
| time-band<n>-startOffset | None                                                                                |
| time-band<n>-duration    | None                                                                                |
| timeband<n>-certsInBand  | None                                                                                |

## Session parameters

Table 2 - pp 44

| Parameter        | Value  | Meaning                                                |
| ---------------- | ------ | ------------------------------------------------------ |
| session-scmsAuth | tls1.3 | Session is protected cryptographically using TLS 1.3   |
| session-eeAuth   | none   | No authentication by the EE at the secure session leve |

## Web API - Generic

Table 3, pp 44

| Parameter     | Value  | Meaning                                                                    |
| ------------- | ------ | -------------------------------------------------------------------------- |
| webApi-name   | scmsV3 | Name of the Web API is SCMS representational state transfer (REST) API v3. |
| webApi-eeAuth | none   | No authentication by the EE at the Web API level.                          |

## Web API - SCMS REST V3

Table 4, pp 45

| Parameter      | Value      | Meaning                                                                                                                    |
| -------------- | ---------- | -------------------------------------------------------------------------------------------------------------------------- |
| scmsV3-eeAuth  | canonical  | Authentication by the EE at the SCMS REST API v3 level is done by the EE using a valid canonical key                       |
| scmsV3-eeAuth  | enrollment | Authentication by the EE at the SCMS REST API v3 level is done by the EE using a valid IEEE 1609.2 enrollment certificate. |
| scmsV3-error   | fine       | Errors are returned as specified for each command                                                                          |
| scmsV3-error   | coarse     | No information is returned in case of error, just an HTTP code of 500.                                                     |
| scmsV3-options | n/a        | Not Supported                                                                                                              |

## ECA Parameters

Table 5, pp 45

| Parameter   | Value | Meaning                                                                                                                                                                                                                                                 |
| ----------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| eca-maxAge  | 1000  | Maximum age of the enrollment certificate request when received by the ECA, that is, the maximum amount of time in the past that the generationTime field in the EeEcaCertRequest can represent relative to the time that the ECA receives the request. |
| eca-maxReqs | 100   | Maximum number of requests in a 7 × 24-hour period. The requests within a given period are determined based on the generationTime field in the request not on the reception time at the ECA.                                                            |
| eca-maxWait | 100   | Maximum time limit between the generationTime field and the tbsCert.validityPeriod.start field in the EeEcaCertRequest.                                                                                                                                 |
| eca-minWait | 0     | Minimum time that an EE shall wait before retrying the request.                                                                                                                                                                                         |

## RA Request Parameters

Table 6, pp 46

| Parameter                                       | Value           | Meaning                                                                                                                                                                                                                                                                                                           |
| ----------------------------------------------- | --------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ra-acpcSupport                                  | Yes             | Indicates whether or not ACPC is supported by the RA.                                                                                                                                                                                                                                                             |
| ra-butterflyType                                | compact unified | Indicates which variations of the butterfly key mechanism are supported by the RA. Current options include original, unified, and compact unified.                                                                                                                                                                |
| ra-maxAge                                       | 0               | Maximum age of the authorization certificate request for a particular set of certificate permissions (PSID/SSP) when received by the RA, that is, the maximum amount of time in the past that the generationTime field in th EeRaCertRequest can represent relative to the time that the RA receives the request. |
| ra-maxGenDelay                                  | 4000            | Maximum time between generationTime in the EeRaCertRequest and the requested start time of the certificates (requested using the validityPeriod field in the tbsCert field).                                                                                                                                      |
| ra-maxReqs                                      | 100             | Maximum number of requests in a 7 × 24-hour period for a particular set of certificate permissions (PSID/SSP) authorized by the same enrollment certificate. The requests within a given period are determined based on the generationTime field in the request, not on the reception time at the RA.             |
| ra-maxPreloadTime                               | 0               | Maximum time from the current time that the SCMS will generate certificates for an EE, for example, up to 3 years from the current time.                                                                                                                                                                          |
| ra-minWait                                      | 0               | Authorization certificate request: Minimum time that an EE shall wait before retrying the request for a particular set of certificate permissions(PSID/SSP) authorized by the same enrollment certificate.                                                                                                        |
| ra-minWait                                      | 0               | Successor enrollment certificate request: Minimum time that an EE shall wait before retrying the                                                                                                                                                                                                                  |
| request for a successor enrollment certificate. |

## Download requests

| Parameter | Value | Meaning |
| ---------------- | ----- | ---- |
| download-maxAge  | 1000  | Maximum age of a request for a particular set of certificate permissions (PSID/SSP) when received by the RA, that is, the maximum amount of time in the past that the generationTime field in the EeRaDownloadRequest can represent relative to the time that the RA receives the request. |
| download-maxReqs | 100   | Maximum number of requests in a 7 × 24-hour period. The requests within a given period are determined based on the generationTime field in the request, not on the reception time at the RA. |
| download-minWait | 0     | Minimum time that an EE shall wait before retrying the request. |
