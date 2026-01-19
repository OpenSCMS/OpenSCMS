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
        "m20240403_000001_populate_caterpillar_status_enum" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 2: Update the values of the new enum column based on the existing string values
            UPDATE caterpillar
            SET caterpillar_status_enum = CASE
                WHEN caterpillar_status = '"ToBeProcessed"' THEN 'ToBeProcessed'
                WHEN caterpillar_status = 'ToBeProcessed' THEN 'ToBeProcessed'
                WHEN caterpillar_status = 'Processing' THEN 'Processing'
                ELSE 'Processed'
            END;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                "
            -- Undo Step 2: Reset the values of the new enum column to the default value
            UPDATE caterpillar
            SET caterpillar_status_enum = 'ToBeProcessed';
            ",
            )
            .await?;

        Ok(())
    }
}
