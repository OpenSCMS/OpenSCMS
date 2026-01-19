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
use oscms_bridge::OscmsBridgeError;
use scmscommon::errors::ConversionError;
use std::fmt;

#[derive(Debug)]
pub struct DecryptError {
    message: String,
}

impl DecryptError {
    pub fn new(message: &str) -> Self {
        DecryptError {
            message: message.to_owned(),
        }
    }
}

impl From<aes_gcm::Error> for DecryptError {
    fn from(err: aes_gcm::Error) -> Self {
        DecryptError::new(&format!("aes_gcm::Error: {}", err))
    }
}

impl From<DeriveSharedSecretError> for DecryptError {
    fn from(err: DeriveSharedSecretError) -> Self {
        DecryptError::new(&format!("DeriveSharedSecretError: {:?}", err))
    }
}

impl From<ConversionError> for DecryptError {
    fn from(err: ConversionError) -> Self {
        DecryptError::new(&format!("ConversionError: {}", err))
    }
}

impl fmt::Display for DecryptError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct DeriveSharedSecretError {
    message: String,
}

impl DeriveSharedSecretError {
    pub fn new(message: &str) -> Self {
        DeriveSharedSecretError {
            message: message.to_owned(),
        }
    }
}

impl From<ErrorStack> for DeriveSharedSecretError {
    fn from(err: ErrorStack) -> Self {
        DeriveSharedSecretError::new(&format!("openssl ErrorStack: {}", err))
    }
}

impl fmt::Display for DeriveSharedSecretError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[derive(Debug)]
pub struct GenCertificateError {
    message: String,
}

impl GenCertificateError {
    pub fn new(message: &str) -> Self {
        GenCertificateError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for GenCertificateError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<ConversionError> for GenCertificateError {
    fn from(err: ConversionError) -> Self {
        GenCertificateError::new(&format!("ConversionError: {}", err))
    }
}

impl From<DecryptError> for GenCertificateError {
    fn from(err: DecryptError) -> Self {
        GenCertificateError::new(&format!("DecryptError: {:?}", err))
    }
}

impl From<OscmsBridgeError> for GenCertificateError {
    fn from(err: OscmsBridgeError) -> Self {
        GenCertificateError::new(&format!("OscmsBridgeError: {:?}", err))
    }
}

#[derive(Debug)]
pub struct FindPlvError {
    message: String,
}

impl FindPlvError {
    pub fn new(message: &str) -> Self {
        FindPlvError {
            message: message.to_owned(),
        }
    }
}

impl fmt::Display for FindPlvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<sea_orm::DbErr> for FindPlvError {
    fn from(err: sea_orm::DbErr) -> Self {
        FindPlvError::new(&format!("sea_orm::DbErr: {}", err))
    }
}

impl From<ConversionError> for FindPlvError {
    fn from(err: ConversionError) -> Self {
        FindPlvError::new(&format!("ConversionError: {}", err))
    }
}
