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

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240403_000002_replace_new_caterpillar_status" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 3: Drop the old caterpillar_status column
            ALTER TABLE caterpillar
            DROP COLUMN caterpillar_status;

            -- Step 4: Rename the new caterpillar_status_enum column to caterpillar_status
            ALTER TABLE caterpillar
            CHANGE COLUMN caterpillar_status_enum caterpillar_status ENUM('ToBeProcessed', 'Processing', 'Processed') NOT NULL DEFAULT 'ToBeProcessed';
            "#,
        )
        .await?;

        Ok(())
    }

    // async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    //     manager.get_connection().execute_unprepared("").await?;
    //     Ok(())
    // }
}
