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

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{CertificateType, RaButterflyType, ScmsEeAuth, ScmsErrorLevel, WebApiEeAuth};
use crate::{GlobalConfig, get_current_time_period_days};

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub enum CryptoAlgorithmType {
    #[serde(alias = "nistP256", rename(serialize = "nistP256"))]
    NistP256,
    #[serde(alias = "nistP384", rename(serialize = "nistP384"))]
    NistP384,
    #[serde(alias = "brainpoolP256", rename(serialize = "brainpoolP256"))]
    BrainpoolP256,
    #[serde(alias = "brainpoolP384", rename(serialize = "brainpoolP384"))]
    BrainpoolP384,
    #[serde(alias = "sha256", rename(serialize = "sha256"))]
    Sha256,
    #[serde(alias = "sha384", rename(serialize = "sha384"))]
    Sha384,
    #[serde(alias = "aes128Ccm", rename(serialize = "aes128Ccm"))]
    Aes128Ccm,
    #[serde(alias = "sm2", rename(serialize = "sm2"))]
    Sm2,
    #[serde(alias = "sm3", rename(serialize = "sm3"))]
    Sm3,
    #[serde(alias = "sm4", rename(serialize = "sm4"))]
    Sm4,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct PsidConfigInfo {
    pub all: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct RegionConfigInfo {
    pub country: Vec<String>,
    #[serde(alias = "countryAndRegion", rename(serialize = "countryAndRegion"))]
    pub country_and_region: Vec<String>,
    #[serde(
        alias = "countryAndRegionAndSubRegion",
        rename(serialize = "countryAndRegionAndSubRegion")
    )]
    pub country_and_region_and_sub_region: Vec<String>,
    pub circle: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct SupportRegionConfigInfo {
    pub region: Vec<RegionConfigInfo>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct CertificateConfigInfo {
    pub name: String,
    #[serde(alias = "cryptoAlgorithm", rename(serialize = "cryptoAlgorithm"))]
    pub crypto_algorithm: Vec<CryptoAlgorithmType>,
    #[serde(alias = "certificateType", rename(serialize = "certificateType"))]
    pub certificate_type: CertificateType,
    pub start: u32,
    pub expiration: u32,
    #[serde(alias = "inUse", rename(serialize = "inUse"))]
    pub in_use: u32,
    #[serde(alias = "requestForRenewal", rename(serialize = "requestForRenewal"))]
    pub request_for_renewal: u32,
    #[serde(
        alias = "concurrentlyValidCertificates",
        rename(serialize = "concurrentlyValidCertificates")
    )]
    pub concurrently_valid_certificates: u16,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct ScmsV3AuthConfig {
    #[serde(
        alias = "enrollment-certificate",
        rename(serialize = "enrollment-certificate")
    )]
    pub enrollment_certificate: Vec<ScmsEeAuth>,
    #[serde(
        alias = "authorization-certificate-request",
        rename(serialize = "authorization-certificate-request")
    )]
    pub authorization_certificate_request: Vec<ScmsEeAuth>,
    #[serde(
        alias = "authorization-certificate-download",
        rename(serialize = "authorization-certificate-download")
    )]
    pub authorization_certificate_download: Vec<ScmsEeAuth>,
    #[serde(
        alias = "authorization-certificate-download-filename",
        rename(serialize = "authorization-certificate-download-filename")
    )]
    pub authorization_certificate_download_filename: Vec<ScmsEeAuth>,
    #[serde(alias = "ccf-ctl", rename(serialize = "ccf-ctl"))]
    pub ccf_ctl: Vec<ScmsEeAuth>,
    #[serde(alias = "composite-ccf-ctl", rename(serialize = "composite-ccf-ctl"))]
    pub composite_crl_ctl: Vec<ScmsEeAuth>,
    #[serde(alias = "ca-certificate", rename(serialize = "ca-certificate"))]
    pub ca_certificate: Vec<ScmsEeAuth>,
    pub crl: Vec<ScmsEeAuth>,
    pub ctl: Vec<ScmsEeAuth>,
    #[serde(alias = "ra-certificate", rename(serialize = "ra-certificate"))]
    pub ra_certificate: Vec<ScmsEeAuth>,
    #[serde(
        alias = "certificate-management-info-status",
        rename(serialize = "certificate-management-info-status")
    )]
    pub certificate_management_info_status: Vec<ScmsEeAuth>,
    #[serde(
        alias = "successor-enrollment-certificate-request",
        rename(serialize = "successor-enrollment-certificate-request")
    )]
    pub successor_enrollment_certificate_request: Vec<ScmsEeAuth>,
    #[serde(
        alias = "successor-enrollment-certificate-download",
        rename(serialize = "successor-enrollment-certificate-download")
    )]
    pub successor_enrollment_certificate_download: Vec<ScmsEeAuth>,
    #[serde(alias = "misbehavior-report", rename(serialize = "misbehavior-report"))]
    pub misbehavior_report: Vec<ScmsEeAuth>,
    #[serde(alias = "ma-certificate", rename(serialize = "ma-certificate"))]
    pub ma_certificate: Vec<ScmsEeAuth>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct RaButterflyTypeConfig {
    pub original: Vec<String>,
    pub unified: Vec<String>,
    #[serde(alias = "compactUnified", rename(serialize = "compactUnified"))]
    pub compact_unified: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct NextUpdateTimeDefault {
    #[serde(alias = "fullHashCrl", rename(serialize = "fullHashCrl"))]
    pub full_hash_crl: u32,
    #[serde(alias = "deltaHashCrl", rename(serialize = "deltaHashCrl"))]
    pub delta_hash_crl: u32,
    #[serde(alias = "fullLinkedCrl", rename(serialize = "fullLinkedCrl"))]
    pub full_linked_crl: u32,
    #[serde(alias = "deltaLinkedCrl", rename(serialize = "deltaLinkedCrl"))]
    pub delta_linked_crl: u32,
    #[serde(alias = "fullRootCtl", rename(serialize = "fullRootCtl"))]
    pub full_root_ctl: u32,
    #[serde(alias = "deltaRootCtl", rename(serialize = "deltaRootCtl"))]
    pub delta_root_ctl: u32,
    #[serde(alias = "fullDomainCtl", rename(serialize = "fullDomainCtl"))]
    pub full_domain_ctl: u32,
    #[serde(alias = "deltaDomainCtl", rename(serialize = "deltaDomainCtl"))]
    pub delta_domain_ctl: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct GlobalConfigInfo {
    // General Info
    #[serde(alias = "quorumCount", rename(serialize = "quorumCount"))]
    pub quorum_count: u16,
    #[serde(alias = "ctlSeriesId", rename(serialize = "ctlSeriesId"))]
    pub ctl_series_id: String,
    #[serde(alias = "cryptoAlgorithm", rename(serialize = "cryptoAlgorithm"))]
    pub crypto_algorithm: Vec<CryptoAlgorithmType>,

    // Certificates
    #[serde(alias = "certElector", rename(serialize = "certElector"))]
    pub cert_elector: CertificateConfigInfo,
    #[serde(alias = "certRootCa", rename(serialize = "certRootCa"))]
    pub cert_root_ca: CertificateConfigInfo,
    #[serde(alias = "certIca", rename(serialize = "certIca"))]
    pub cert_ica: CertificateConfigInfo,
    #[serde(alias = "certEca", rename(serialize = "certEca"))]
    pub cert_eca: CertificateConfigInfo,
    #[serde(alias = "certAca", rename(serialize = "certAca"))]
    pub cert_aca: CertificateConfigInfo,
    #[serde(alias = "certPca", rename(serialize = "certPca"))]
    pub cert_pca: CertificateConfigInfo,
    #[serde(alias = "certCrlSigner", rename(serialize = "certCrlSigner"))]
    pub cert_crl_signer: CertificateConfigInfo,
    #[serde(alias = "certRa", rename(serialize = "certRa"))]
    pub cert_ra: CertificateConfigInfo,
    #[serde(alias = "certLa", rename(serialize = "certLa"))]
    pub cert_la: CertificateConfigInfo,
    #[serde(alias = "certLa1", rename(serialize = "certLa1"))]
    pub cert_la1: CertificateConfigInfo,
    #[serde(alias = "certLa2", rename(serialize = "certLa2"))]
    pub cert_la2: CertificateConfigInfo,
    #[serde(alias = "certMa", rename(serialize = "certMa"))]
    pub cert_ma: CertificateConfigInfo,
    #[serde(alias = "certPg", rename(serialize = "certPg"))]
    pub cert_pg: CertificateConfigInfo,
    #[serde(alias = "certDc", rename(serialize = "certDc"))]
    pub cert_dc: CertificateConfigInfo,
    #[serde(alias = "certEnrollment", rename(serialize = "certEnrollment"))]
    pub cert_enrollment: CertificateConfigInfo,
    #[serde(alias = "certPseudonym", rename(serialize = "certPseudonym"))]
    pub cert_pseudonym: CertificateConfigInfo,
    #[serde(alias = "certIdentification", rename(serialize = "certIdentification"))]
    pub cert_identification: CertificateConfigInfo,

    // Region and Psids
    #[serde(alias = "supportRegion", rename(serialize = "supportRegion"))]
    pub support_region: Vec<RegionConfigInfo>,
    #[serde(alias = "supportPsidSsp", rename(serialize = "supportPsidSsp"))]
    pub support_psid_ssp: Vec<PsidConfigInfo>,

    // URLs
    #[serde(alias = "urlDomain", rename(serialize = "urlDomain"))]
    pub url_domain: String,
    #[serde(alias = "ecaHostname", rename(serialize = "ecaHostname"))]
    pub eca_hostname: String,
    #[serde(alias = "raHostname", rename(serialize = "raHostname"))]
    pub ra_hostname: String,
    #[serde(alias = "dcHostname", rename(serialize = "dcHostname"))]
    pub dc_hostname: String,
    #[serde(alias = "dcmHostname", rename(serialize = "dcmHostname"))]
    pub dcm_hostname: String,
    #[serde(alias = "ecaUrl", rename(serialize = "ecaUrl"))]
    pub eca_url: String,
    #[serde(alias = "raUrl", rename(serialize = "raUrl"))]
    pub ra_url: String,
    #[serde(alias = "dcUrl", rename(serialize = "dcUrl"))]
    pub dc_url: String,
    #[serde(alias = "dcmUrl", rename(serialize = "dcmUrl"))]
    pub dcm_url: String,

    // Time and Session Parameters
    #[serde(alias = "time-iPeriod", rename(serialize = "time-iPeriod"))]
    pub time_i_period: u64,
    #[serde(alias = "time-iPeriodEpoch", rename(serialize = "time-iPeriodEpoch"))]
    pub time_i_period_epoch: u64,
    #[serde(alias = "time-iPeriodInit", rename(serialize = "time-iPeriodInit"))]
    pub time_i_period_init: u64,
    #[serde(alias = "time-iPeriodLength", rename(serialize = "time-iPeriodLength"))]
    pub time_i_period_length: u64,
    #[serde(alias = "time-overlap", rename(serialize = "time-overlap"))]
    pub time_overlap: u64,
    #[serde(
        alias = "time-baseCertNumber",
        rename(serialize = "time-baseCertNumber")
    )]
    pub time_base_cert_number: u64,
    #[serde(
        alias = "time-overlapCertNumber",
        rename(serialize = "time-overlapCertNumber")
    )]
    pub time_overlap_cert_number: u64,
    #[serde(alias = "session-scmsAuth", rename(serialize = "session-scmsAuth"))]
    pub session_scms_auth: String,
    #[serde(alias = "session-eeAuth", rename(serialize = "session-eeAuth"))]
    pub session_ee_auth: WebApiEeAuth,

    #[serde(alias = "webApi-name", rename(serialize = "webApi-name"))]
    pub web_api_name: String,
    #[serde(alias = "webApi-eeAuth", rename(serialize = "webApi-eeAuth"))]
    pub web_api_ee_auth: WebApiEeAuth,
    #[serde(alias = "scmsV3-eeAuth", rename(serialize = "scmsV3-eeAuth"))]
    pub scms_v3_ee_auth: ScmsV3AuthConfig,
    #[serde(alias = "scmsV3-error", rename(serialize = "scmsV3-error"))]
    pub scms_v3_error: ScmsErrorLevel,
    #[serde(alias = "scmsV3-options", rename(serialize = "scmsV3-options"))]
    pub scms_v3_options: Vec<String>,

    #[serde(alias = "eca-maxAge", rename(serialize = "eca-maxAge"))]
    pub eca_max_age: u16,
    #[serde(alias = "eca-maxReqs", rename(serialize = "eca-maxReqs"))]
    pub eca_max_reqs: u16,
    #[serde(alias = "eca-maxWait", rename(serialize = "eca-maxWait"))]
    pub eca_max_wait: u16,
    #[serde(alias = "eca-minWait", rename(serialize = "eca-minWait"))]
    pub eca_min_wait: u16,

    #[serde(alias = "ra-acpcSupport", rename(serialize = "ra-acpcSupport"))]
    pub ra_acpc_support: Vec<String>,
    #[serde(alias = "ra-butterflyType", rename(serialize = "ra-butterflyType"))]
    pub ra_butterfly_type: RaButterflyTypeConfig,

    #[serde(alias = "ra-maxAge", rename(serialize = "ra-maxAge"))]
    pub ra_max_age: u16,
    #[serde(alias = "ra-maxGenDelay", rename(serialize = "ra-maxGenDelay"))]
    pub ra_max_gen_delay: u16,
    #[serde(alias = "ra-maxReqs", rename(serialize = "ra-maxReqs"))]
    pub ra_max_reqs: u16,
    #[serde(alias = "ra-maxPreloadTime", rename(serialize = "ra-maxPreloadTime"))]
    pub ra_max_preload_time: u16,
    #[serde(alias = "ra-minWait", rename(serialize = "ra-minWait"))]
    pub ra_min_wait: u16,
    #[serde(alias = "download-maxAge", rename(serialize = "download-maxAge"))]
    pub download_max_age: u16,
    #[serde(alias = "download-maxReqs", rename(serialize = "download-maxReqs"))]
    pub download_max_reqs: u16,
    #[serde(alias = "download-minWait", rename(serialize = "download-minWait"))]
    pub download_min_wait: u16,
    #[serde(
        alias = "time-identificationCertLength",
        rename(serialize = "time-identificationCertLength")
    )]
    pub time_identification_cert_length: u64,
    #[serde(
        alias = "time-identificationCertOverlap",
        rename(serialize = "time-identificationCertOverlap")
    )]
    pub time_identification_cert_overlap: u64,
    #[serde(alias = "time-leapSeconds", rename(serialize = "time-leapSeconds"))]
    pub time_leap_seconds: u64,
    #[serde(
        alias = "webApi-connectionRetries",
        rename(serialize = "webApi-connectionRetries")
    )]
    pub web_api_connection_retries: u16,
    #[serde(
        alias = "webApi-betweenRetries",
        rename(serialize = "webApi-betweenRetries")
    )]
    pub web_api_between_retries: u64,
    #[serde(
        alias = "webApi-betweenRetries-infoStatus",
        rename(serialize = "webApi-betweenRetries-infoStatus")
    )]
    pub web_api_between_retries_info_status: u64,
    #[serde(
        alias = "webApi-toleranceTime",
        rename(serialize = "webApi-toleranceTime")
    )]
    pub web_api_tolerance_time: u64,

    #[serde(
        alias = "nextUpdate-timeDefault",
        rename(serialize = "nextUpdate-timeDefault")
    )]
    pub next_update_time_default: NextUpdateTimeDefault,
    #[serde(
        alias = "supportRegion-obuDefault",
        rename(serialize = "supportRegion-obuDefault")
    )]
    pub support_region_obu_default: Vec<SupportRegionConfigInfo>,
    #[serde(
        alias = "supportRegion-rsuDefault",
        rename(serialize = "supportRegion-rsuDefault")
    )]
    pub support_region_rsu_default: Vec<SupportRegionConfigInfo>,
    #[serde(
        alias = "supportPsidSsp-obuDefault",
        rename(serialize = "supportPsidSsp-obuDefault")
    )]
    pub support_psid_ssp_obu_default: Vec<PsidConfigInfo>,
    #[serde(
        alias = "supportPsidSsp-rsuDefault",
        rename(serialize = "supportPsidSsp-rsuDefault")
    )]
    pub support_psid_ssp_rsu_default: Vec<PsidConfigInfo>,

    #[serde(
        alias = "scmsV3-eeAuth-default",
        rename(serialize = "scmsV3-eeAuth-default")
    )]
    pub scms_v3_ee_auth_default: ScmsV3AuthConfig,

    #[serde(
        alias = "ra-butterflyType-default",
        rename(serialize = "ra-butterflyType-default")
    )]
    pub ra_butterfly_type_default: RaButterflyType,
    #[serde(
        alias = "ra-encryptionKey-default",
        rename(serialize = "ra-encryptionKey-default")
    )]
    pub ra_encryption_key_default: String,
    #[serde(
        alias = "download-url-default",
        rename(serialize = "download-url-default")
    )]
    pub download_url_default: String,
    #[serde(
        alias = "count-certPseudonym-default",
        rename(serialize = "count-certPseudonym-default")
    )]
    pub count_cert_pseudonym_default: u64,
    #[serde(
        alias = "count-certIdentification-default",
        rename(serialize = "count-certIdentification-default")
    )]
    pub count_cert_identification_default: u64,
    #[serde(
        alias = "ee-heartbeatInterval",
        rename(serialize = "ee-heartbeatInterval")
    )]
    pub ee_heartbeat_interval: u64,
}

