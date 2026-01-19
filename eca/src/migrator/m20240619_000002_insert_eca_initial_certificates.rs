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

// m20240619_000002_insert_eca_initial_certificates.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240619_000002_insert_eca_initial_certificates" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 1: Store initial ra certificate
            INSERT INTO eca_certificates (name, insert_date, file) 
            VALUES (
                'eca_private_key', 
                NOW(), 
                0xCCABF934100DEC1DCF8E8B5095A64EF6FF2F88C0AB5F20614B1AA45696F1360B
            );

            -- Step 1: Store initial ra private key
            INSERT INTO eca_certificates (name, insert_date, file) 
            VALUES (
                'eca_public_uncompressed', 
                NOW(), 
                0x78441C4A6F557205E9D1E8E0D58292053914F43A3CAAB22B953EA35CA13809E977A00C44620C4A4705354ADA96F7872AC8D5F59B27AF2485D9676F76015783D4
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
