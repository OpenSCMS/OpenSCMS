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

// m20250818_000001_update_all_eca_certs_and_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250818_000001_update_all_eca_certs_and_keys" // Make sure this matches with the file name
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
            DELETE FROM eca_certificates;

            -- Store initial rootca certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100108210FBA26C725AA2FCA027FC9ED8079DBDD7010203000028B19F7E86000A010180012380038100028080828FA35C637BEDDF1B711ED8EE5C742CEC434CD6D176289DB1A425A8E26586A0A78080D12C28F474AB8EE25C755F6A7A8C0B6B808B57ACE6725513ED9019EEAA70709F2543644C0262458F52213823929ABBBF4DE61B4E131ED7546C472FDCF346E45B
            );

            -- Store initial ica certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'intermediate_ca_cert',
                NOW(),
                0x80030080D163E79063BEEF6610821064B8CA758AF9D051E6AEC5327775A611010203000028B1A00986000A01018001238003830002808083BF5D281BA78D6827F0E5FE7B7855A6AC0D3D86FC840EA7A45EC24002E6B9C43D808030ABE803EFF631EBDDF1D71EBD7918E251A3B82669712F125F8BBF3D7AC31DB998C265D16F3A8C8320CD77015B85A6533F243EFDA8963EF444C10D2431894103
            );

            -- Store initial eca certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_certificate',
                NOW(),
                0x800300803631EEE885298CCC108210A1A716F77017678846039E7FB263675B010203000028B1A04C86000A01018001238003840002808083EDBFB3839538FB8625C9DCEE2F859150386DB09446D62AFEEBE52786E75B19ED8080C77E222553B2063223CC4CDC421EE8C6142BF6BC9B24C039EA8DD53695A136BC139862F47F97E1C9EF7A82C946A5C6245E747D6ED6094AB7DFC6E7563F2517E0
            );

            -- Store initial eca public key
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_public_uncompressed',
                NOW(),
                0xEDBFB3839538FB8625C9DCEE2F859150386DB09446D62AFEEBE52786E75B19ED3AE7B5FB71D075E30299A4E2DBF0097ED02512181A8BDA58997660B1BF5405CB
            );

            -- Store initial aca public key
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_private_key',
                NOW(),
                0xEF6DA24CBB6E039BEC9E501FE74D8E46B3A6AF45CF847E8BCFD431EE74D55EAB
            );
            
            "#,
        )
        .await?;

        Ok(())
    }
}