impl GlobalConfigInfo {
    pub fn from_global_config_or_default() -> Self {
        let global_config = GlobalConfig::global();

        let default_cert_info = CertificateConfigInfo {
            name: "".to_string(),
            crypto_algorithm: vec![CryptoAlgorithmType::NistP256, CryptoAlgorithmType::Sha256],
            certificate_type: CertificateType::Explicit,
            start: 0,
            expiration: 0,
            in_use: 0,
            request_for_renewal: 0,
            concurrently_valid_certificates: 1,
        };

        let default_scms_v3_auth = ScmsV3AuthConfig {
            enrollment_certificate: vec![ScmsEeAuth::Canonical],
            authorization_certificate_request: vec![ScmsEeAuth::Enrollment],
            authorization_certificate_download: vec![ScmsEeAuth::Enrollment],
            authorization_certificate_download_filename: vec![ScmsEeAuth::Enrollment],
            ccf_ctl: vec![ScmsEeAuth::NoAuth],
            composite_crl_ctl: vec![ScmsEeAuth::NoAuth],
            ca_certificate: vec![ScmsEeAuth::NoAuth],
            crl: vec![ScmsEeAuth::NoAuth],
            ctl: vec![ScmsEeAuth::NoAuth],
            ra_certificate: vec![ScmsEeAuth::NoAuth],
            certificate_management_info_status: vec![ScmsEeAuth::NoAuth],
            successor_enrollment_certificate_request: vec![ScmsEeAuth::Enrollment],
            successor_enrollment_certificate_download: vec![ScmsEeAuth::Enrollment],
            misbehavior_report: vec![ScmsEeAuth::Authorization],
            ma_certificate: vec![ScmsEeAuth::NoAuth],
        };

        GlobalConfigInfo {
            quorum_count: 3,
            ctl_series_id: global_config.ctl_series_id.clone(),
            crypto_algorithm: vec![
                CryptoAlgorithmType::NistP256,
                CryptoAlgorithmType::NistP384,
                CryptoAlgorithmType::BrainpoolP256,
                CryptoAlgorithmType::BrainpoolP384,
                CryptoAlgorithmType::Sha256,
                CryptoAlgorithmType::Sha384,
                CryptoAlgorithmType::Aes128Ccm,
            ],
            cert_elector: CertificateConfigInfo {
                name: "elector".to_string(),
                crypto_algorithm: vec![CryptoAlgorithmType::NistP384, CryptoAlgorithmType::Sha384],
                certificate_type: CertificateType::Explicit,
                start: 0,
                expiration: 0,
                in_use: 0,
                request_for_renewal: 0,
                concurrently_valid_certificates: 1,
            },
            cert_root_ca: {
                let mut default = default_cert_info.clone();
                default.name = "root-ca".to_string();
                default
            },
            cert_ica: {
                let mut default = default_cert_info.clone();
                default.name = "ica".to_string();
                default
            },
            cert_eca: {
                let mut default = default_cert_info.clone();
                default.name = "eca".to_string();
                default
            },
            cert_aca: {
                let mut default = default_cert_info.clone();
                default.name = "aca".to_string();
                default
            },
            cert_pca: {
                let mut default = default_cert_info.clone();
                default.name = "pca".to_string();
                default
            },
            cert_crl_signer: {
                let mut default = default_cert_info.clone();
                default.name = "crl-signer".to_string();
                default
            },
            cert_ra: {
                let mut default = default_cert_info.clone();
                default.name = "ra".to_string();
                default
            },
            cert_la: {
                let mut default = default_cert_info.clone();
                default.name = "la".to_string();
                default
            },
            cert_la1: {
                let mut default = default_cert_info.clone();
                default.name = "la1".to_string();
                default
            },
            cert_la2: {
                let mut default = default_cert_info.clone();
                default.name = "la2".to_string();
                default
            },
            cert_ma: {
                let mut default = default_cert_info.clone();
                default.name = "ma".to_string();
                default
            },
            cert_pg: {
                let mut default = default_cert_info.clone();
                default.name = "pg".to_string();
                default
            },
            cert_dc: {
                let mut default = default_cert_info.clone();
                default.name = "dc".to_string();
                default
            },
            cert_enrollment: {
                let mut default = default_cert_info.clone();
                default.certificate_type = CertificateType::Implicit;
                default
            },
            cert_pseudonym: {
                let mut default = default_cert_info.clone();
                default.certificate_type = CertificateType::Implicit;
                default
            },
            cert_identification: {
                let mut default = default_cert_info.clone();
                default.certificate_type = CertificateType::Implicit;
                default
            },
            support_region: vec![RegionConfigInfo {
                country: ["840", "124", "484", "410", "724", "156"]
                    .iter()
                    .map(|s| s.to_string())
                    .collect(),
                country_and_region: vec![],
                country_and_region_and_sub_region: vec![],
                circle: vec![],
            }],
            support_psid_ssp: vec![PsidConfigInfo { all: vec![] }],
            url_domain: "192.168.49.2".to_string(),
            eca_hostname: "192.168.49.2:30085".to_string(),
            ra_hostname: "192.168.49.2:30080".to_string(),
            dc_hostname: "".to_string(),
            dcm_hostname: "".to_string(),
            eca_url: "http://192.168.49.2:30085/v3/".to_string(),
            ra_url: "http://192.168.49.2:30080/v3/".to_string(),
            dc_url: "http://192.168.49.2:30080/v3/".to_string(),
            dcm_url: "http://192.168.49.2:30080/".to_string(),
            time_i_period: get_current_time_period_days() as u64,
            time_i_period_epoch: global_config.time_i_period_epoch,
            time_i_period_init: global_config.time_i_period_init,
            time_i_period_length: (global_config.period_length_days as u64) * 24 * 60 * 60,
            time_overlap: 0,
            time_base_cert_number: global_config.certificates_per_batch,
            time_overlap_cert_number: 0,
            session_scms_auth: "none".to_string(),
            session_ee_auth: WebApiEeAuth::NoAuth,
            web_api_name: global_config.param_webapi_name.clone(),
            web_api_ee_auth: global_config.param_webapi_ee_auth.clone(),
            scms_v3_ee_auth: default_scms_v3_auth.clone(),
            scms_v3_error: global_config.param_scmsv3_error,
            scms_v3_options: vec![global_config.param_scmsv3_options.clone()],
            eca_max_age: global_config.param_eca_max_age,
            eca_max_reqs: global_config.param_eca_max_reqs,
            eca_max_wait: global_config.param_eca_max_wait,
            eca_min_wait: global_config.param_eca_min_wait,
            ra_acpc_support: vec![],
            ra_butterfly_type: RaButterflyTypeConfig {
                original: ["32", "38"].iter().map(|s| s.to_string()).collect(),
                unified: vec![],
                compact_unified: vec![],
            },
            ra_max_age: global_config.param_ra_max_age,
            ra_max_gen_delay: global_config.param_ra_max_gen_delay,
            ra_max_reqs: global_config.param_ra_max_reqs,
            ra_max_preload_time: global_config.param_ra_max_reload_time,
            ra_min_wait: global_config.param_ra_min_wait,
            download_max_age: global_config.param_download_max_age,
            download_min_wait: global_config.param_download_min_wait,
            download_max_reqs: 288,
            time_identification_cert_length: 0,
            time_identification_cert_overlap: 0,
            time_leap_seconds: 0,
            web_api_connection_retries: 3,
            web_api_between_retries: 1000,
            web_api_between_retries_info_status: 1000,
            web_api_tolerance_time: 1000,
            next_update_time_default: NextUpdateTimeDefault {
                full_hash_crl: 0,
                delta_hash_crl: 0,
                full_linked_crl: 0,
                delta_linked_crl: 0,
                full_root_ctl: 0,
                delta_root_ctl: 0,
                full_domain_ctl: 0,
                delta_domain_ctl: 0,
            },
            support_region_obu_default: vec![SupportRegionConfigInfo {
                region: vec![RegionConfigInfo {
                    country: vec![],
                    country_and_region: vec![],
                    country_and_region_and_sub_region: vec![],
                    circle: vec![],
                }],
            }],
            support_region_rsu_default: vec![SupportRegionConfigInfo {
                region: vec![RegionConfigInfo {
                    country: vec![],
                    country_and_region: vec![],
                    country_and_region_and_sub_region: vec![],
                    circle: vec![],
                }],
            }],
            support_psid_ssp_obu_default: vec![PsidConfigInfo { all: vec![] }],
            support_psid_ssp_rsu_default: vec![PsidConfigInfo { all: vec![] }],
            scms_v3_ee_auth_default: default_scms_v3_auth,
            ra_butterfly_type_default: RaButterflyType::Original,
            ra_encryption_key_default: "encryptionKey".to_string(),
            download_url_default: "".to_string(),
            count_cert_pseudonym_default: global_config.number_cert_batches,
            count_cert_identification_default: 1,
            ee_heartbeat_interval: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone, ToSchema)]
pub struct DeviceConfigInfo {
    #[serde(alias = "ee-scmsV3-eeAuth", rename(serialize = "ee-scmsV3-eeAuth"))]
    pub ee_scms_v3_auth: ScmsV3AuthConfig,

