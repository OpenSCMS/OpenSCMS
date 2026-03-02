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

use crate::persistence::eca_certificates::store_new_data_to_eca_certificates;

pub async fn load_certificates_to_database(
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!("Loading ECA certificates and keys into the database...");
    // Load root_ca_cert
    let root_ca_cert = read_certificate_or_key_file(LoadMaterialType::RootCaCertificate)
        .expect("Failed to load Root CA Certificate");

    // Load intermediate_ca_cert
    let intermediate_ca_cert = read_certificate_or_key_file(LoadMaterialType::IcaCertificate)
        .expect("Failed to load Intermediate CA Certificate");

    // Load eca_certificate
    let eca_cert = read_certificate_or_key_file(LoadMaterialType::EcaCertificate)
        .expect("Failed to load ECA Certificate");

    // Load eca_public_key
    let eca_public_key = read_certificate_or_key_file(LoadMaterialType::EcaPublicKey)
        .expect("Failed to load ECA Public Key");

    // Load aca_certificate
    let eca_private_key = read_certificate_or_key_file(LoadMaterialType::EcaPrivateKey)
        .expect("Failed to load ECA Private Key");

    store_new_data_to_eca_certificates(db, "root_ca_cert".to_string(), root_ca_cert).await?;
    store_new_data_to_eca_certificates(
        db,
        "intermediate_ca_cert".to_string(),
        intermediate_ca_cert,
    )
    .await?;
    store_new_data_to_eca_certificates(db, "eca_certificate".to_string(), eca_cert).await?;
    store_new_data_to_eca_certificates(db, "eca_public_uncompressed".to_string(), eca_public_key)
        .await?;
    store_new_data_to_eca_certificates(db, "eca_private_key".to_string(), eca_private_key).await?;

    log::debug!("Loaded ECA certificates and keys into the database.");
    Ok(())
}
