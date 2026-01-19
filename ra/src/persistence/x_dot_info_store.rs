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

use crate::entities::x_dot_info_store::ActiveModel as XDotInfoStoreActiveModel;
use crate::entities::x_dot_info_store::Column::CurrentI as IndexIColumn;
use crate::entities::x_dot_info_store::Column::Downloaded as DownloadedColumn;
use crate::entities::x_dot_info_store::Column::HashId as HashIdColumn;
use crate::entities::x_dot_info_store::Column::Id as IdColumn;
use crate::entities::x_dot_info_store::Entity as XDotInfoStoreActiveEntity;

use scmscommon::errors::PersistenceStoreError;
use sea_orm::ColumnTrait;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ActiveValue, DatabaseConnection};

pub async fn store_x_dot_info(
    db: &DatabaseConnection,
    hash_id: String,
    current_i: u64,
    file_binary: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    let x_dot_info_file_to_store = XDotInfoStoreActiveModel {
        hash_id: ActiveValue::Set(hash_id),
        current_i: ActiveValue::Set(current_i),
        file: ActiveValue::Set(file_binary),
        ..Default::default()
    };
    let _ = x_dot_info_file_to_store.insert(db).await?;

    Ok(())
}

pub async fn fetch_x_info_file_by_hash_id_and_i_value(
    db: &DatabaseConnection,
    hash_id: String,
    index_i: u64,
) -> Result<(i32, Vec<u8>), PersistenceStoreError> {
    let x_dot_info_model = XDotInfoStoreActiveEntity::find()
        .filter(HashIdColumn.eq(hash_id))
        .filter(IndexIColumn.eq(index_i))
        .one(db)
        .await?;

    if x_dot_info_model.is_none() {
        return Err(PersistenceStoreError::new("X.info not found"));
    }

    let x_dot_info = x_dot_info_model.unwrap();
    Ok((x_dot_info.id, x_dot_info.file.clone()))
}

pub async fn update_x_dot_info_set_downloaded_true(
    db: &DatabaseConnection,
    x_dot_info_id: i32,
) -> Result<(), PersistenceStoreError> {
    log::debug!(
        "Updating ({:?}) x_dot_info entry setting downloaded to true",
        x_dot_info_id,
    );

    XDotInfoStoreActiveEntity::update_many()
        .col_expr(DownloadedColumn, Expr::value(1))
        .filter(IdColumn.eq(x_dot_info_id))
        .exec(db)
        .await?;

    Ok(())
}
