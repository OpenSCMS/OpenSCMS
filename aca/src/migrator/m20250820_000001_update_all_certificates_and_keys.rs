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

// m20250820_000001_update_all_certificates_and_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250820_000001_update_all_certificates_and_keys" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Clean up table
            DELETE FROM aca_certificates;
        
            -- Step 1: Store initial aca certificate
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_private_key', 
                NOW(), 
                0x8B86A351D1DAE075E5E5D5A4496DBF6E817C34B878BF3EB44838C8F230FDBFAC
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_public_uncompressed', 
                NOW(), 
                0xBB3414744554D998FF6ED1625163F0E3803EA5173BA6341850A2E53AFAA581DB81297961A2EC511E77D3B7CBE4954BE3773B3DDE1E133FF748F2DD03503EC5A3
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x800300803631EEE885298CCC108210E5FB954C49C8C815046B9A871EE8CCD2010203000028B1A04C86000A01018001238003850002808083BB3414744554D998FF6ED1625163F0E3803EA5173BA6341850A2E53AFAA581DB8080AB652021AE92D6E9D98156B81600754A804286E7BE51ECC4DA9B2F0A52EF592FEC7F7A68CCC7ADD0FD4024F6E6849B0882F0B578300C8BD129BD45022EC22ACC
            );

            -- Step 1: Store root ca certificate
            INSERT INTO aca_certificates (name, insert_date, file)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100108210FBA26C725AA2FCA027FC9ED8079DBDD7010203000028B19F7E86000A010180012380038100028080828FA35C637BEDDF1B711ED8EE5C742CEC434CD6D176289DB1A425A8E26586A0A78080D12C28F474AB8EE25C755F6A7A8C0B6B808B57ACE6725513ED9019EEAA70709F2543644C0262458F52213823929ABBBF4DE61B4E131ED7546C472FDCF346E45B
            );

            -- Step 1: Store ICA certificate
            INSERT INTO aca_certificates (name, insert_date, file)
            VALUES (
                'intermediate_ca_cert',
                NOW(),
                0x80030080D163E79063BEEF6610821064B8CA758AF9D051E6AEC5327775A611010203000028B1A00986000A01018001238003830002808083BF5D281BA78D6827F0E5FE7B7855A6AC0D3D86FC840EA7A45EC24002E6B9C43D808030ABE803EFF631EBDDF1D71EBD7918E251A3B82669712F125F8BBF3D7AC31DB998C265D16F3A8C8320CD77015B85A6533F243EFDA8963EF444C10D2431894103
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
