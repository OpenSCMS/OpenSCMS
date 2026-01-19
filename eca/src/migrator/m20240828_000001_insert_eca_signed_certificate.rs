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

// m20240828_000001_insert_eca_signed_certificate.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240828_000001_insert_eca_signed_certificate" // Make sure this matches with the file name
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

            -- Step 1: Store eca private key
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_private_key',
                NOW(),
                0x7203B088A95EC483AA6F77383CD113E968D95B88971970150550A13644DE4F33
            );

            -- Step 1: Store eca public key
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_public_uncompressed',
                NOW(),
                0xF8CDEC8E9FC7F52A2F30D0484DED80626FAA02790C21AAC8527B88AE56ADCBEEE88119A22C020C5F9917482CBF70F182148EC3D1E19B73DA6A20A3CA61CE6FC9
            );

            -- Step 1: Store eca certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_certificate',
                NOW(),
                0x80030080BAC70087DC64F29000810F6563612E6C6773636D732E74657374010203000026F095C686000A808083F8CDEC8E9FC7F52A2F30D0484DED80626FAA02790C21AAC8527B88AE56ADCBEE80802BD5F9C0695FAFBF3E4F7610A196750C3A2E81B87E003D6279F9FB212787A5CE7EC18BF343C611CD956458AEEF8B6C8A8A6A00644C6DE5C16E1F5482C6E97B07
            );

            -- Step 1: Store root ca certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100008112726F6F7463612E6C6773636D732E74657374010203000026F0953986000A808083B1B194821CDCEEBB0666AEA13FC50771C8955FC3398F6BCD3E2494DA6D39C28C80808090A2310BE7BD045F122D1A2E77FD694B37DBAA675DE48C73C3475302CE8C94CA4C6D76D8AB64B40388E8AB3C600291997FE9F9415D11A5402D4AB3CD1E76BA
            );

            -- Step 1: Store ICA certificate
            INSERT INTO eca_certificates (name, insert_date, file)
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
