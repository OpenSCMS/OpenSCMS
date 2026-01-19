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

// use liblgasn::encode_certificate;
use scmscommon::{
    PersistenceLoadError,
    load_certificates_keys::{LoadMaterialType, read_certificate_or_key_file},
};

use crate::persistence::aca_certificates::store_new_data_to_aca_certificates;

pub async fn load_certificates_to_database(
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!("Loading ACA certificates and keys into the database...");

    // Load aca_certificate
    let aca_cert = read_certificate_or_key_file(LoadMaterialType::AcaCertificate)
        .expect("Failed to load ACA Certificate");

    // Load aca_public_key
    let aca_public_key = read_certificate_or_key_file(LoadMaterialType::AcaPublicKey)
        .expect("Failed to load ACA Public Key");

    // Load aca_certificate
    let aca_private_key = read_certificate_or_key_file(LoadMaterialType::AcaPrivateKey)
        .expect("Failed to load ACA Private Key");

    store_new_data_to_aca_certificates(db, "aca_certificate".to_string(), aca_cert).await?;
    store_new_data_to_aca_certificates(db, "aca_public_uncompressed".to_string(), aca_public_key)
        .await?;
    store_new_data_to_aca_certificates(db, "aca_private_key".to_string(), aca_private_key).await?;

    log::debug!("Loaded ACA certificates and keys into the database.");
    Ok(())
}
