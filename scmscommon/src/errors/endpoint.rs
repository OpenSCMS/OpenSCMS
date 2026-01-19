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

use crate::endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes;
use crate::errors::{PersistenceLoadError, PersistenceStoreError};
use actix_web::{HttpResponse, HttpResponseBuilder, http::StatusCode};
use core::fmt;
use sea_orm;

#[derive(Debug)]
pub struct HandleResponseError {
    pub message: String,
    pub status_code: StatusCode,
    pub error_code: Ieee1609Dot2Dot1ErrorCodes,
}

impl HandleResponseError {
    pub fn new(
        message: &str,
        status_code: StatusCode,
        error_code: Ieee1609Dot2Dot1ErrorCodes,
    ) -> Self {
        HandleResponseError {
            message: message.to_owned(),
            status_code,
            error_code,
        }
    }

    pub fn ieee_1609dot2dot1_error_header(&self) -> (String, String) {
        (
            "Ieee-1609.2.1-Error".to_string(),
            self.error_code.to_string(),
        )
    }

    pub fn http_error_response(&self) -> HttpResponse {
        let mut response = HttpResponseBuilder::new(self.status_code);

        if self.status_code == StatusCode::BAD_REQUEST || self.status_code == StatusCode::FORBIDDEN
        {
            response.append_header(self.ieee_1609dot2dot1_error_header());
        }

        response.body(self.message.clone())
    }
}

impl fmt::Display for HandleResponseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl From<PersistenceLoadError> for HandleResponseError {
    fn from(err: PersistenceLoadError) -> Self {
        HandleResponseError::new(
            &format!("PersistenceLoadError: {}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    }
}

impl From<PersistenceStoreError> for HandleResponseError {
    fn from(err: PersistenceStoreError) -> Self {
        HandleResponseError::new(
            &format!("PersistenceStoreError: {}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    }
}

impl From<sea_orm::DbErr> for HandleResponseError {
    fn from(err: sea_orm::DbErr) -> Self {
        HandleResponseError::new(
            &format!("Database Error: {}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    }
}

impl From<Box<dyn std::error::Error>> for HandleResponseError {
    fn from(err: Box<dyn std::error::Error>) -> Self {
        HandleResponseError::new(
            &format!("Std Error: {}", err),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::http::header::HeaderValue;

    #[test]
    fn test_handle_response_error() {
        let error = HandleResponseError::new(
            "Test error message",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                crate::errors::endpoint_error_codes::BadRequestErrorCodes::AbsentEncryption,
            ),
        );

        assert_eq!(error.message, "Test error message");
        assert_eq!(error.status_code, StatusCode::BAD_REQUEST);
        assert_eq!(
            error.error_code,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                crate::errors::endpoint_error_codes::BadRequestErrorCodes::AbsentEncryption,
            )
        );
    }

    #[test]
    fn test_ieee_1609dot2dot1_error_header() {
        let error = HandleResponseError::new(
            "Test bad request error message",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                crate::errors::endpoint_error_codes::BadRequestErrorCodes::AbsentEncryption,
            ),
        );

        let (header_name, header_value) = error.ieee_1609dot2dot1_error_header();
        assert_eq!(header_name, "Ieee-1609.2.1-Error");
        assert_eq!(header_value, "400-10");

        let error = HandleResponseError::new(
            "Test forbidden error message",
            StatusCode::FORBIDDEN,
            Ieee1609Dot2Dot1ErrorCodes::Forbidden(
                crate::errors::endpoint_error_codes::ForbiddenErrorCodes::FailedSignatureVerification,
            ),
        );

        let (header_name, header_value) = error.ieee_1609dot2dot1_error_header();
        assert_eq!(header_name, "Ieee-1609.2.1-Error");
        assert_eq!(header_value, "403-43");
    }

    #[test]
    fn test_http_error_response() {
        let error = HandleResponseError::new(
            "Test error message",
            StatusCode::BAD_REQUEST,
            Ieee1609Dot2Dot1ErrorCodes::BadRequest(
                crate::errors::endpoint_error_codes::BadRequestErrorCodes::AbsentEncryption,
            ),
        );

        let response = error.http_error_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response.headers().get("Ieee-1609.2.1-Error"),
            Some(&HeaderValue::from_static("400-10"))
        );
    }
}
