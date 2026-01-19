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

// m20240924_000001_insert_ra_enc_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240924_000001_insert_ra_enc_keys" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Update RA Certificate
            DELETE FROM ra_certificates WHERE name = 'ra_certificate';

            -- Store ra private key
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_enc_private_key',
                NOW(),
                0x9467DB701C173EFEEED5395557713C8667C87DD2798300B835B9F6FB633DCEC4
            );

            -- Store ra enc public key uncompressed
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_enc_public_uncompressed',
                NOW(),
                0x0868C250F6BAC467734AE2F4CD14E675E55A37BDF69C5C4186CDEDC8E7C505E5852B92641637A80117BE663C927111198BAAAC225C5998C4A36B8E11CB121844
            );

            -- Store ra certificate
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_certificate',
                NOW(),
                0x80030080BAC70087DC64F29001810E72612E6C6773636D732E74657374010203000026FF29BA86000A0080820868C250F6BAC467734AE2F4CD14E675E55A37BDF69C5C4186CDEDC8E7C505E5808083DD38AE46AF9B3BFBEA2653A191DAA6E3D022B67462D2B09D403AA7EF02E2F56F8080E1BAC542A641085A5DF60133274C3CC591F8019ADA326A29D42DAE61A060B6536E0177D160B801F86D38F393014C1183B80E653D216B4CEE46152476A4100046
            );
            
            "#,
        )
        .await?;

        Ok(())
    }
}
