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

// m20240828_000001_insert_signed_ra_certificate.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240828_000001_insert_signed_ra_certificate" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Store ra private key
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_private_key',
                NOW(),
                0xE53B5A752B5826EDE5A65F49A7924F06D854F187D806676C92857CADF8CDA2E7
            );

            -- Store ra public key uncompressed
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_public_uncompressed',
                NOW(),
                0xB53EE6070CB45624F4DD11F98E444685FDD6403FD1B735A29C93856E2C5FC4DE1B54B2CB41137A067E2A42B8F88131DDE1B2F0E665B4CBD092BC17FFAFDF7297
            );

            -- Store initial ra certificate
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_certificate',
                NOW(),
                0x80030080F099F499793FE1C90082104E51BC48F685B32B700FEE491321AE47010203000026DB562A840018808084B53EE6070CB45624F4DD11F98E444685FDD6403FD1B735A29C93856E2C5FC4DE1B54B2CB41137A067E2A42B8F88131DDE1B2F0E665B4CBD092BC17FFAFDF72978080D8B22C6AB70CA09C490F3C93062DA8F92C6DA8466C5EE44759249FF902489A898879F1A2DA727E8BD4028E00127C27D0C67D5655E632D4E3984ED233B8CE3E99
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
