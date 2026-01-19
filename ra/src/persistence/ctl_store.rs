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

use crate::entities::ctl_store::ActiveModel as CtlStoreActiveModel;
use crate::entities::ctl_store::Column::CtlSeriesId as CtlStoreCtlSeriesIdColumn;
use crate::entities::ctl_store::Column::SquenceNumber as CtlStoreSequenceNumberColumn;
use crate::entities::ctl_store::Column::UpdatedTime as CtlStoreUpdatedTimeColumn;
use crate::entities::ctl_store::Entity as CtlStoreEntity;
use crate::entities::ctl_store::Model as CtlStoreModel;
use chrono::{DateTime, Local, offset::TimeZone};
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

fn hex_to_vec_u8(hex: &str) -> Result<Vec<u8>, PersistenceLoadError> {
    if hex.len() != 16 {
        return Err(PersistenceLoadError::new("Invalid hex string"));
    }
    (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
                .map_err(|_| PersistenceLoadError::new("Invalid hex string"))
        })
        .collect()
}

pub async fn fetch_all_ctl_series_ids_sequence_number_and_updated_time(
    db: &DatabaseConnection,
) -> Result<Vec<(Vec<u8>, u16, u32)>, PersistenceLoadError> {
    let models: Vec<CtlStoreModel> = CtlStoreEntity::find()
        .order_by_desc(CtlStoreUpdatedTimeColumn)
        .all(db)
        .await?;

    let mut result: Vec<(Vec<u8>, u16, u32)> = Vec::new();
    for model in models {
        // Convert ctl series id to vec<u8>
        let ctl_series_id: Vec<u8> = hex_to_vec_u8(&model.ctl_series_id)?;
        let updated_datetime: DateTime<Local> =
            Local.from_local_datetime(&model.updated_time).unwrap();

        result.push((
            ctl_series_id,
            model.squence_number as u16,
            updated_datetime.timestamp() as u32,
        ));
    }
    Ok(result)
}

pub async fn fetch_ctl_file_by_ctl_series_id(
    ctl_series_id: String,
    sequence_number: Option<u16>,
    db: &DatabaseConnection,
) -> Result<(Vec<(Vec<u8>, String)>, i32), PersistenceLoadError> {
    let models: Vec<CtlStoreModel> = CtlStoreEntity::find()
        .filter(CtlStoreCtlSeriesIdColumn.eq(&ctl_series_id))
        .order_by_desc(CtlStoreSequenceNumberColumn)
        .all(db)
        .await?;

    if models.is_empty() {
        log::warn!(
            "No Ctl file found for ctl series id {} found",
            ctl_series_id
        );
        return Ok((Vec::new(), -1));
    }

    let target_number = sequence_number.map_or(-1, |seq| seq as i32);

    let mut files: Vec<(Vec<u8>, String)> = Vec::new();
    let mut min_sequence_number = i32::MAX;
    for model in models {
        if target_number != -1 && model.squence_number != target_number {
            continue;
        }
        files.push((
            model.file,
            format!("ctl_{}_{}.oer", model.ctl_series_id, model.squence_number),
        ));
        if model.squence_number < min_sequence_number {
            min_sequence_number = model.squence_number;
        }
    }
    Ok((files, min_sequence_number))
}

pub async fn cleanup_ctl_store(
    ctl_series_id: String,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    CtlStoreEntity::delete_many()
        .filter(CtlStoreCtlSeriesIdColumn.eq(ctl_series_id.to_uppercase()))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn store_ctl_file(
    db: &DatabaseConnection,
    ctl_series_id: String,
    squence_number: i32,
    ctl_file: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    let ctl_file_store = CtlStoreActiveModel {
        ctl_series_id: ActiveValue::Set(ctl_series_id),
        squence_number: ActiveValue::Set(squence_number),
        file: ActiveValue::Set(ctl_file),
        ..Default::default()
    };
    let _ = ctl_file_store.insert(db).await?;

    Ok(())
}
