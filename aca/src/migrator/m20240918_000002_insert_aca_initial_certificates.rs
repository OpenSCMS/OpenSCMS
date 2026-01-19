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

// m20240918_000002_insert_aca_initial_certificates.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240918_000002_insert_aca_initial_certificates" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 1: Store initial aca certificate
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_private_key', 
                NOW(), 
                0x98F1C8E3EDA3D3F8E7FC6E712E870AAEAC795CA0100AB600D87A845D625AC2DD
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_public_uncompressed', 
                NOW(), 
                0x2BEC66080C3BC259E5E964E3A20E4107200525B9E91BFEBF340715618ED50D4F7235F2EB93B9102205E6FA8EF5FA9C07AA6B836821384934B85B59F8E57831AF
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x80030080BAC70087DC64F29000810F6163612E6C6773636D732E74657374010203000026F7283C84000A8080832BEC66080C3BC259E5E964E3A20E4107200525B9E91BFEBF340715618ED50D4F8080DD009B18F03D62C97005CD96ED00D03D5534AA9EA30910AA3CA9725602C6BF94D955AB3BE81E3F11C4A104255B9E1DB564F6DF244A9F83728F6BCC746FF88BC3
            );

            -- Step 1: Store root ca certificate
            INSERT INTO aca_certificates (name, insert_date, file)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100008112726F6F7463612E6C6773636D732E74657374010203000026F0953986000A808083B1B194821CDCEEBB0666AEA13FC50771C8955FC3398F6BCD3E2494DA6D39C28C80808090A2310BE7BD045F122D1A2E77FD694B37DBAA675DE48C73C3475302CE8C94CA4C6D76D8AB64B40388E8AB3C600291997FE9F9415D11A5402D4AB3CD1E76BA
            );

            -- Step 1: Store ICA certificate
            INSERT INTO aca_certificates (name, insert_date, file)
            VALUES (
                'intermediate_ca_cert',
                NOW(),
                0x80030080E877AD45927D6BB100810F6963612E6C6773636D732E74657374010203000026F0958486000A808083C2273953A5AEFAB791DF995FCBBF3AB2CA90A9CA95703BB407CDE3517403FFF380804326405270E5281D000AF56ED6CF1BE11204A10F600A9BB528C4D380F1A8CC1073F24382974E5F0733259E7434EA51EA05878FACCA212417A06EFA7E10C897A1
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
