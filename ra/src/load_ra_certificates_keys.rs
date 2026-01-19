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

use futures::TryFutureExt;
use sea_orm::DatabaseConnection;

// use liblgasn::encode_certificate;
use scmscommon::{
    GlobalConfig, PersistenceLoadError,
    load_certificates_keys::{LoadMaterialType, read_certificate_or_key_file},
};
use sha2::{Digest, Sha256};

use crate::persistence::ccf_store::cleanup_ccf_store;
use crate::persistence::composite_crl_store::cleanup_composite_crl_store;
use crate::persistence::crl_store::cleanup_crl_store;
use crate::persistence::ctl_store::cleanup_ctl_store;

use crate::persistence::ccf_store::store_ccf;
use crate::persistence::ctl_store::store_ctl_file;
use crate::persistence::ra_certificates::store_new_data_to_ra_certificates;

use crate::ra_endpoint::certificate_revocation_list::build_and_store_composite_crl;

fn compute_certificate_hash_id(certificate: Vec<u8>) -> String {
    let mut hasher = Sha256::new();
    hasher.update(&certificate);
    let certificate_hash = hasher.finalize();

    format!(
        "{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        certificate_hash[24],
        certificate_hash[25],
        certificate_hash[26],
        certificate_hash[27],
        certificate_hash[28],
        certificate_hash[29],
        certificate_hash[30],
        certificate_hash[31]
    )
}

pub async fn load_certificates_to_database(
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!("Loading RA certificates and keys into the database...");
    // Load RA Certificate
    let ra_cert = read_certificate_or_key_file(LoadMaterialType::RaCertificate)
        .expect("Failed to load RA Certificate");

    // Load RA Private Key
    let ra_private_key = read_certificate_or_key_file(LoadMaterialType::RaPrivateKey)
        .expect("Failed to load RA Private Key");

    // Load RA Public Key
    let ra_public_key = read_certificate_or_key_file(LoadMaterialType::RaPublicKey)
        .expect("Failed to load RA Public Key");

    // Load RA Encryption Private Key
    let ra_enc_private_key = read_certificate_or_key_file(LoadMaterialType::RaEncPrivateKey)
        .expect("Failed to load RA Encryption Private Key");

    // Load RA Encryption Public Key
    let ra_enc_public_key = read_certificate_or_key_file(LoadMaterialType::RaEncPublicKey)
        .expect("Failed to load RA Encryption Public Key");

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
    let aca_cert = read_certificate_or_key_file(LoadMaterialType::AcaCertificate)
        .expect("Failed to load ACA Certificate");

    // Compute certificate hash ID
    let aca_cert_hash = compute_certificate_hash_id(aca_cert.clone());
    let eca_cert_hash = compute_certificate_hash_id(eca_cert.clone());
    let ra_cert_hash = compute_certificate_hash_id(ra_cert.clone());
    let rootca_cert_hash = compute_certificate_hash_id(root_ca_cert.clone());
    let ica_cert_hash = compute_certificate_hash_id(intermediate_ca_cert.clone());

    // Store
    store_new_data_to_ra_certificates(
        db,
        "ra_certificate".to_string(),
        ra_cert,
        Some(ra_cert_hash),
    )
    .await?;
    store_new_data_to_ra_certificates(db, "ra_private_key".to_string(), ra_private_key, None)
        .await?;
    store_new_data_to_ra_certificates(
        db,
        "ra_public_uncompressed".to_string(),
        ra_public_key,
        None,
    )
    .await?;
    store_new_data_to_ra_certificates(
        db,
        "ra_enc_private_key".to_string(),
        ra_enc_private_key,
        None,
    )
    .await?;
    store_new_data_to_ra_certificates(
        db,
        "ra_enc_public_uncompressed".to_string(),
        ra_enc_public_key,
        None,
    )
    .await?;
    store_new_data_to_ra_certificates(
        db,
        "root_ca_cert".to_string(),
        root_ca_cert,
        Some(rootca_cert_hash),
    )
    .await?;
    store_new_data_to_ra_certificates(
        db,
        "intermediate_ca_cert".to_string(),
        intermediate_ca_cert,
        Some(ica_cert_hash),
    )
    .await?;
    store_new_data_to_ra_certificates(
        db,
        "eca_certificate".to_string(),
        eca_cert,
        Some(eca_cert_hash),
    )
    .await?;
    store_new_data_to_ra_certificates(db, "eca_public_key".to_string(), eca_public_key, None)
        .await?;
    store_new_data_to_ra_certificates(
        db,
        "aca_certificate".to_string(),
        aca_cert,
        Some(aca_cert_hash),
    )
    .await?;

    log::debug!("Loaded RA certificates and keys into the database.");
    Ok(())
}

pub async fn build_load_crl_ccf_ctl_to_ra_db(
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!("Building and loading CRL, CCF and CTL to RA database...");

    let ctl_series_id = GlobalConfig::global().ctl_series_id.clone();

    // Load and store CTL file
    log::debug!("Loading and storing CTL file.");
    cleanup_ctl_store(ctl_series_id.clone(), db).await?;

    let sequence_number: i32 = 0;
    let ctl_file =
        read_certificate_or_key_file(LoadMaterialType::Ctl).expect("Failed to load CTL file");

    store_ctl_file(db, ctl_series_id.clone(), sequence_number, ctl_file)
        .map_err(|e| {
            PersistenceLoadError::new(&format!("Failed to store CTL file to database: {:?}", e))
        })
        .await?;

    // Load and store CCF CTL file
    log::debug!("Loading and storing CCF CTL file.");
    cleanup_ccf_store(ctl_series_id.clone(), db).await?;

    let ccf_file = read_certificate_or_key_file(LoadMaterialType::CcfCtl)
        .expect("Failed to load CCF CTL file");
    let version: i32 = 1;

    store_ccf(db, ctl_series_id.clone(), version, ccf_file)
        .map_err(|e| {
            PersistenceLoadError::new(&format!("Failed to store CCF file to database: {:?}", e))
        })
        .await?;

    // build_and_store_composite_crl
    log::debug!("Cleaning up existing Composite CRL files...");
    cleanup_composite_crl_store(ctl_series_id.clone(), db).await?;
    cleanup_crl_store(db).await?;

    log::debug!("Building Composite CRL file...");
    build_and_store_composite_crl(ctl_series_id, db)
        .await
        .map_err(|e| {
            PersistenceLoadError::new(&format!("Failed to Composite CRL file: {:?}", e))
        })?;

    log::debug!("Built and loaded CRL, CCF and CTL to RA database.");
    Ok(())
}
