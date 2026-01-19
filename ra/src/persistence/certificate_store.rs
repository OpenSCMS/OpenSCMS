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

use crate::entities::certificate_store::ActiveModel as CertificateStoreActiveModel;
use crate::entities::certificate_store::Column::Downloaded as CertificateStoreDownloadedColumn;
use crate::entities::certificate_store::Column::HashId as HashIdColumn;
use crate::entities::certificate_store::Column::Id as CertificateStoreIdColumn;
use crate::entities::certificate_store::Entity as CertificateStoreEntity;
use scmscommon::errors::PersistenceStoreError;
use sea_orm::QueryFilter;
use sea_orm::sea_query::Expr;
use sea_orm::{ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait};

pub struct CertificateFile {
    pub id: i32,
    pub hash_id: String,
    pub index_i: u64,
    pub index_j: u64,
    pub certificate_binary: Vec<u8>,
}

pub async fn store_certificate(
    db: &DatabaseConnection,
    hash_id: String,
    index_i: u64,
    index_j: u64,
    certificate_binary: Vec<u8>,
) -> Result<(), PersistenceStoreError> {
    let certificate_file_to_store = CertificateStoreActiveModel {
        hash_id: ActiveValue::Set(hash_id),
        index_i: ActiveValue::Set(index_i),
        index_j: ActiveValue::Set(index_j),
        file: ActiveValue::Set(certificate_binary),
        ..Default::default()
    };
    let _ = certificate_file_to_store.insert(db).await?;

    Ok(())
}

pub async fn fetch_certificates_files_by_hash_id_and_i_value(
    db: &DatabaseConnection,
    hash_id: String,
    index_i: u64,
) -> Result<(Vec<CertificateFile>, bool), PersistenceStoreError> {
    let certificates_model = CertificateStoreEntity::find()
        .filter(HashIdColumn.eq(hash_id))
        .all(db)
        .await?;

    if certificates_model.len() == 1 {
        let certificate_model = &certificates_model[0];
        if certificate_model.index_i == 0 && certificate_model.index_j == 0 {
            // Case when it's a non butterfly request
            let certificate_file = CertificateFile {
                id: certificate_model.id,
                hash_id: certificate_model.hash_id.clone(),
                index_i: certificate_model.index_i,
                index_j: certificate_model.index_j,
                certificate_binary: certificate_model.file.clone(),
            };
            // non butterfly request
            return Ok((vec![certificate_file], false));
        }
    }

    let mut certificate_files: Vec<CertificateFile> = Vec::new();
    for certificate_model in certificates_model {
        if certificate_model.index_i != index_i {
            // Applying index i filter here
            continue;
        }
        let certificate_file = CertificateFile {
            id: certificate_model.id,
            hash_id: certificate_model.hash_id.clone(),
            index_i: certificate_model.index_i,
            index_j: certificate_model.index_j,
            certificate_binary: certificate_model.file.clone(),
        };
        certificate_files.push(certificate_file);
    }
    // butterfly request
    Ok((certificate_files, true))
}

pub async fn update_set_downloaded_true(
    db: &DatabaseConnection,
    certificates_ids: Vec<i32>,
) -> Result<(), PersistenceStoreError> {
    log::debug!(
        "Updating ({:?}) certificates setting downloaded to true",
        certificates_ids,
    );

    CertificateStoreEntity::update_many()
        .col_expr(CertificateStoreDownloadedColumn, Expr::value(1))
        .filter(CertificateStoreIdColumn.is_in(certificates_ids))
        .exec(db)
        .await?;

    Ok(())
}
