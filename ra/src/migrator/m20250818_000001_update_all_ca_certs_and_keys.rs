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

// m20250818_000001_update_all_ca_certs_and_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250818_000001_update_all_ca_certs_and_keys" // Make sure this matches with the file name
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
            DELETE FROM ra_certificates;

            -- Store ra enc private key
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'ra_enc_private_key',
                NOW(),
                0x6964607A883B160BC5712F1A59606A240431F4EE4C6D7BF61F93377DDF336D70,
                ''
            );

            -- Store ra enc public key uncompressed
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'ra_enc_public_uncompressed',
                NOW(),
                0xB79FF17F059BAA90A2BF8401A6D4AF824AFCF78203B9D01FB58FD4C5948A9F3CF7FA739628AD5ADFF1F0D1AE16DD0DBBA3BF8007453C380B1C4B555F7F6F46F6,
                ''
            );

            -- Store ra private key
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'ra_private_key',
                NOW(),
                0x7805AC3C9CA8082EA7663102B1328412541639647D60A7F63FDC948958688F39,
                ''
            );

            -- Store ra public key uncompressed
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'ra_public_uncompressed',
                NOW(),
                0xAC755B830636493D81575B4F15FF114F8848D778C923A4CF5D805AA6D03337E628F5FF83EC1A9944B12E604E944A6EA10AAB0345B3289055EEB787CF50C626BD,
                ''
            );

            -- Store initial ra certificate
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'ra_certificate',
                NOW(),
                0x800300803631EEE885298CCC118210E61F790557A4CD9CE8B57B97348FE216010203000028B1A04C86000A010180012380038B0002008082B79FF17F059BAA90A2BF8401A6D4AF824AFCF78203B9D01FB58FD4C5948A9F3C808083AC755B830636493D81575B4F15FF114F8848D778C923A4CF5D805AA6D03337E68080B6D1C11D8656D8643DB8CA7CBFA80688A6EE5BE12AD926B06020B421B54EBF78E4B1CF37F40498C012C239CF603AD1B20FE38DB86EFE3E0B961F117B9FA30290,
                '15e298fe7b1d9199'
            );

            -- Store initial rootca certificate
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100108210FBA26C725AA2FCA027FC9ED8079DBDD7010203000028B19F7E86000A010180012380038100028080828FA35C637BEDDF1B711ED8EE5C742CEC434CD6D176289DB1A425A8E26586A0A78080D12C28F474AB8EE25C755F6A7A8C0B6B808B57ACE6725513ED9019EEAA70709F2543644C0262458F52213823929ABBBF4DE61B4E131ED7546C472FDCF346E45B,
                'd163e79063beef66'
            );

            -- Store initial ica certificate
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'intermediate_ca_cert',
                NOW(),
                0x80030080D163E79063BEEF6610821064B8CA758AF9D051E6AEC5327775A611010203000028B1A00986000A01018001238003830002808083BF5D281BA78D6827F0E5FE7B7855A6AC0D3D86FC840EA7A45EC24002E6B9C43D808030ABE803EFF631EBDDF1D71EBD7918E251A3B82669712F125F8BBF3D7AC31DB998C265D16F3A8C8320CD77015B85A6533F243EFDA8963EF444C10D2431894103,
                '3631eee885298ccc'
            );

            -- Store initial eca certificate
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'eca_certificate',
                NOW(),
                0x800300803631EEE885298CCC108210A1A716F77017678846039E7FB263675B010203000028B1A04C86000A01018001238003840002808083EDBFB3839538FB8625C9DCEE2F859150386DB09446D62AFEEBE52786E75B19ED8080C77E222553B2063223CC4CDC421EE8C6142BF6BC9B24C039EA8DD53695A136BC139862F47F97E1C9EF7A82C946A5C6245E747D6ED6094AB7DFC6E7563F2517E0,
                'b4e9c6cdab3ac006'
            );

            -- Store initial eca public key
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'eca_public_key',
                NOW(),
                0xEDBFB3839538FB8625C9DCEE2F859150386DB09446D62AFEEBE52786E75B19ED3AE7B5FB71D075E30299A4E2DBF0097ED02512181A8BDA58997660B1BF5405CB,
                ''
            );

            -- Store initial aca public key
            INSERT INTO ra_certificates (name, insert_date, file, cert_id)
            VALUES (
                'aca_certificate',
                NOW(),
                0x800300803631EEE885298CCC108210E5FB954C49C8C815046B9A871EE8CCD2010203000028B1A04C86000A01018001238003850002808083BB3414744554D998FF6ED1625163F0E3803EA5173BA6341850A2E53AFAA581DB8080AB652021AE92D6E9D98156B81600754A804286E7BE51ECC4DA9B2F0A52EF592FEC7F7A68CCC7ADD0FD4024F6E6849B0882F0B578300C8BD129BD45022EC22ACC,
                '8e5477d837c469d4'
            );
            
            "#,
        )
        .await?;

        Ok(())
    }
}
