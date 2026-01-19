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

// m20250221_000001_update_all_certificates_and_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250221_000001_update_all_certificates_and_keys" // Make sure this matches with the file name
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
                0x364A6C6CC0232C549E3258AAB28367F5FFDEA5211C473CCAEA4E6719065EE255
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_public_uncompressed', 
                NOW(), 
                0x32681AEC7978A221ACCEA44504C3B1A4684E257331CF88DC9A852B5A0E5B4C8FF94D77ADC6A0DB88EDFA5160BDB160582B680C55C8C9DB33ED204FEE820CD8F5
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x80030080A5E944214BE5B9E60082105073AE84F5F6B4839512FFAB1DE91E28010203000027C920BA86000A80808332681AEC7978A221ACCEA44504C3B1A4684E257331CF88DC9A852B5A0E5B4C8F8080B5367BA4DD3BCA06471DE977D73BC62C380B05CBB991D167FAA169303DEF3B42F4BDAD37D85DD5DDFD84D6C01FB9EB971A1A2D8C65D5FB2D54FA4CCEE3087143
            );

            -- Step 1: Store root ca certificate
            INSERT INTO aca_certificates (name, insert_date, file)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100008210E552A811E2329E936E8CB7212CE093490102030000272737CA86000A8080821BED0BC3C17823B3CEDC15CF3384D61F699AD4FB3D350AE7FB2683EA9A1AB8BA8080ADACF2B4EE7FC2333B8D54D2695F91A7AA162E3EC35FD1161F6083D008EB6CC234E3904685008DDDBB61DBD9F247CF8041591946BD33611BDB01544BE38DFC0C
            );

            -- Step 1: Store ICA certificate
            INSERT INTO aca_certificates (name, insert_date, file)
            VALUES (
                'intermediate_ca_cert',
                NOW(),
                0x80030080033F821E76587884008210D13590F825DA6A9FB09EA308A900EE2F01020300002727381986000A8080836FD6F9DC3F8F33FCE13C9B330176537BA4DCF93A8A10F94749D2BE151EB5CCF48080DFF4D0CABA9446D60958A6D93816E345202AD59B8AF8F2440D988DACE7C7357CEE98B66E234E0B1E5C3710E90F0189C6E12C76093664DCC01E039F3B71EC9B60
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
