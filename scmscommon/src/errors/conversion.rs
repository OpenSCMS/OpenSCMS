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

use core::fmt;

#[derive(Debug)]
pub struct ConversionError {
    message: String,
}

impl ConversionError {
    pub fn new(message: &str) -> Self {
        ConversionError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<aes_gcm::Error> for ConversionError {
    fn from(err: aes_gcm::Error) -> Self {
        ConversionError::new(&format!("aes_gcm::Error: {}", err))
    }
}
