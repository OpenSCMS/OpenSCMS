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

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// By default, all services listen on port 5000
#[allow(dead_code)] // referenced by string
fn default_service_port() -> u16 {
    5000
}

#[allow(dead_code)]
fn default_la_prefix() -> String {
    "la".to_string()
}

#[allow(dead_code)]
fn default_aca_prefix() -> String {
    "aca".to_string()
}

#[allow(dead_code)]
fn default_cam_prefix() -> String {
    "cam".to_string()
}

#[allow(dead_code)]
fn default_eca_prefix() -> String {
    "eca".to_string()
}

#[allow(dead_code)]
fn default_ra_prefix() -> String {
    "ra".to_string()
}

#[allow(dead_code)]
fn default_post_endpoint() -> String {
    "/".to_string()
}

#[allow(dead_code)]
fn default_post_ee_registration() -> String {
    "/dcms/device/na".to_string()
}

#[allow(dead_code)]
fn default_aca_certs_folder() -> String {
    "/certs/aca".to_string()
}

#[allow(dead_code)]
fn default_eca_certs_folder() -> String {
    "/certs/eca".to_string()
}

#[allow(dead_code)]
fn default_ica_certs_folder() -> String {
    "/certs/ica".to_string()
}

#[allow(dead_code)]
fn default_rootca_certs_folder() -> String {
    "/certs/rootca".to_string()
}

#[allow(dead_code)]
fn default_ra_certs_folder() -> String {
    "/certs/ra".to_string()
}

#[allow(dead_code)]
fn default_aca_certificate_file() -> String {
    "acaCertificate.coer".to_string()
}

#[allow(dead_code)]
fn default_aca_public_key_file() -> String {
    "acaPublicKey.dat".to_string()
}

#[allow(dead_code)]
fn default_aca_private_key_file() -> String {
    "acaPrivateKey.dat".to_string()
}

#[allow(dead_code)]
fn default_eca_certificate_file() -> String {
    "ecaCertificate.coer".to_string()
}

#[allow(dead_code)]
fn default_eca_public_key_file() -> String {
    "ecaPublicKey.dat".to_string()
}

#[allow(dead_code)]
fn default_eca_private_key_file() -> String {
    "ecaPrivateKey.dat".to_string()
}

#[allow(dead_code)]
fn default_ica_certificate_file() -> String {
    "icaCertificate.coer".to_string()
}

#[allow(dead_code)]
fn default_rootca_certificate_file() -> String {
    "rootCaCertificate.coer".to_string()
}

#[allow(dead_code)]
fn default_ra_certificate_file() -> String {
    "raCertificate.coer".to_string()
}

#[allow(dead_code)]
fn default_ra_public_key_file() -> String {
    "raPublicKey.dat".to_string()
}

#[allow(dead_code)]
fn default_ra_private_key_file() -> String {
    "raPrivateKey.dat".to_string()
}

#[allow(dead_code)]
fn default_ra_enc_private_key_file() -> String {
    "raEncPrivateKey.dat".to_string()
}

#[allow(dead_code)]
fn default_ra_enc_public_key_file() -> String {
    "raEncPublicKey.dat".to_string()
}

#[allow(dead_code)]
fn default_ctl_series_id() -> String {
    "FFFFFFFFFFFFFF00".to_string()
}

#[allow(dead_code)]
fn default_ctl_folder() -> String {
    "/certs/ctl".to_string()
}

#[allow(dead_code)]
fn default_ctl_file() -> String {
    "ctl.coer".to_string()
}

#[allow(dead_code)]
fn default_ctl_ccf_file() -> String {
    "ccf.coer".to_string()
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, ToSchema)]
pub enum WebApiEeAuth {
    #[serde(alias = "None")]
    NoAuth,
    #[serde(alias = "oAuth2.0")]
    OAuth2,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Copy, Clone, ToSchema)]
pub enum ScmsEeAuth {
    #[serde(alias = "authorization")]
    Authorization,
    #[serde(alias = "canonical")]
    Canonical,
    #[serde(alias = "enrollment")]
    Enrollment,
    #[serde(alias = "None")]
    NoAuth,
    #[serde(alias = "x509")]
    X509,
}

#[derive(Deserialize, Serialize, Debug, Eq, PartialEq, Clone, Copy, ToSchema)]
pub enum ScmsErrorLevel {
    #[serde(alias = "coarse")]
    Coarse,
    #[serde(alias = "fine")]
    Fine,
}

#[derive(Deserialize, Serialize, Debug, PartialEq, Clone, ToSchema)]
pub enum RaButterflyType {
    #[serde(alias = "original", alias = "OBK")]
    Original,
    #[serde(alias = "unified", alias = "UBK")]
    Unified,
    #[serde(alias = "compact unified", alias = "CUBK")]
    CompactUnified,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Disable "never read" warning for all fields
pub struct GlobalConfig {
    pub certificates_per_batch: u64,
    pub period_length_days: u16,
    pub deploy_environment: String,

    #[serde(default = "default_ctl_series_id")]
    pub ctl_series_id: String,

    pub la_count: u16,
    #[serde(default = "default_service_port")]
    pub la_port: u16,
    #[serde(default = "default_la_prefix")]
    pub la_prefix: String,
    #[serde(default = "default_post_endpoint")]
    pub la_post_endpoint: String,

