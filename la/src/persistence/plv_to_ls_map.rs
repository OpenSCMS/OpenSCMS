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

use crate::core_types::PlvToLsMap;
use crate::entities::plv_to_ls_map::ActiveModel;
use crate::entities::plv_to_ls_map::Column::Plv as PlvValueColumn;
use crate::entities::plv_to_ls_map::Entity;
use crate::entities::plv_to_ls_map::Model as PlvToLsModel;
use crate::errors::FindLsError;
use scmscommon::errors::PersistenceStoreError;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter,
};

impl PlvToLsMap {
    pub async fn store(&self, db: &DatabaseConnection) -> Result<(), PersistenceStoreError> {
        let plv_to_ls_map_to_store = ActiveModel {
            plv: ActiveValue::Set(self.plv.to_vec()),
            ls: ActiveValue::Set(self.ls.to_vec()),
            i_index: ActiveValue::Set(self.i_index),
            req_id: ActiveValue::Set(self.req_id),
            request_id: ActiveValue::Set(self.request_id.clone()),
            ..Default::default()
        };

        let _ = plv_to_ls_map_to_store.insert(db).await?;

        Ok(())
    }
}

pub async fn find_ls(db: &DatabaseConnection, plv_value: Vec<u8>) -> Result<Vec<u8>, FindLsError> {
    // use PLV and return LS
    let plv_to_ls_model: PlvToLsModel = find_record_with_plv(db, plv_value).await?;
    let plv_value = plv_to_ls_model.plv;
    Ok(plv_value)
}

async fn find_record_with_plv(
    db: &DatabaseConnection,
    plv_value: Vec<u8>,
) -> Result<PlvToLsModel, FindLsError> {
    let plv_to_ls_model_option: Option<PlvToLsModel> = Entity::find()
        .filter(PlvValueColumn.eq(plv_value.clone()))
        .one(db)
        .await?;

    match plv_to_ls_model_option {
        Some(plv_to_ls_model) => Ok(plv_to_ls_model),
        None => Err(FindLsError::new(
            format!("Couldn't find the record with this plv: '{:?}'", plv_value).as_str(),
        )),
    }
}
