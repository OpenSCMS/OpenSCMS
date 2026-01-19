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
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RaConfig {
    pub min_number_certs_ra: u16,
    pub rabbitmq_connection_string: String,
    pub ra_id: u64,
}

impl RaConfig {
    pub fn from_envy() -> Self {
        let config = envy::from_env::<RaConfig>();
        match config {
            Ok(config) => config,
            Err(e) => panic!("Fatal error loading global configuration: {:#?}", e),
        }
    }

    pub fn global() -> &'static RaConfig {
        RA_CONFIG.get().expect("RA_CONFIG has not been set")
    }
}

static RA_CONFIG: OnceCell<RaConfig> = OnceCell::new();

pub fn load_ra_config() -> &'static RaConfig {
    let config = RaConfig::from_envy();
    RA_CONFIG.set(config).unwrap();
    RaConfig::global()
}
