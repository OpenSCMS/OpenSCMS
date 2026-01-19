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

use crate::entities::caterpillar::ActiveModel as CaterpillarActiveModel;
use crate::entities::caterpillar::Column::CaterpillarStatus as CaterpillarStatusColumn;
use crate::entities::caterpillar::Column::Id as CaterpillarIdColumn;
use crate::entities::caterpillar::Entity as CaterpillarEntity;
use crate::entities::sea_orm_active_enums::ExpType;
use crate::raconfig::RaConfig;
use scmscommon::core_types::ExpansionType;
use scmscommon::core_types::caterpillar::{
    Caterpillar, CaterpillarObk, CaterpillarStatus, CaterpillarUbk,
};
use scmscommon::errors::{PersistenceLoadError, PersistenceStoreError};
use sea_orm::sea_query::Expr;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, EntityTrait, QueryFilter, QueryResult, Statement,
};
use serde_json;

pub trait CaterpillarPersistence {
    async fn store(&self, db: &DatabaseConnection) -> Result<(), PersistenceStoreError>;
}

impl CaterpillarPersistence for CaterpillarUbk {
    async fn store(&self, db: &DatabaseConnection) -> Result<(), PersistenceStoreError> {
        let serialized = serde_json::to_string(self).unwrap();
        let res = serde_json::from_str(serialized.as_str())?;
        let exp_type = match &self.exp_type {
            ExpansionType::Unified => ExpType::Ubk,
            ExpansionType::Compact => ExpType::Cubk,
            _ => {
                return Err(PersistenceStoreError::new(&format!(
                    "Not supported ExpansionType for CaterpillarUbk: {:?}",
                    self.exp_type
                )));
            }
        };

        let caterpillar_to_store = CaterpillarActiveModel {
            exp_type: ActiveValue::Set(exp_type),
            caterpillar: ActiveValue::Set(res),
            ..Default::default()
        };
        let _ = caterpillar_to_store.insert(db).await?;

        Ok(())
    }
}

impl CaterpillarPersistence for CaterpillarObk {
    async fn store(&self, db: &DatabaseConnection) -> Result<(), PersistenceStoreError> {
        let serialized = serde_json::to_string(self).unwrap();
        let res = serde_json::from_str(serialized.as_str())?;

        let caterpillar_to_store = CaterpillarActiveModel {
            exp_type: ActiveValue::Set(ExpType::Obk),
            caterpillar: ActiveValue::Set(res),
            ..Default::default()
        };
        let _ = caterpillar_to_store.insert(db).await?;

        Ok(())
    }
}

impl CaterpillarPersistence for Caterpillar {
    async fn store(&self, db: &DatabaseConnection) -> Result<(), PersistenceStoreError> {
        match &self {
            Caterpillar::Obk(x) => x.store(db).await,
            Caterpillar::Ubk(x) => x.store(db).await,
        }
    }
}

pub async fn fetch_caterpillars_if_enough(
    db: &DatabaseConnection,
    exp_type: ExpansionType,
) -> Result<(Vec<i32>, Vec<Caterpillar>), PersistenceLoadError> {
    let min_number_certs = RaConfig::global().min_number_certs_ra as u64;

    log::debug!(
        "Fetching caterpillars: threshold {} and exp_type: {}",
        min_number_certs,
        exp_type
    );

    let stmt = format!(
        "CALL SelectCaterpillarsIfEnough({}, '{}');",
        min_number_certs, exp_type
    );

    let result: Vec<QueryResult> = db
        .query_all(Statement::from_string(DatabaseBackend::MySql, &stmt))
        .await?;

    let mut caterpillars = Vec::new();
    let mut caterpillars_ids = Vec::new();
    for row in result {
        let id = row.try_get_by_index::<i32>(4).unwrap_or_default();
        let exp_type = row.try_get_by_index::<String>(3).unwrap_or_default();
        let caterpillar_json = row
            .try_get_by_index::<serde_json::Value>(1)
            .unwrap_or_default();

        let caterpillar = match exp_type.as_str() {
            "Ubk" => {
                let caterpillar: CaterpillarUbk = match serde_json::from_value(caterpillar_json) {
                    Ok(caterpillar) => caterpillar,
                    Err(e) => {
                        return Err(PersistenceLoadError::new(&format!(
                            "Failed to convert caterpillar UBK: {}",
                            e
                        )));
                    }
                };
                Caterpillar::Ubk(caterpillar)
            }
            "Obk" => {
                let caterpillar: CaterpillarObk = match serde_json::from_value(caterpillar_json) {
                    Ok(caterpillar) => caterpillar,
                    Err(e) => {
                        return Err(PersistenceLoadError::new(&format!(
                            "Failed to convert caterpillar OBK: {}",
                            e
                        )));
                    }
                };
                Caterpillar::Obk(caterpillar)
            }
            "Cubk" => {
                let caterpillar: CaterpillarUbk = match serde_json::from_value(caterpillar_json) {
                    Ok(caterpillar) => caterpillar,
                    Err(e) => {
                        return Err(PersistenceLoadError::new(&format!(
                            "Failed to convert caterpillar CUBK: {}",
                            e
                        )));
                    }
                };
                Caterpillar::Ubk(caterpillar)
            }
            _ => {
                return Err(PersistenceLoadError::new(&format!(
                    "Not supported ExpansionType: {:?}",
                    exp_type
                )));
            }
        };
        caterpillars.push(caterpillar);
        caterpillars_ids.push(id);
    }

    log::debug!(
        "Found {:?} caterpillars to be processed ids ({:?}).",
        caterpillars.len(),
        caterpillars_ids
    );
    Ok((caterpillars_ids, caterpillars))
}

pub async fn update_status_by_ids(
    db: &DatabaseConnection,
    ids: Vec<i32>,
    status: CaterpillarStatus,
) -> Result<(), PersistenceLoadError> {
    log::debug!(
        "Updating {} caterpillars setting caterpillar_status {}",
        ids.len(),
        status
    );
    CaterpillarEntity::update_many()
        .col_expr(CaterpillarStatusColumn, Expr::value(status.to_string()))
        .filter(CaterpillarIdColumn.is_in(ids))
        .exec(db)
        .await?;

    Ok(())
}

pub async fn delete_by_ids(
    db: &DatabaseConnection,
    ids: Vec<i32>,
) -> Result<(), PersistenceLoadError> {
    log::debug!("Deleting {} caterpillars", ids.len(),);
    CaterpillarEntity::delete_many()
        .filter(CaterpillarIdColumn.is_in(ids))
        .exec(db)
        .await?;

    Ok(())
}
