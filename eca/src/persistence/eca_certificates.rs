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

use crate::entities::eca_certificates::ActiveModel as EcaCertificatesActiveModel;
use crate::entities::eca_certificates::Column::InsertDate as EcaCertificatesInsertDateColumn;
use crate::entities::eca_certificates::Column::Name as EcaCertificatesNameColumn;
use crate::entities::eca_certificates::Entity as EcaCertificatesEntity;
use crate::entities::eca_certificates::Model as EcaCertificatesModel;
use chrono::Utc;
use scmscommon::errors::PersistenceLoadError;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

async fn fetch_latest_file_by_name(
    target_name: String,
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let model: Option<EcaCertificatesModel> = EcaCertificatesEntity::find()
        .filter(EcaCertificatesNameColumn.eq(&target_name))
        .order_by_desc(EcaCertificatesInsertDateColumn)
        .one(db)
        .await?;

    if model.is_none() {
        return Err(PersistenceLoadError::new(
            format!("No file {} found", target_name).as_str(),
        ));
    }

    let certificate_entity: EcaCertificatesModel = model.unwrap();
    Ok(certificate_entity.file)
}

pub async fn latest_eca_private_key(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "eca_private_key".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_eca_public_uncompressed(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "eca_public_uncompressed".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_eca_certificate(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "eca_certificate".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_ica_certificate(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "intermediate_ca_cert".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_rca_certificate(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "root_ca_cert".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_eca_certificate_chain(
    db: &DatabaseConnection,
) -> Result<Vec<Vec<u8>>, PersistenceLoadError> {
    let eca_certificate = latest_eca_certificate(db).await?;
    let ica_certificate = latest_ica_certificate(db).await?;
    let root_ca: Vec<u8> = latest_rca_certificate(db).await?;
    Ok(vec![eca_certificate, ica_certificate, root_ca])
}

pub async fn store_new_data_to_eca_certificates(
    db: &DatabaseConnection,
    filename: String,
    file: Vec<u8>,
) -> Result<(), PersistenceLoadError> {
    // Check if the entry already exists
    let existing_entry = EcaCertificatesEntity::find()
        .filter(EcaCertificatesNameColumn.eq(filename.clone()))
        .one(db)
        .await?;

    match existing_entry {
        Some(entry) => {
            let mut eca_certificate_active_model: EcaCertificatesActiveModel = entry.into();
            // Update existing entry
            eca_certificate_active_model.file = ActiveValue::Set(file);
            eca_certificate_active_model.insert_date = ActiveValue::Set(Utc::now().naive_utc());
            eca_certificate_active_model.update(db).await?;
        }
        None => {
            // Define the RA Certificate model
            let eca_certificate_model = EcaCertificatesActiveModel {
                name: ActiveValue::Set(filename),
                insert_date: ActiveValue::Set(Utc::now().naive_utc()),
                file: ActiveValue::Set(file.clone()),
                ..Default::default()
            };
            // Insert new entry
            eca_certificate_model.insert(db).await?;
        }
    }

    Ok(())
}
