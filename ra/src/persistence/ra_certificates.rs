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

use crate::entities::ra_certificates::ActiveModel as RaCertificatesActiveModel;
use crate::entities::ra_certificates::Column::CertId as RaCertificatesCertIdColumn;
use crate::entities::ra_certificates::Column::InsertDate as RaCertificatesInsertDateColumn;
use crate::entities::ra_certificates::Column::Name as RaCertificatesNameColumn;
use crate::entities::ra_certificates::Column::UpdatedTime as RaCertificatesUpdatedTimeColumn;
use crate::entities::ra_certificates::Entity as RaCertificatesEntity;
use crate::entities::ra_certificates::Model as RaCertificatesModel;
use chrono::{DateTime, Local, Utc, offset::TimeZone};
use scmscommon::errors::PersistenceLoadError;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

pub async fn get_ra_certificates_latest_updated_time(
    db: &DatabaseConnection,
) -> Result<u32, PersistenceLoadError> {
    let model: Option<RaCertificatesModel> = RaCertificatesEntity::find()
        .order_by_desc(RaCertificatesUpdatedTimeColumn)
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(0);
    }

    let ra_certificate_entity: RaCertificatesModel = model.unwrap();
    let updated_datetime: DateTime<Local> = Local
        .from_local_datetime(&ra_certificate_entity.updated_time)
        .unwrap();

    Ok(updated_datetime.timestamp() as u32)
}

pub async fn fetch_latest_file_by_cert_id(
    cert_id: String,
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let model: Option<RaCertificatesModel> = RaCertificatesEntity::find()
        .filter(RaCertificatesCertIdColumn.eq(&cert_id))
        .order_by_desc(RaCertificatesInsertDateColumn)
        .one(db)
        .await?;

    if model.is_none() {
        return Err(PersistenceLoadError::new(
            format!("No file {} found", cert_id).as_str(),
        ));
    }

    let certificate_entity: RaCertificatesModel = model.unwrap();
    Ok(certificate_entity.file)
}

async fn fetch_latest_file_by_name(
    target_name: String,
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let model: Option<RaCertificatesModel> = RaCertificatesEntity::find()
        .filter(RaCertificatesNameColumn.eq(&target_name))
        .order_by_desc(RaCertificatesInsertDateColumn)
        .one(db)
        .await?;

    if model.is_none() {
        return Err(PersistenceLoadError::new(
            format!("No file {} found", target_name).as_str(),
        ));
    }

    let certificate_entity: RaCertificatesModel = model.unwrap();
    Ok(certificate_entity.file)
}

pub async fn latest_ra_certificate(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "ra_certificate".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_ra_private_key(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "ra_private_key".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_ra_enc_private_key(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "ra_enc_private_key".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn latest_eca_public_key(
    db: &DatabaseConnection,
) -> Result<[u8; 64], PersistenceLoadError> {
    let target_name = "eca_public_key".to_string();
    let file = fetch_latest_file_by_name(target_name, db).await?;

    let mut array = [0; 64];
    array.copy_from_slice(&file);
    Ok(array)
}

pub async fn latest_eca_certificate(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "eca_certificate".to_string();
    let file = fetch_latest_file_by_name(target_name, db).await?;
    Ok(file)
}

pub async fn store_new_eca_public_key(
    file: Vec<u8>,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    // Check if the entry already exists
    let existing_entry = RaCertificatesEntity::find()
        .filter(RaCertificatesNameColumn.eq("eca_public_key"))
        .one(db)
        .await?;

    match existing_entry {
        Some(entry) => {
            let mut ra_certificate_active_model: RaCertificatesActiveModel = entry.into();
            // Update existing entry
            ra_certificate_active_model.file = ActiveValue::Set(file);
            ra_certificate_active_model.insert_date = ActiveValue::Set(Utc::now().naive_utc());
            ra_certificate_active_model.update(db).await?;
        }
        None => {
            // Define the RA Certificate model
            let ra_certificate_model = RaCertificatesActiveModel {
                name: ActiveValue::Set("eca_public_key".to_string()),
                insert_date: ActiveValue::Set(Utc::now().naive_utc()),
                file: ActiveValue::Set(file.clone()),
                ..Default::default()
            };
            // Insert new entry
            ra_certificate_model.insert(db).await?;
        }
    }

    Ok(())
}

pub async fn store_new_data_to_ra_certificates(
    db: &DatabaseConnection,
    filename: String,
    file: Vec<u8>,
    cert_id: Option<String>,
) -> Result<(), PersistenceLoadError> {
    // Check if the entry already exists
    let existing_entry = RaCertificatesEntity::find()
        .filter(RaCertificatesNameColumn.eq(filename.clone()))
        .one(db)
        .await?;

    match existing_entry {
        Some(entry) => {
            let mut ra_certificate_active_model: RaCertificatesActiveModel = entry.into();
            // Update existing entry
            ra_certificate_active_model.file = ActiveValue::Set(file);
            ra_certificate_active_model.insert_date = ActiveValue::Set(Utc::now().naive_utc());
            ra_certificate_active_model.cert_id = ActiveValue::Set(cert_id);

            ra_certificate_active_model.update(db).await?;

            log::debug!("Updated existing RA certificate entry: {}", filename);
        }
        None => {
            // Define the RA Certificate model
            let ra_certificate_model = RaCertificatesActiveModel {
                name: ActiveValue::Set(filename.clone()),
                insert_date: ActiveValue::Set(Utc::now().naive_utc()),
                file: ActiveValue::Set(file.clone()),
                cert_id: ActiveValue::Set(cert_id.clone()),
                ..Default::default()
            };
            // Insert new entry
            ra_certificate_model.insert(db).await?;

            log::debug!(
                "Inserted new RA certificate entry: {:?} {:?}",
                filename,
                cert_id
            );
        }
    }

    Ok(())
}
