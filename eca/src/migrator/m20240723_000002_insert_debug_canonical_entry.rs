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

// m20240723_000002_insert_debug_canonical_entry.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240723_000002_insert_debug_canonical_entry" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 1: Store debug canonical pair <canonical_id, canonical_public_key>
            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                'ABCDEFGHIJKLMNOP', 
                NOW(), 
                0x3D1DC4402681F1A5F28927FF9CEB04352220871D09E2B564D12F75E287B2BA1C7A3D204A06E63697B8817E070A13C81FAC60A7F423353F8E0AAA3ECC7BA936A8
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
