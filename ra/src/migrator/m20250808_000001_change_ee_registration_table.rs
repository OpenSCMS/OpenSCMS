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

// m20250808_000001_change_ee_registration_table.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250808_000001_change_ee_registration_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
                ALTER TABLE ee_registration
                DROP COLUMN vehicle_id;

                ALTER TABLE ee_registration
                DROP COLUMN issuer_id;

                ALTER TABLE ee_registration
                ADD COLUMN status ENUM('Registered', 'Enrolled', 'Successor-Enrolled', 'Blocked', 'Provisioning', 'Deleted') NOT NULL;

                ALTER TABLE ee_registration
                ADD COLUMN device_type ENUM('OBU', 'RSU') NOT NULL;

                ALTER TABLE ee_registration
                ADD COLUMN device_id BIGINT UNSIGNED NOT NULL;

                ALTER TABLE ee_registration
                ADD COLUMN public_key VARCHAR(255) NOT NULL;

                ALTER TABLE ee_registration
                ADD COLUMN created_time DATETIME NOT NULL;

                ALTER TABLE ee_registration
                ADD COLUMN updated_time DATETIME NOT NULL;
            "#,
        )
        .await?;

        Ok(())
    }
}
