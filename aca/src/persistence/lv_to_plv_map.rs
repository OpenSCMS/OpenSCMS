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

use crate::core_types::LvToPlvMap;
use crate::entities::lv_to_plv_map::ActiveModel;
use crate::entities::lv_to_plv_map::Column::Lv as LvColumn;
use crate::entities::lv_to_plv_map::Entity;
use crate::entities::lv_to_plv_map::Model as LvToPlvRecord;
use crate::errors::FindPlvError;
use scmscommon::errors::PersistenceStoreError;
use scmscommon::util::vec_to_array_16;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

pub async fn store_lv_to_plv_map(
    db: &DatabaseConnection,
    lv_to_plv_map: LvToPlvMap,
) -> Result<(), PersistenceStoreError> {
    let lv_to_plv_map_to_store = ActiveModel {
        lv: ActiveValue::Set(lv_to_plv_map.lv.to_vec()),
        plv: ActiveValue::Set(lv_to_plv_map.plv.to_vec()),
        i_value: ActiveValue::Set(lv_to_plv_map.i_index),
        ..Default::default()
    };
    let _ = lv_to_plv_map_to_store.insert(db).await?;
    Ok(())
}

pub async fn find_plvs(
    db: &DatabaseConnection,
    lv: [u8; 16],
) -> Result<Vec<[u8; 16]>, FindPlvError> {
    let records: Vec<LvToPlvRecord> = find_records_with_lv(db, lv).await?;
    let mut plvs: Vec<[u8; 16]> = vec![];
    for record in records {
        let plv = record.plv;
        let plv = vec_to_array_16(plv)?;
        plvs.push(plv);
    }
    Ok(plvs)
}

async fn find_records_with_lv(
    db: &DatabaseConnection,
    lv: [u8; 16],
) -> Result<Vec<LvToPlvRecord>, FindPlvError> {
    let records: Vec<LvToPlvRecord> = Entity::find()
        .filter(LvColumn.eq(lv.to_vec()))
        .all(db)
        .await?;

    if records.is_empty() {
        Err(FindPlvError::new(
            format!("Couldn't find any records with this LV: '{:?}'", lv).as_str(),
        ))
    } else {
        Ok(records)
    }
}
