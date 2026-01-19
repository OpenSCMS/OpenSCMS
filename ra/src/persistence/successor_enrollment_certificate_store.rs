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

use crate::entities::sea_orm_active_enums::EnrollmentStatus;
use crate::entities::successor_enrollment_certificate_store::ActiveModel as SuccessorEnrollmentCertStoreActiveModel;
use crate::entities::successor_enrollment_certificate_store::Column::EnrollmentStatus as StatusColumn;
use crate::entities::successor_enrollment_certificate_store::Column::ErrorCode as ErrorCodeColumn;
use crate::entities::successor_enrollment_certificate_store::Column::File as FileColumn;
use crate::entities::successor_enrollment_certificate_store::Column::FileName as FileNameColumn;
use crate::entities::successor_enrollment_certificate_store::Column::HashId as HashIdColumn;
use crate::entities::successor_enrollment_certificate_store::Column::StatusCode as StatusCodeColumn;
use crate::entities::successor_enrollment_certificate_store::Entity as SuccessorEnrollmentCertStoreEntity;
use crate::entities::successor_enrollment_certificate_store::Model as SuccessorEnrollmentCertStoreModel;

use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};

use sea_orm::QueryFilter;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait};

pub async fn store_enrollment_request_to_be_processed(
    db: &DatabaseConnection,
    hash_id: String,
    request_time_period: u64,
    request_message: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    let enrollment_request = SuccessorEnrollmentCertStoreActiveModel {
        hash_id: ActiveValue::Set(hash_id),
        request_time_period: ActiveValue::Set(request_time_period),
        request_message: ActiveValue::Set(request_message),
        enrollment_status: ActiveValue::Set(EnrollmentStatus::ToBeProcessed),
        ..Default::default()
    };
    let _ = enrollment_request.insert(db).await?;

    Ok(())
}

pub async fn fetch_request_message_from_enrollment_request_by_hash_id(
    db: &DatabaseConnection,
    hash_id: String,
) -> Result<Vec<u8>, PersistenceLoadError> {
    let successor_enrollment_request = fetch_enrollment_request_by_hash_id_and_status(
        db,
        hash_id,
        EnrollmentStatus::ToBeProcessed,
    )
    .await?;
    Ok(successor_enrollment_request.request_message)
}

pub async fn download_enrollment_certificate(
    db: &DatabaseConnection,
    filename: String,
) -> Result<(String, Vec<u8>, Option<i32>, Option<i32>), PersistenceLoadError> {
    log::debug!(
        "Downloading successor enrollment certificate for filename {}",
        filename
    );
    let successor_enrollment_request =
        fetch_enrollment_request_by_filename(db, filename.clone()).await?;

    let hash_id = successor_enrollment_request.hash_id;
    let file = successor_enrollment_request.file;
    let error_code = successor_enrollment_request.error_code;
    let status_code = successor_enrollment_request.status_code;

    if (hash_id.is_empty() || file.is_none()) && (error_code.is_none() && status_code.is_none()) {
        return Err(PersistenceLoadError::new(
            format!(
                "No certificate found for filename {} and no errors were reported",
                filename
            )
            .as_str(),
        ));
    }

    set_enrollment_status_by_hash_id(db, hash_id, EnrollmentStatus::Downloaded).await?;

    Ok((filename, file.unwrap_or(vec![]), error_code, status_code))
}

async fn fetch_enrollment_request_by_filename(
    db: &DatabaseConnection,
    filename: String,
) -> Result<SuccessorEnrollmentCertStoreModel, PersistenceLoadError> {
    log::debug!(
        "Fetching successor enrollment certificate request for filename {}",
        filename
    );

    let model: Option<SuccessorEnrollmentCertStoreModel> =
        SuccessorEnrollmentCertStoreEntity::find()
            .filter(FileNameColumn.eq(filename.clone()))
            .one(db)
            .await?;

    if model.is_none() {
        log::debug!(
            "No successor enrollment certificate request found for filename {}",
            filename
        );
        return Err(PersistenceLoadError::new(
            format!(
                "No successor certificate request register was found for filename {}",
                filename
            )
            .as_str(),
        ));
    }

    log::debug!(
        "Found successor enrollment certificate request for filename {}",
        filename
    );

    let successor_enrollment_request: SuccessorEnrollmentCertStoreModel = model.unwrap();
    Ok(successor_enrollment_request)
}

async fn fetch_enrollment_request_by_hash_id_and_status(
    db: &DatabaseConnection,
    hash_id: String,
    status: EnrollmentStatus,
) -> Result<SuccessorEnrollmentCertStoreModel, PersistenceLoadError> {
    let model: Option<SuccessorEnrollmentCertStoreModel> =
        SuccessorEnrollmentCertStoreEntity::find()
            .filter(HashIdColumn.eq(hash_id.clone()))
            .filter(StatusColumn.eq(status))
            .one(db)
            .await?;

    if model.is_none() {
        return Err(PersistenceLoadError::new(
            format!("No Entry {} found", hash_id).as_str(),
        ));
    }

    let successor_enrollment_request: SuccessorEnrollmentCertStoreModel = model.unwrap();
    Ok(successor_enrollment_request)
}

pub async fn set_enrollment_certificate_by_hash_id(
    db: &DatabaseConnection,
    hash_id: String,
    file_name: String,
    file: Vec<u8>,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} successor enrollment certificate setting certificate file and name {}",
        hash_id,
        file_name
    );

    SuccessorEnrollmentCertStoreEntity::update_many()
        .col_expr(FileNameColumn, Expr::value(file_name))
        .col_expr(FileColumn, Expr::value(file))
        .col_expr(StatusColumn, Expr::value(EnrollmentStatus::Processed))
        .filter(HashIdColumn.eq(hash_id))
        .filter(StatusColumn.eq(EnrollmentStatus::ToBeProcessed))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn set_enrollment_certificate_error_by_hash_id(
    db: &DatabaseConnection,
    hash_id: String,
    status_code: i32,
    error_code: i32,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} successor enrollment certificate setting status code {} and error code {}",
        hash_id,
        status_code,
        error_code
    );

    SuccessorEnrollmentCertStoreEntity::update_many()
        .col_expr(StatusCodeColumn, Expr::value(status_code))
        .col_expr(ErrorCodeColumn, Expr::value(error_code))
        .filter(HashIdColumn.eq(hash_id))
        .exec(db)
        .await?;

    Ok(())
}

async fn set_enrollment_status_by_hash_id(
    db: &DatabaseConnection,
    hash_id: String,
    to_status: EnrollmentStatus,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} successor enrollment certificate setting status to {:?}",
        hash_id,
        to_status
    );

    SuccessorEnrollmentCertStoreEntity::update_many()
        .col_expr(StatusColumn, Expr::value(to_status))
        .filter(HashIdColumn.eq(hash_id))
        .exec(db)
        .await?;

    Ok(())
}
