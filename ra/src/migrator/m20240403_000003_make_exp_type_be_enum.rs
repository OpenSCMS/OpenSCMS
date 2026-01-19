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
        "m20240403_000003_make_exp_type_be_enum" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 1: Add a new enum column
            ALTER TABLE caterpillar
            ADD COLUMN exp_type_enum ENUM('Obk', 'Ubk', 'Cubk', 'ObkAcpc', 'UbkAcpc') NOT NULL DEFAULT 'Obk';

            -- Step 2: Update the values of the new enum column based on the existing string values
            UPDATE caterpillar
            SET exp_type_enum = CASE
                WHEN exp_type = 'Obk' THEN 'Obk'
                WHEN exp_type = 'Ubk' THEN 'Ubk'
                WHEN exp_type = 'Cubk' THEN 'Cubk'
                WHEN exp_type = 'ObkAcpc' THEN 'ObkAcpc'
                WHEN exp_type = 'UbkAcpc' THEN 'UbkAcpc'
            END;

            -- Step 3: Drop the old exp_type column
            ALTER TABLE caterpillar
            DROP COLUMN exp_type;

            -- Step 4: Rename the new exp_type_enum column to exp_type
            ALTER TABLE caterpillar
            CHANGE COLUMN exp_type_enum exp_type ENUM('Obk', 'Ubk', 'Cubk', 'ObkAcpc', 'UbkAcpc') NOT NULL DEFAULT 'Obk';
            "#,
        )
        .await?;

        Ok(())
    }
}
