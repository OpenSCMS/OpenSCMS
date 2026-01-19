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

// m20250224_000001_update_aca_cert_and_clean_ccf_store.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250224_000001_update_aca_cert_and_clean_ccf_store" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            DELETE FROM ra_certificates WHERE name = 'aca_certificate';

            INSERT INTO ra_certificates (name, insert_date, file, cert_id) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x80030080A5E944214BE5B9E60082105073AE84F5F6B4839512FFAB1DE91E28010203000027C920BA86000A80808332681AEC7978A221ACCEA44504C3B1A4684E257331CF88DC9A852B5A0E5B4C8F8080B5367BA4DD3BCA06471DE977D73BC62C380B05CBB991D167FAA169303DEF3B42F4BDAD37D85DD5DDFD84D6C01FB9EB971A1A2D8C65D5FB2D54FA4CCEE3087143,
                'c40c706f988aa5d9'
            );

            DELETE FROM ctl_store;
            DELETE FROM ccf_store;
            
            "#,
        )
        .await?;

        Ok(())
    }
}
