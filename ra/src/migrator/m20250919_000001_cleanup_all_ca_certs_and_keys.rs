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

// m20250919_000001_cleanup_all_ca_certs_and_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250919_000001_cleanup_all_ca_certs_and_keys" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Clean up ra_certificates table
            DELETE FROM ra_certificates;

            -- Clean up electors_store table
            DELETE FROM electors_store;

            -- Clean up ccf_store table
            DELETE FROM ccf_store;

            -- Clean up composite_crl_store table
            DELETE FROM composite_crl_store;

            -- Clean up crl_store table
            DELETE FROM crl_store;

            -- Clean up ctl_store table
            DELETE FROM ctl_store;
            "#,
        )
        .await?;

        Ok(())
    }
}
