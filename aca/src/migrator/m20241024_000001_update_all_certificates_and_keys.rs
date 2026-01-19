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

// m20241024_000001_update_all_certificates_and_keys.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241024_000001_update_all_certificates_and_keys" // Make sure this matches with the file name
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
                0x70CA81F0DE7FA27D9D7D3F6CB5E232C9BE73FAEFCE4574483944449D19DC8B7A
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_public_uncompressed', 
                NOW(), 
                0x0D611145A49CBC6DFDCC0338B90DB76C484030146D47B3580E54E727604261C20DCE3E746C8A2A214BC534AC7BA7E9A1F6CA13C19CF760D3CCD70B4983BACFA3
            );

            -- Step 1: Store initial aca private key
            INSERT INTO aca_certificates (name, insert_date, file) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x80030080A5E944214BE5B9E6008210D8987EC66C7278BE67075A446C4D386E01020300002727390D86000A8080830D611145A49CBC6DFDCC0338B90DB76C484030146D47B3580E54E727604261C280805564C5324DF2D8EFEB38AD6F74AAEF0D4ECA817018BE5EF4E9656BB12EC11B03B0734B65500E4EF08569442F004EF6B441F38418800F41A6D482CA032E0D9015
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