    #[serde(default = "default_service_port")]
    pub ra_port: u16,
    #[serde(default = "default_ra_prefix")]
    pub ra_prefix: String,
    #[serde(default = "default_post_ee_registration")]
    pub ra_post_ee_registration: String,

    #[serde(default = "default_service_port")]
    pub aca_port: u16,
    #[serde(default = "default_aca_prefix")]
    pub aca_prefix: String,
    #[serde(default = "default_post_endpoint")]
    pub aca_post_endpoint: String,

    #[serde(default = "default_service_port")]
    pub cam_port: u16,
    #[serde(default = "default_cam_prefix")]
    pub cam_prefix: String,
    #[serde(default = "default_post_endpoint")]
    pub cam_post_endpoint: String,

    #[serde(default = "default_service_port")]
    pub eca_port: u16,
    #[serde(default = "default_eca_prefix")]
    pub eca_prefix: String,
    #[serde(default = "default_post_endpoint")]
    pub eca_post_endpoint: String,

    // System Parameters
    pub param_webapi_name: String,
    pub param_webapi_ee_auth: WebApiEeAuth,

    pub param_scmsv3_ee_auth: ScmsEeAuth,
    pub param_scmsv3_error: ScmsErrorLevel,
    pub param_scmsv3_options: String,

    pub param_eca_max_age: u16,
    pub param_eca_max_reqs: u16,
    pub param_eca_max_wait: u16,
    pub param_eca_min_wait: u16,

    param_ra_acpc_support: bool,
    param_ra_butterfly_type: RaButterflyType,
    pub param_ra_max_age: u16,
    pub param_ra_max_gen_delay: u16,
    pub param_ra_max_reqs: u16,
    pub param_ra_max_reload_time: u16,
    pub param_ra_min_wait: u16,

    pub param_download_max_age: u16,
    pub param_download_min_wait: u16,

    pub param_successor_next_dl_time: u16,
    pub param_cert_next_dl_time: u16,

    #[serde(default = "default_aca_certs_folder")]
    pub aca_certs_folder: String,
    #[serde(default = "default_aca_certificate_file")]
    pub aca_certificate_file: String,
    #[serde(default = "default_aca_public_key_file")]
    pub aca_public_key_file: String,
    #[serde(default = "default_aca_private_key_file")]
    pub aca_private_key_file: String,

    #[serde(default = "default_eca_certs_folder")]
    pub eca_certs_folder: String,
    #[serde(default = "default_eca_certificate_file")]
    pub eca_certificate_file: String,
    #[serde(default = "default_eca_public_key_file")]
    pub eca_public_key_file: String,
    #[serde(default = "default_eca_private_key_file")]
    pub eca_private_key_file: String,

    #[serde(default = "default_ica_certs_folder")]
    pub ica_certs_folder: String,
    #[serde(default = "default_ica_certificate_file")]
    pub ica_certificate_file: String,
    #[serde(default = "default_rootca_certs_folder")]
    pub rootca_certs_folder: String,
    #[serde(default = "default_rootca_certificate_file")]
    pub rootca_certificate_file: String,

    #[serde(default = "default_ra_certs_folder")]
    pub ra_certs_folder: String,
    #[serde(default = "default_ra_certificate_file")]
    pub ra_certificate_file: String,
    #[serde(default = "default_ra_public_key_file")]
    pub ra_public_key_file: String,
    #[serde(default = "default_ra_private_key_file")]
    pub ra_private_key_file: String,
    #[serde(default = "default_ra_enc_public_key_file")]
    pub ra_enc_public_key_file: String,
    #[serde(default = "default_ra_enc_private_key_file")]
    pub ra_enc_private_key_file: String,

    #[serde(default = "default_ctl_folder")]
    pub ctl_folder: String,
    #[serde(default = "default_ctl_file")]
    pub ctl_file: String,
    #[serde(default = "default_ctl_ccf_file")]
    pub ctl_ccf_file: String,

    pub number_cert_batches: u64,
    pub time_i_period_epoch: u64,
    pub time_i_period_init: u64,
}

impl GlobalConfig {
    pub fn from_envy() -> Self {
        let config = envy::from_env::<GlobalConfig>();
        match config {
            Ok(config) => config,
            Err(e) => panic!("Fatal error loading global configuration: {:#?}", e),
        }
    }

    pub fn global() -> &'static GlobalConfig {
        GLOBAL_CONFIG.get().expect("GLOBAL_CONFIG has not been set")
    }

    pub fn la_addr(&self, la_id: u16) -> String {
        format!("http://{}{}", self.la_prefix, la_id)
    }

    pub fn cam_addr(&self) -> String {
        format!("http://{}", self.cam_prefix)
    }

    pub fn eca_addr(&self) -> String {
        format!("http://{}", self.eca_prefix)
    }

    pub fn aca_addr(&self) -> String {
        format!("http://{}", self.aca_prefix)
    }

    pub fn ra_addr(&self) -> String {
        format!("http://{}", self.ra_prefix)
    }
}

static GLOBAL_CONFIG: OnceCell<GlobalConfig> = OnceCell::new();

pub fn load_global_config() -> &'static GlobalConfig {
    let config = GlobalConfig::from_envy();
    GLOBAL_CONFIG.set(config).unwrap();
    GlobalConfig::global()
}
