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

use crate::entities::eca_request_management::ActiveModel as EcaRequestManagementActiveModel;
use crate::entities::eca_request_management::Column::AppIp as EcaRequestManagementAppIp;
use crate::entities::eca_request_management::Column::LastRequestTime as EcaRequestManagementLastRequestTime;
use crate::entities::eca_request_management::Column::PeriodRequestCount as EcaRequestManagementPeriodRequestCount;
use crate::entities::eca_request_management::Entity as EcaRequestManagementEntity;
use crate::entities::eca_request_management::Model as EcaRequestManagementModel;
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use scmscommon::get_current_time_seconds;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue, ColumnTrait};

pub async fn fetch_last_request_time_and_count_by_ip(
    app_ip: String,
    db: &DatabaseConnection,
) -> Result<Option<(u32, i32)>, PersistenceLoadError> {
    let model: Option<EcaRequestManagementModel> = EcaRequestManagementEntity::find()
        .filter(EcaRequestManagementAppIp.eq(&app_ip))
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(None);
    }

    let request_entity: EcaRequestManagementModel = model.unwrap();
    Ok(Some((
        request_entity.last_request_time,
        request_entity.period_request_count,
    )))
}

pub async fn store_new_eca_request(
    db: &DatabaseConnection,
    app_ip: String,
    last_request_time: u32,
) -> Result<(), PersistenceStoreError> {
    let request_store = EcaRequestManagementActiveModel {
        app_ip: ActiveValue::Set(app_ip),
        last_request_time: ActiveValue::Set(last_request_time),
        period_started_at: ActiveValue::Set(get_current_time_seconds()),
        period_request_count: ActiveValue::Set(1),
        ..Default::default()
    };
    let _ = request_store.insert(db).await?;

    Ok(())
}

pub async fn update_status_by_ids(
    db: &DatabaseConnection,
    app_ip: String,
    request_time: u32,
    new_count: i32,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} eca request entry setting request time to {} and weekly count to {}",
        app_ip,
        request_time,
        new_count
    );
    EcaRequestManagementEntity::update_many()
        .col_expr(
            EcaRequestManagementLastRequestTime,
            Expr::value(request_time),
        )
        .col_expr(
            EcaRequestManagementPeriodRequestCount,
            Expr::value(new_count),
        )
        .filter(EcaRequestManagementAppIp.eq(app_ip))
        .exec(db)
        .await?;

    Ok(())
}
