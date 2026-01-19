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

use openssl::error::ErrorStack;
use std::fmt;

#[derive(Debug)]
pub struct GenSharedSecretError {
    message: String,
}

impl GenSharedSecretError {
    pub fn new(message: &str) -> Self {
        GenSharedSecretError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for GenSharedSecretError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error generating shared secret: {}", self.message)
    }
}

impl From<ErrorStack> for GenSharedSecretError {
    fn from(err: ErrorStack) -> Self {
        GenSharedSecretError::new(&format!("openssl ErrorStack: {}", err))
    }
}

#[derive(Debug)]
pub struct EncryptPlvError {
    message: String,
}

impl EncryptPlvError {
    pub fn new(message: &str) -> Self {
        EncryptPlvError {
            message: message.to_owned(),
        }
    }
}
impl fmt::Display for EncryptPlvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in encrypt_plv: {}", self.message)
    }
}

impl From<ErrorStack> for EncryptPlvError {
    fn from(err: ErrorStack) -> Self {
        EncryptPlvError::new(&format!("openssl ErrorStack: {}", err))
    }
}

impl From<GenSharedSecretError> for EncryptPlvError {
    fn from(err: GenSharedSecretError) -> Self {
        EncryptPlvError::new(&format!("GenSharedSecretError: {}", err))
    }
}

#[derive(Debug)]
pub struct ProcessPlvRequestError {
    pub message: String,
}

impl ProcessPlvRequestError {
    pub fn new(message: &str) -> Self {
        ProcessPlvRequestError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for ProcessPlvRequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in process_plv_request: {}", self.message)
    }
}

impl From<EncryptPlvError> for ProcessPlvRequestError {
    fn from(err: EncryptPlvError) -> Self {
        ProcessPlvRequestError::new(&format!("EncryptPlvError: {}", err))
    }
}

#[derive(Debug)]
pub struct FindLsError {
    message: String,
}

impl FindLsError {
    pub fn new(message: &str) -> Self {
        FindLsError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for FindLsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FindLsError: {}", self.message)
    }
}

impl From<sea_orm::DbErr> for FindLsError {
    fn from(err: sea_orm::DbErr) -> Self {
        FindLsError::new(&format!("sea_orm::DbErr: {}", err))
    }
}
