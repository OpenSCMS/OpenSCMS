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

use crate::entities::ra_request_management::ActiveModel as RaRequestManagementActiveModel;
use crate::entities::ra_request_management::Column::AppId as RaRequestManagementAppIpColumn;
use crate::entities::ra_request_management::Column::LastRequestTime as RaRequestManagementLastRequestTime;
use crate::entities::ra_request_management::Column::PeriodRequestCount as RaRequestManagementPeriodRequestCount;
use crate::entities::ra_request_management::Column::RaRequestManagementRequestType as RaRequestManagementRequestTypeColumn;
use crate::entities::ra_request_management::Column::VehicleId as RaRequestManagementVid;
use crate::entities::ra_request_management::Entity as RaRequestManagementEntity;
use crate::entities::ra_request_management::Model as RaRequestManagementModel;
use crate::entities::sea_orm_active_enums::RaRequestManagementRequestType;
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use scmscommon::get_current_time_seconds;
use sea_orm::ActiveModelTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::QueryFilter;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveValue, ColumnTrait};

pub async fn fetch_last_request_time_and_count_by_vid(
    vid: u64,
    request_type: RaRequestManagementRequestType,
    db: &DatabaseConnection,
) -> Result<Option<(u32, i32)>, PersistenceLoadError> {
    let model: Option<RaRequestManagementModel> = RaRequestManagementEntity::find()
        .filter(RaRequestManagementVid.eq(vid))
        .filter(RaRequestManagementRequestTypeColumn.eq(request_type))
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(None);
    }

    let request_entity: RaRequestManagementModel = model.unwrap();
    Ok(Some((
        request_entity.last_request_time,
        request_entity.period_request_count,
    )))
}

pub async fn fetch_last_request_time_and_count_by_app_ip(
    app_ip: String,
    request_type: RaRequestManagementRequestType,
    db: &DatabaseConnection,
) -> Result<Option<(u32, i32)>, PersistenceLoadError> {
    let model: Option<RaRequestManagementModel> = RaRequestManagementEntity::find()
        .filter(RaRequestManagementAppIpColumn.eq(&app_ip))
        .filter(RaRequestManagementRequestTypeColumn.eq(request_type))
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(None);
    }

    let request_entity: RaRequestManagementModel = model.unwrap();
    Ok(Some((
        request_entity.last_request_time,
        request_entity.period_request_count,
    )))
}

pub async fn store_new_ra_request(
    db: &DatabaseConnection,
    app_id: String,
    vid: Option<u64>,
    last_request_time: u32,
    request_type: RaRequestManagementRequestType,
) -> Result<(), PersistenceStoreError> {
    let request_store = RaRequestManagementActiveModel {
        app_id: ActiveValue::Set(app_id),
        vehicle_id: ActiveValue::Set(vid),
        last_request_time: ActiveValue::Set(last_request_time),
        period_started_at: ActiveValue::Set(get_current_time_seconds()),
        period_request_count: ActiveValue::Set(1),
        ra_request_management_request_type: ActiveValue::Set(request_type),
        ..Default::default()
    };
    let _ = request_store.insert(db).await?;

    Ok(())
}

pub async fn update_status_by_vid_and_type(
    db: &DatabaseConnection,
    vid: u64,
    request_time: u32,
    new_count: i32,
    request_type: RaRequestManagementRequestType,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} (vid) ra request entry setting request time to {} and weekly count to {}",
        vid,
        request_time,
        new_count
    );
    RaRequestManagementEntity::update_many()
        .col_expr(
            RaRequestManagementLastRequestTime,
            Expr::value(request_time),
        )
        .col_expr(
            RaRequestManagementPeriodRequestCount,
            Expr::value(new_count),
        )
        .filter(RaRequestManagementVid.eq(vid))
        .filter(RaRequestManagementRequestTypeColumn.eq(request_type))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn update_status_by_app_ip_and_type(
    db: &DatabaseConnection,
    app_id: String,
    request_time: u32,
    new_count: i32,
    request_type: RaRequestManagementRequestType,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} (app ip) ra request entry setting request time to {} and weekly count to {}",
        app_id,
        request_time,
        new_count
    );
    RaRequestManagementEntity::update_many()
        .col_expr(
            RaRequestManagementLastRequestTime,
            Expr::value(request_time),
        )
        .col_expr(
            RaRequestManagementPeriodRequestCount,
            Expr::value(new_count),
        )
        .filter(RaRequestManagementAppIpColumn.eq(app_id))
        .filter(RaRequestManagementRequestTypeColumn.eq(request_type))
        .exec(db)
        .await?;

    Ok(())
}