    #[serde(
        alias = "ee-ra-butterflyType",
        rename(serialize = "ee-ra-butterflyType")
    )]
    pub ee_ra_butterfly_type: RaButterflyType,

    #[serde(alias = "ee-download-url", rename(serialize = "ee-download-url"))]
    pub ee_download_url: String,

    #[serde(alias = "ee-supportPsidSsp", rename(serialize = "ee-supportPsidSsp"))]
    pub ee_support_psid_ssp: Vec<PsidConfigInfo>,

    #[serde(alias = "ee-supportRegion", rename(serialize = "ee-supportRegion"))]
    pub ee_support_region: Vec<SupportRegionConfigInfo>,

    #[serde(alias = "ee-countCert", rename(serialize = "ee-countCert"))]
    pub ee_count_cert_pseudonym: u16,

    #[serde(
        alias = "ee-heartbeatInterval",
        rename(serialize = "ee-heartbeatInterval")
    )]
    pub ee_heartbeat_interval: u64,
}

impl DeviceConfigInfo {
    pub fn from_config_info() -> Self {
        let global_config = GlobalConfigInfo::from_global_config_or_default();
        DeviceConfigInfo {
            ee_scms_v3_auth: global_config.scms_v3_ee_auth_default.clone(),
            ee_ra_butterfly_type: global_config.ra_butterfly_type_default.clone(),
            ee_download_url: global_config.download_url_default.clone(),
            ee_support_psid_ssp: global_config.support_psid_ssp_obu_default.clone(),
            ee_support_region: global_config.support_region_obu_default.clone(),
            ee_count_cert_pseudonym: global_config.count_cert_pseudonym_default as u16,
            ee_heartbeat_interval: global_config.ee_heartbeat_interval,
        }
    }
}
