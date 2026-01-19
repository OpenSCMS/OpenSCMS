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

use crate::entities::ee_registration::ActiveModel as EeRegistrationActiveModel;
use crate::entities::ee_registration::Column::CanonicalId as EeRegistrationCanonicalId;
use crate::entities::ee_registration::Column::DeviceId as EeRegistrationDeviceId;
use crate::entities::ee_registration::Column::PublicKey as EeRegistrationPublicKey;
use crate::entities::ee_registration::Entity as EeRegistrationEntity;
use crate::entities::ee_registration::Model as EeRegistrationModel;

use crate::entities::sea_orm_active_enums::DeviceType;
use crate::entities::sea_orm_active_enums::Status;

use scmscommon::EeRegistrationDeviceType;
use scmscommon::errors::PersistenceLoadError;

use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue;
use sea_orm::ColumnTrait;
use sea_orm::DatabaseConnection;
use sea_orm::EntityTrait;
use sea_orm::IntoActiveModel;
use sea_orm::QueryFilter;

pub async fn fetch_ee_registration_by_device_id(
    device_id: u64,
    db: &DatabaseConnection,
) -> Result<Option<EeRegistrationModel>, PersistenceLoadError> {
    let model: Option<EeRegistrationModel> = EeRegistrationEntity::find()
        .filter(EeRegistrationDeviceId.eq(device_id))
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(None);
    }

    let certificate_entity: EeRegistrationModel = model.unwrap();
    Ok(Some(certificate_entity))
}

pub async fn fetch_ee_registration_by_canonical_id(
    canonical_id: String,
    db: &DatabaseConnection,
) -> Result<Option<EeRegistrationModel>, PersistenceLoadError> {
    let model: Option<EeRegistrationModel> = EeRegistrationEntity::find()
        .filter(EeRegistrationCanonicalId.eq(canonical_id))
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(None);
    }

    let certificate_entity: EeRegistrationModel = model.unwrap();
    Ok(Some(certificate_entity))
}

pub async fn fetch_ee_registration_by_public_key(
    public_key: String,
    db: &DatabaseConnection,
) -> Result<Option<EeRegistrationModel>, PersistenceLoadError> {
    let model: Option<EeRegistrationModel> = EeRegistrationEntity::find()
        .filter(EeRegistrationPublicKey.eq(public_key))
        .one(db)
        .await?;

    if model.is_none() {
        return Ok(None);
    }

    let certificate_entity: EeRegistrationModel = model.unwrap();
    Ok(Some(certificate_entity))
}

pub async fn patch_status_ee_registration(
    canonical_id: String,
    status: Status,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Patching EE registration status for canonical id {:?} to {:?}",
        canonical_id,
        status
    );

    let ee_registration_model: EeRegistrationActiveModel = EeRegistrationEntity::find()
        .filter(EeRegistrationCanonicalId.eq(canonical_id))
        .one(db)
        .await?
        .ok_or_else(|| PersistenceLoadError::new("EE registration not found"))?
        .into();

    let mut ee_registration_model: EeRegistrationActiveModel =
        ee_registration_model.into_active_model();
    ee_registration_model.status = ActiveValue::Set(status);
    ee_registration_model.updated_time = ActiveValue::Set(chrono::Utc::now().naive_utc());

    ee_registration_model.update(db).await?;

    Ok(())
}

pub async fn patch_status_ee_registration_by_device_id(
    device_id: u64,
    status: Status,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Patching EE registration status for device id {:?} to {:?}",
        device_id,
        status
    );

    let ee_registration_model: EeRegistrationActiveModel = EeRegistrationEntity::find()
        .filter(EeRegistrationDeviceId.eq(device_id))
        .one(db)
        .await?
        .ok_or_else(|| PersistenceLoadError::new("EE registration not found"))?
        .into();

    let mut ee_registration_model: EeRegistrationActiveModel =
        ee_registration_model.into_active_model();
    ee_registration_model.status = ActiveValue::Set(status);
    ee_registration_model.updated_time = ActiveValue::Set(chrono::Utc::now().naive_utc());

    ee_registration_model.update(db).await?;

    Ok(())
}

pub async fn store_new_registration(
    device_id: u64,
    device_type: EeRegistrationDeviceType,
    canonical_id: String,
    public_key: String,
    db: &DatabaseConnection,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "New EE registration: {:?} - Inserting a new entry with device id {:?}",
        canonical_id,
        device_id
    );

    let device_type_ent: DeviceType = match device_type {
        EeRegistrationDeviceType::OBU => DeviceType::Obu,
        EeRegistrationDeviceType::RSU => DeviceType::Rsu,
    };

    let ee_registration_model = EeRegistrationActiveModel {
        device_id: ActiveValue::Set(device_id),
        device_type: ActiveValue::Set(device_type_ent),
        canonical_id: ActiveValue::Set(canonical_id),
        public_key: ActiveValue::Set(public_key),
        status: ActiveValue::Set(Status::Registered),
        created_time: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        updated_time: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        blocked: ActiveValue::Set(false as i8),
        ..Default::default()
    };

    ee_registration_model.insert(db).await?;

    Ok(())
}
