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

// m20241105_000002_add_cert_id.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241105_000002_add_cert_id" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            UPDATE ra_certificates SET cert_id = '7eed89085309e01d' WHERE name = 'ra_certificate';
            UPDATE ra_certificates SET cert_id = 'b00c8deb2075e694' WHERE name = 'root_ca_cert';
            UPDATE ra_certificates SET cert_id = '5b16e2699b48491b' WHERE name = 'intermediate_ca_cert';
            UPDATE ra_certificates SET cert_id = '5b3564dd243a5ff7' WHERE name = 'eca_certificate';

            -- Step 1: Store RootCa Certificate
            INSERT INTO ra_certificates (name, insert_date, file, cert_id) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x80030080A5E944214BE5B9E6008210D8987EC66C7278BE67075A446C4D386E01020300002727390D86000A8080830D611145A49CBC6DFDCC0338B90DB76C484030146D47B3580E54E727604261C280805564C5324DF2D8EFEB38AD6F74AAEF0D4ECA817018BE5EF4E9656BB12EC11B03B0734B65500E4EF08569442F004EF6B441F38418800F41A6D482CA032E0D9015,
                '3bf24fb045b7ec18'
            );

            "#,
        )
        .await?;

        Ok(())
    }
}
