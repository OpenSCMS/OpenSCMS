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

use crate::entities::la_certificates::ActiveModel as LaCertificatesActiveModel;
use crate::entities::la_certificates::Column::InsertDate as laCertificatesInsertDateColumn;
use crate::entities::la_certificates::Column::Name as laCertificatesNameColumn;
use crate::entities::la_certificates::Entity as laCertificatesEntity;
use crate::entities::la_certificates::Model as laCertificatesModel;
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
    let model: Option<laCertificatesModel> = laCertificatesEntity::find()
        .filter(laCertificatesNameColumn.eq(&target_name))
        .order_by_desc(laCertificatesInsertDateColumn)
        .one(db)
        .await?;

    if model.is_none() {
        return Err(PersistenceLoadError::new(
            format!("No file {} found", target_name).as_str(),
        ));
    }

    let certificate_entity: laCertificatesModel = model.unwrap();
    Ok(certificate_entity.file)
}

pub async fn latest_aca_public_key(
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let target_name = "aca_public_uncompressed".to_string();
    fetch_latest_file_by_name(target_name, db).await
}

pub async fn store_new_data_to_la_certificates(
    db: &DatabaseConnection,
    filename: String,
    file: Vec<u8>,
) -> Result<(), PersistenceLoadError> {
    // Check if the entry already exists
    let existing_entry = laCertificatesEntity::find()
        .filter(laCertificatesNameColumn.eq(filename.clone()))
        .one(db)
        .await?;

    match existing_entry {
        Some(entry) => {
            let mut la_certificate_active_model: LaCertificatesActiveModel = entry.into();
            // Update existing entry
            la_certificate_active_model.file = ActiveValue::Set(file);
            la_certificate_active_model.insert_date = ActiveValue::Set(Utc::now().naive_utc());
            la_certificate_active_model.update(db).await?;
        }
        None => {
            // Define the RA Certificate model
            let la_certificate_model = LaCertificatesActiveModel {
                name: ActiveValue::Set(filename),
                insert_date: ActiveValue::Set(Utc::now().naive_utc()),
                file: ActiveValue::Set(file.clone()),
                ..Default::default()
            };
            // Insert new entry
            la_certificate_model.insert(db).await?;
        }
    }

    Ok(())
}
