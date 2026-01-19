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

use log::debug;
use once_cell::sync::OnceCell;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MysqlConfig {
    // These environment variables are defined in the infra-mysql kubernetes secret.
    // The first four are marked as allow(dead_code) as they are rarely used.
    #[allow(dead_code)]
    mysql_username: String,
    #[allow(dead_code)]
    mysql_password: String,
    #[allow(dead_code)]
    mysql_port: u16,
    #[allow(dead_code)]
    mysql_host: String,

    mysql_connection_string: String,

    // Database name is defined for each service
    mysql_database_name: String,
}

impl MysqlConfig {
    pub fn from_envy() -> Self {
        let config = envy::from_env::<MysqlConfig>();
        match config {
            Ok(config) => config,
            Err(e) => panic!("Fatal error loading global configuration: {:#?}", e),
        }
    }

    pub fn global() -> &'static MysqlConfig {
        MYSQL_CONFIG.get().expect("MYSQL_CONFIG has not been set")
    }
}

static MYSQL_CONFIG: OnceCell<MysqlConfig> = OnceCell::new();

pub fn load_mysql_config() -> &'static MysqlConfig {
    let config = MysqlConfig::from_envy();
    debug!("{:#?}", config);
    MYSQL_CONFIG.set(config).unwrap();

    MysqlConfig::global()
}

// Return the full connection string  including url, username, password, and database name
pub fn connection_string() -> String {
    let config = MysqlConfig::global();
    format!(
        "{}/{}",
        config.mysql_connection_string, config.mysql_database_name
    )
}

pub fn get_db_name() -> &'static str {
    let config = MysqlConfig::global();
    config.mysql_database_name.as_str()
}

pub fn get_url() -> &'static str {
    let config = MysqlConfig::global();
    config.mysql_connection_string.as_str()
}
