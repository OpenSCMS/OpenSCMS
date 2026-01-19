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

use crate::entities::ccf_store::ActiveModel as CcfStoreActiveModel;
use crate::entities::ccf_store::Column::CtlSeriesId as CcfStoreCtlSeriesIdColumn;
use crate::entities::ccf_store::Column::UpdatedTime as CcfStoreUpdatedTime;
use crate::entities::ccf_store::Column::Version as CcfStoreVersion;
use crate::entities::ccf_store::Entity as CcfStoreEntity;
use crate::entities::ccf_store::Model as CcfStoreModel;
use chrono::{DateTime, Local, offset::TimeZone};
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::QueryOrder;

pub async fn get_latest_updated_time(db: &DatabaseConnection) -> Result<u32, PersistenceLoadError> {
    let model: Option<CcfStoreModel> = CcfStoreEntity::find()
        .order_by_desc(CcfStoreUpdatedTime)
        .one(db)
        .await?;

    if model.is_none() {
        log::warn!("No Ccf file found");
        return Ok(0);
    }

    let ccf_entity: CcfStoreModel = model.unwrap();
    let updated_datetime: DateTime<Local> =
        Local.from_local_datetime(&ccf_entity.updated_time).unwrap();

    Ok(updated_datetime.timestamp() as u32)
}

pub async fn fetch_ccf_by_ctl_series_id(
    ctl_series_id: String,
    db: &DatabaseConnection,
) -> Result<(Vec<u8>, i32), PersistenceLoadError> {
    let model: Option<CcfStoreModel> = CcfStoreEntity::find()
        .filter(CcfStoreCtlSeriesIdColumn.eq(&ctl_series_id))
        .order_by_desc(CcfStoreVersion)
        .one(db)
        .await?;

    if model.is_none() {
        log::warn!(
            "No Ccf file found for ctl series id {} found",
            ctl_series_id
        );
        return Ok((Vec::new(), 0));
    }

    let ccf_entity: CcfStoreModel = model.unwrap();
    Ok((ccf_entity.file, ccf_entity.version))
}

pub async fn cleanup_ccf_store(
    ctl_series_id: String,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    CcfStoreEntity::delete_many()
        .filter(CcfStoreCtlSeriesIdColumn.eq(ctl_series_id.to_uppercase()))
        .exec(db)
        .await?;
    Ok(())
}

pub async fn store_ccf(
    db: &DatabaseConnection,
    ctl_series_id: String,
    version: i32,
    ccf: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    let ccf_store = CcfStoreActiveModel {
        ctl_series_id: ActiveValue::Set(ctl_series_id),
        version: ActiveValue::Set(version),
        file: ActiveValue::Set(ccf),
        ..Default::default()
    };
    let _ = ccf_store.insert(db).await?;

    Ok(())
}
