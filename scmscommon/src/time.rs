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

use crate::GlobalConfig;
use std::time::{SystemTime, UNIX_EPOCH};

// 00:00:00 UTC, 1 January, 2004.
const INITIAL_EPOCH_2004_01_01_00_00_00: u32 = 1072915200;

pub fn get_current_time_seconds() -> u32 {
    let epoch_now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as u32;

    epoch_now - INITIAL_EPOCH_2004_01_01_00_00_00
}

pub fn get_current_time_period_days() -> u32 {
    let current_time_seconds = get_current_time_seconds();
    current_time_seconds / (60 * 60 * 24 * GlobalConfig::global().period_length_days as u32)
}
