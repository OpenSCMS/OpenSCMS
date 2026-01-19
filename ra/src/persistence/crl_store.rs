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

use crate::entities::crl_store::ActiveModel as CrlStoreActiveModel;
use crate::entities::crl_store::Column::CracaId as CrlStoreCracaIdColumn;
use crate::entities::crl_store::Column::CrlSeries as CrlStoreCrlSeriesColumn;
use crate::entities::crl_store::Entity as CrlStoreEntity;
use crate::entities::crl_store::Model as CrlStoreModel;
use chrono::Utc;
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;

pub async fn fech_all_crls(db: &DatabaseConnection) -> Result<Vec<Vec<u8>>, PersistenceLoadError> {
    let models = CrlStoreEntity::find().all(db).await?;

    let mut all_crl_files = Vec::new();
    for model in models {
        all_crl_files.push(model.file);
    }

    Ok(all_crl_files)
}

pub async fn fech_all_crls_models(
    db: &DatabaseConnection,
) -> Result<Vec<CrlStoreModel>, PersistenceLoadError> {
    let models = CrlStoreEntity::find().all(db).await?;
    Ok(models)
}

pub async fn fetch_crl_by_crl_series_and_or_cracaid(
    crl_series: Option<u32>,
    craca_id: Option<String>,
    db: &DatabaseConnection,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let mut crl_model_find = CrlStoreEntity::find();
    // Filter by crl_series and cracaid if provided
    if let Some(crl_series) = crl_series {
        crl_model_find = crl_model_find.filter(CrlStoreCrlSeriesColumn.eq(crl_series as i32));
    }
    if let Some(craca_id) = craca_id {
        crl_model_find = crl_model_find.filter(CrlStoreCracaIdColumn.eq(craca_id));
    }

    let model: Option<CrlStoreModel> = crl_model_find.one(db).await?;
    if model.is_none() {
        // Warning
        log::warn!("Crl not found");
        return Ok(vec![]);
    }

    let crl_entry = model.unwrap();
    Ok(crl_entry.file)
}

pub async fn cleanup_crl_store(db: &DatabaseConnection) -> Result<(), PersistenceLoadError> {
    CrlStoreEntity::delete_many().exec(db).await?;
    Ok(())
}

pub async fn store_crl(
    db: &DatabaseConnection,
    crl_series: i32,
    craca_id: String,
    file: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    // Check if the CRL already exists
    let existing_entry: Option<CrlStoreModel> = CrlStoreEntity::find()
        .filter(CrlStoreCrlSeriesColumn.eq(crl_series))
        .filter(CrlStoreCracaIdColumn.eq(craca_id.clone()))
        .one(db)
        .await?;

    match existing_entry {
        Some(entry) => {
            let mut crl_active_model: CrlStoreActiveModel = entry.into();
            // Update existing entry
            crl_active_model.file = ActiveValue::Set(file);
            crl_active_model.updated_time = ActiveValue::Set(Utc::now().naive_utc());
            crl_active_model.update(db).await?;
        }
        None => {
            // Define the RA Certificate model
            let crl_active_model = CrlStoreActiveModel {
                updated_time: ActiveValue::Set(Utc::now().naive_utc()),
                craca_id: ActiveValue::Set(craca_id.clone()),
                crl_series: ActiveValue::Set(crl_series),
                file: ActiveValue::Set(file.clone()),
                ..Default::default()
            };
            // Insert new entry
            crl_active_model.insert(db).await?;
        }
    }

    Ok(())
}
