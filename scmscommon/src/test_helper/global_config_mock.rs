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

use crate::load_global_config;
use port_selector::select_free_port;
use std::env;
use std::sync::Once;

#[allow(dead_code)]
static SETUP_CONF: Once = Once::new();

#[warn(dead_code)]
pub fn setup_global_config() {
    let random_la_port = select_free_port(Default::default());
    let random_aca_port = select_free_port(Default::default());

    if random_la_port.is_none() || random_aca_port.is_none() {
        panic!("Failed to select free ports for LA or ACA");
    }

    unsafe {
        env::set_var("ACA_PREFIX", "127.0.0.11");
        env::set_var("ACA_PORT", random_aca_port.unwrap().to_string());

        env::set_var("LA_PREFIX", "127.0.0.");
        env::set_var("LA_PORT", random_la_port.unwrap().to_string());
    };

    SETUP_CONF.call_once(|| {
        let _global_config = load_global_config();
    });
}
