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

use sea_orm::{ConnectionTrait, Database, DatabaseConnection, DbBackend, DbErr, Statement};
use std::sync::Arc;

use crate::mysql_config;
pub struct AppState {
    pub app_name: String,
    pub db: DatabaseConnection,
    pub celery_app: Option<Arc<celery::Celery>>,
}

pub async fn setup_db() -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(mysql_config::get_url()).await?;
    log::debug!(
        "Connected to the database Server {}",
        mysql_config::get_url()
    );

    let db = match db.get_database_backend() {
        DbBackend::MySql => {
            db.execute(Statement::from_string(
                db.get_database_backend(),
                format!(
                    "CREATE DATABASE IF NOT EXISTS `{}`;",
                    mysql_config::get_db_name()
                ),
            ))
            .await?;

            let url = format!(
                "{}/{}",
                mysql_config::get_url(),
                mysql_config::get_db_name()
            );
            log::debug!("Connecting to the database Instance {}", url);
            Database::connect(&url).await?
        }
        _ => panic!("The SCMS only accepts MySQL"),
    };

    Ok(db)
}

pub async fn connect_to_db() -> Result<DatabaseConnection, DbErr> {
    let url = format!(
        "{}/{}",
        mysql_config::get_url(),
        mysql_config::get_db_name()
    );
    log::debug!("Connecting to the database Instance {}", url);
    Database::connect(&url).await
}
