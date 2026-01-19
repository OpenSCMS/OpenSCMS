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

use actix_web::http::StatusCode;
use scmscommon::endpoint_error_codes::Ieee1609Dot2Dot1ErrorCodes;
use scmscommon::errors;
use std::io::Write;
use zip::ZipWriter;
use zip::write::ExtendedFileOptions;
use zip::write::FileOptions;

pub fn write_file_to_zip(
    file_path: String,
    content: Vec<u8>,
    zip_writer: &mut ZipWriter<std::io::Cursor<&mut Vec<u8>>>,
) -> Result<(), errors::HandleResponseError> {
    let options: FileOptions<ExtendedFileOptions> =
        FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    zip_writer
        .start_file(file_path.clone(), options.clone())
        .map_err(|e| {
            errors::HandleResponseError::new(
                format!("Failed to start file in zip: {}", e).as_str(),
                StatusCode::INTERNAL_SERVER_ERROR,
                Ieee1609Dot2Dot1ErrorCodes::NotDefined,
            )
        })?;

    zip_writer.write_all(&content).map_err(|e| {
        errors::HandleResponseError::new(
            format!("Failed to write file to zip: {}", e).as_str(),
            StatusCode::INTERNAL_SERVER_ERROR,
            Ieee1609Dot2Dot1ErrorCodes::NotDefined,
        )
    })?;

    log::debug!("Added file {} to zip", file_path);
    Ok(())
}
