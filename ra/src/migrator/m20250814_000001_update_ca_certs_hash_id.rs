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

// m20250814_000001_update_ca_certs_hash_id.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250814_000001_update_ca_certs_hash_id" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            UPDATE ra_certificates SET cert_id = 'd863e118f8a4afb7'  WHERE name = 'ra_certificate';

            UPDATE ra_certificates SET cert_id = '033f821e76587884'  WHERE name = 'root_ca_cert';

            UPDATE ra_certificates SET cert_id = 'a5e944214be5b9e6'  WHERE name = 'intermediate_ca_cert';

            UPDATE ra_certificates SET cert_id = '5d215da31efc7f75'  WHERE name = 'eca_certificate';

            UPDATE ra_certificates SET cert_id = 'df158f6cb0f1e4f2'  WHERE name = 'aca_certificate';
            "#,
        )
        .await?;

        Ok(())
    }
}
