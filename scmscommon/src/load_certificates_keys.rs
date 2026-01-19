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

use std::fs::File;
use std::io::{self, Read};

use crate::config::GlobalConfig;
use crate::errors::PersistenceLoadError;

pub enum LoadMaterialType {
    AcaCertificate,
    AcaPublicKey,
    AcaPrivateKey,
    EcaCertificate,
    EcaPublicKey,
    EcaPrivateKey,
    RaCertificate,
    RaPublicKey,
    RaPrivateKey,
    RaEncPublicKey,
    RaEncPrivateKey,
    RootCaCertificate,
    IcaCertificate,
    Ctl,
    CcfCtl,
}

fn read_binary_file(path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn read_certificate_or_key_file(
    material_type: LoadMaterialType,
) -> Result<Vec<u8>, PersistenceLoadError> {
    // Helper function to get folder and file names based on material type
    fn get_folder_and_file<'a>(material_type: &'a LoadMaterialType) -> (&'a str, &'a str) {
        match material_type {
            LoadMaterialType::AcaCertificate => (
                &GlobalConfig::global().aca_certs_folder,
                &GlobalConfig::global().aca_certificate_file,
            ),
            LoadMaterialType::AcaPublicKey => (
                &GlobalConfig::global().aca_certs_folder,
                &GlobalConfig::global().aca_public_key_file,
            ),
            LoadMaterialType::AcaPrivateKey => (
                &GlobalConfig::global().aca_certs_folder,
                &GlobalConfig::global().aca_private_key_file,
            ),
            LoadMaterialType::EcaCertificate => (
                &GlobalConfig::global().eca_certs_folder,
                &GlobalConfig::global().eca_certificate_file,
            ),
            LoadMaterialType::EcaPublicKey => (
                &GlobalConfig::global().eca_certs_folder,
                &GlobalConfig::global().eca_public_key_file,
            ),
            LoadMaterialType::EcaPrivateKey => (
                &GlobalConfig::global().eca_certs_folder,
                &GlobalConfig::global().eca_private_key_file,
            ),
            LoadMaterialType::RaCertificate => (
                &GlobalConfig::global().ra_certs_folder,
                &GlobalConfig::global().ra_certificate_file,
            ),
            LoadMaterialType::RaPublicKey => (
                &GlobalConfig::global().ra_certs_folder,
                &GlobalConfig::global().ra_public_key_file,
            ),
            LoadMaterialType::RaPrivateKey => (
                &GlobalConfig::global().ra_certs_folder,
                &GlobalConfig::global().ra_private_key_file,
            ),
            LoadMaterialType::RaEncPublicKey => (
                &GlobalConfig::global().ra_certs_folder,
                &GlobalConfig::global().ra_enc_public_key_file,
            ),
            LoadMaterialType::RaEncPrivateKey => (
                &GlobalConfig::global().ra_certs_folder,
                &GlobalConfig::global().ra_enc_private_key_file,
            ),
            LoadMaterialType::Ctl => (
                &GlobalConfig::global().ctl_folder,
                &GlobalConfig::global().ctl_file,
            ),
            LoadMaterialType::CcfCtl => (
                &GlobalConfig::global().ctl_folder,
                &GlobalConfig::global().ctl_ccf_file,
            ),
            LoadMaterialType::RootCaCertificate => (
                &GlobalConfig::global().rootca_certs_folder,
                &GlobalConfig::global().rootca_certificate_file,
            ),
            LoadMaterialType::IcaCertificate => (
                &GlobalConfig::global().ica_certs_folder,
                &GlobalConfig::global().ica_certificate_file,
            ),
        }
    }

    // Get the folder and file names
    let (folder, file) = get_folder_and_file(&material_type);

    // Construct the path
    let path = format!("{}/{}", folder, file);

    // Read the binary file and handle errors
    read_binary_file(&path).map_err(|e: io::Error| {
        PersistenceLoadError::new(&format!("Failed to Read Certificate or Key: {}", e))
    })
}
