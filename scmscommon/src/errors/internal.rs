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

#[derive(Debug)]
pub enum InternalCommWire {
    RaToLa,
    RaToCam,
    RaToAca,
    RaToEca,
    LaToRa,
    CamToRa,
    AcaToRa,
    EcaToRa,
    RaSetup,
    NotDefined,
}

#[derive(Debug)]
pub struct ScmsInternalCommError {
    pub message: String,
    pub wire: InternalCommWire,
    pub error_code: u16,
    pub detailed_code: Option<i32>,
}

impl ScmsInternalCommError {
    pub fn new(message: &str, wire: InternalCommWire, error_code: u16) -> Self {
        ScmsInternalCommError {
            message: message.to_owned(),
            wire,
            error_code,
            detailed_code: None,
        }
    }

    pub fn set_detailed_code(&mut self, detailed_code: Option<i32>) {
        self.detailed_code = detailed_code;
    }
}

impl From<serde_json::Error> for ScmsInternalCommError {
    fn from(err: serde_json::Error) -> Self {
        ScmsInternalCommError::new(
            &format!("JSON error: {}", err),
            InternalCommWire::NotDefined,
            500,
        )
    }
}

impl From<reqwest::Error> for ScmsInternalCommError {
    fn from(err: reqwest::Error) -> Self {
        ScmsInternalCommError::new(
            &format!("Request error: {}", err),
            InternalCommWire::NotDefined,
            500,
        )
    }
}

impl From<std::io::Error> for ScmsInternalCommError {
    fn from(err: std::io::Error) -> Self {
        ScmsInternalCommError::new(
            &format!("IO Error: {}", err),
            InternalCommWire::NotDefined,
            500,
        )
    }
}
