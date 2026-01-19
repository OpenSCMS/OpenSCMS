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

use crate::errors::ConversionError;
use core::fmt;

#[derive(Debug)]
pub struct PersistenceLoadError {
    pub message: String,
}

#[derive(Debug)]
pub struct PersistenceStoreError {
    pub message: String,
}

impl PersistenceStoreError {
    pub fn new(message: &str) -> Self {
        PersistenceStoreError {
            message: message.to_owned(),
        }
    }
}

impl PersistenceLoadError {
    pub fn new(message: &str) -> Self {
        PersistenceLoadError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for PersistenceStoreError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl fmt::Display for PersistenceLoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<sea_orm::DbErr> for PersistenceLoadError {
    fn from(err: sea_orm::DbErr) -> Self {
        PersistenceLoadError::new(&format!("Database error: {}", err))
    }
}

impl From<serde_json::Error> for PersistenceLoadError {
    fn from(err: serde_json::Error) -> Self {
        PersistenceLoadError::new(&format!("JSON error: {}", err))
    }
}

impl From<ConversionError> for PersistenceLoadError {
    fn from(err: ConversionError) -> Self {
        PersistenceLoadError::new(&format!("Conversion error: {}", err))
    }
}

impl From<serde_json::Error> for PersistenceStoreError {
    fn from(err: serde_json::Error) -> Self {
        PersistenceStoreError::new(&format!("JSON error: {}", err))
    }
}

impl From<sea_orm::DbErr> for PersistenceStoreError {
    fn from(err: sea_orm::DbErr) -> Self {
        PersistenceStoreError::new(&format!("Database error: {}", err))
    }
}
