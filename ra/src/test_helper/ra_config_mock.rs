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

use std::sync::Once;

use crate::raconfig::load_ra_config;

#[allow(dead_code)]
static SETUP_RA_CONF: Once = Once::new();

#[allow(dead_code)]
pub fn setup_ra_conf() {
    SETUP_RA_CONF.call_once(|| {
        load_ra_config();
    });
}
