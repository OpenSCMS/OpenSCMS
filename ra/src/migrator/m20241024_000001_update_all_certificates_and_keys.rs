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
            DELETE FROM ra_certificates;

            -- Store ra enc private key
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_enc_private_key',
                NOW(),
                0xEBCEC04ED3AC08F0DF221C00DC5C751B910E2C6B5ED7D18796FE33041147E859
            );

            -- Store ra enc public key uncompressed
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_enc_public_uncompressed',
                NOW(),
                0xD3D6D41F3B4E81209E2C068E5DD05C07A97CC1055EB947320DC99FF7F267DD199861F9BBA945098E56139806A8864059A43069FB141761674BE12CB81B72F2E7
            );

            -- Store ra private key
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_private_key',
                NOW(),
                0x7A5356DB6B95500ADAB4A2FFCC756B94213C4634921FE65FFC4C07DB5789F68A
            );

            -- Store ra public key uncompressed
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_public_uncompressed',
                NOW(),
                0xC9179C9F36D7F0FFF425224155DEE34E276DD36DAC1537A856912E7925C04A58396FAA0519830FDA15DDE0BE7A7A91685ED617B3F68E31ABA2F10F35AC05D56A
            );

            -- Store initial ra certificate
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'ra_certificate',
                NOW(),
                0x80030080A5E944214BE5B9E6018210B09548E7BC6858964C8CA1D30E8D56CF01020300002727388086000A008083D3D6D41F3B4E81209E2C068E5DD05C07A97CC1055EB947320DC99FF7F267DD19808082C9179C9F36D7F0FFF425224155DEE34E276DD36DAC1537A856912E7925C04A588080E68592DB43EFC1D1938420DA215B82F0607009746E043E41E81FA0E0AE83E4BFF25A59566CA04D28A71A61191F36A1CA3EB51951EF8B0721F88F5AECA76CD0F5
            );

            -- Step 1: Store RootCa Certificate
            INSERT INTO ra_certificates (name, insert_date, file) 
            VALUES (
                'root_ca_cert', 
                NOW(), 
                0x8003008100008210E552A811E2329E936E8CB7212CE093490102030000272737CA86000A8080821BED0BC3C17823B3CEDC15CF3384D61F699AD4FB3D350AE7FB2683EA9A1AB8BA8080ADACF2B4EE7FC2333B8D54D2695F91A7AA162E3EC35FD1161F6083D008EB6CC234E3904685008DDDBB61DBD9F247CF8041591946BD33611BDB01544BE38DFC0C
            );

            -- Step 1: Store ICA certificate
            INSERT INTO ra_certificates (name, insert_date, file) 
            VALUES (
                'intermediate_ca_cert', 
                NOW(), 
                0x80030080033F821E76587884008210D13590F825DA6A9FB09EA308A900EE2F01020300002727381986000A8080836FD6F9DC3F8F33FCE13C9B330176537BA4DCF93A8A10F94749D2BE151EB5CCF48080DFF4D0CABA9446D60958A6D93816E345202AD59B8AF8F2440D988DACE7C7357CEE98B66E234E0B1E5C3710E90F0189C6E12C76093664DCC01E039F3B71EC9B60
            );

            -- Step 1: Store ECA certificate
            INSERT INTO ra_certificates (name, insert_date, file)
            VALUES (
                'eca_certificate',
                NOW(),
                0x80030080A5E944214BE5B9E6008210B95492D216ED9D183B5510AB6D3F770B01020300002727388086000A808083651FB15720AE59AD415D691D2157437D38C0EAB97F90AE3F1CC1F241248DB0F28080D336537ED68FAEC71182343770F2003FBC7EBA6F2198CA3ADEF19B827802E2F641AB230473C1D9B7C02B605C872AF2D3E51EFD3DBDCB993EAB3090D1E6ECDC6B
            );            
            "#,
        )
        .await?;

        Ok(())
    }
}
