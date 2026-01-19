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

use crate::entities::composite_crl_store::ActiveModel as CompositeCrlStoreActiveModel;
use crate::entities::composite_crl_store::Column::CtlSeriesId as CompositeCrlStoreCtlSeriesIdColumn;
use crate::entities::composite_crl_store::Entity as CompositeCrlStoreEntity;
use crate::entities::composite_crl_store::Model as CompositeCrlStoreModel;
use chrono::Utc;
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

pub async fn fetch_composite_crl_by_ctl_series_id(
    ctl_series_id: String,
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, String), PersistenceLoadError> {
    let model = CompositeCrlStoreEntity::find()
        .filter(CompositeCrlStoreCtlSeriesIdColumn.eq(ctl_series_id))
        .one(db)
        .await?;

    if model.is_none() {
        log::warn!("Composite crl file not found");
        // return empty file
        return Ok((vec![], "".to_string()));
    }

    let composite_crl_entry = model.unwrap();
    Ok((composite_crl_entry.file, composite_crl_entry.file_name))
}

pub async fn cleanup_composite_crl_store(
    ctl_series_id: String,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    CompositeCrlStoreEntity::delete_many()
        .filter(CompositeCrlStoreCtlSeriesIdColumn.eq(ctl_series_id.to_uppercase()))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn store_composite_crl(
    db: &DatabaseConnection,
    ctl_series_id: String,
    file_name: String,
    file: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    // Check if the CRL already exists
    let existing_entry: Option<CompositeCrlStoreModel> = CompositeCrlStoreEntity::find()
        .filter(CompositeCrlStoreCtlSeriesIdColumn.eq(ctl_series_id.clone()))
        .one(db)
        .await?;

    match existing_entry {
        Some(entry) => {
            let mut composite_crl_active_model: CompositeCrlStoreActiveModel = entry.into();
            // Update existing entry
            composite_crl_active_model.file = ActiveValue::Set(file);
            composite_crl_active_model.file_name = ActiveValue::Set(file_name);
            composite_crl_active_model.updated_time = ActiveValue::Set(Utc::now().naive_utc());
            // Increment version
            let current_version = composite_crl_active_model.version.unwrap();
            composite_crl_active_model.version = ActiveValue::Set(current_version + 1);
            composite_crl_active_model.update(db).await?;
        }
        None => {
            // Define the RA Certificate model
            let composite_crl_active_model = CompositeCrlStoreActiveModel {
                updated_time: ActiveValue::Set(Utc::now().naive_utc()),
                ctl_series_id: ActiveValue::Set(ctl_series_id),
                version: ActiveValue::Set(1),
                file: ActiveValue::Set(file.clone()),
                file_name: ActiveValue::Set(file_name),
                ..Default::default()
            };
            // Insert new entry
            composite_crl_active_model.insert(db).await?;
        }
    }

    Ok(())
}
