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

use sea_orm::DatabaseConnection;

use scmscommon::{
    PersistenceLoadError,
    load_certificates_keys::{LoadMaterialType, read_certificate_or_key_file},
};

use crate::persistence::la_certificates::store_new_data_to_la_certificates;

pub async fn load_certificates_to_database(
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!("Loading LA certificates and keys into the database...");
    // Load aca_public_key
    let aca_public_key = read_certificate_or_key_file(LoadMaterialType::AcaPublicKey)
        .expect("Failed to load LA Public Key");

    store_new_data_to_la_certificates(db, "aca_public_uncompressed".to_string(), aca_public_key)
        .await?;

    log::debug!("Loaded LA certificates and keys into the database.");
    Ok(())
}
