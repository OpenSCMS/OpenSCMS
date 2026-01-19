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
            DELETE FROM eca_certificates;

            -- Step 1: Store eca private key
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_private_key',
                NOW(),
                0xD023D49F145275110995F88DDA3A512FFE3BE05C5653AD99681B385B38E54D34
            );

            -- Step 1: Store eca public key
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_public_uncompressed',
                NOW(),
                0x651FB15720AE59AD415D691D2157437D38C0EAB97F90AE3F1CC1F241248DB0F25E68A8ED1BE1009B0574B93C9A417F9661537423DCC8229B65C9CC3155BD4A13
            );

            -- Step 1: Store eca certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'eca_certificate',
                NOW(),
                0x80030080A5E944214BE5B9E6008210B95492D216ED9D183B5510AB6D3F770B01020300002727388086000A808083651FB15720AE59AD415D691D2157437D38C0EAB97F90AE3F1CC1F241248DB0F28080D336537ED68FAEC71182343770F2003FBC7EBA6F2198CA3ADEF19B827802E2F641AB230473C1D9B7C02B605C872AF2D3E51EFD3DBDCB993EAB3090D1E6ECDC6B
            );

            -- Step 1: Store root ca certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'root_ca_cert',
                NOW(),
                0x8003008100008210E552A811E2329E936E8CB7212CE093490102030000272737CA86000A8080821BED0BC3C17823B3CEDC15CF3384D61F699AD4FB3D350AE7FB2683EA9A1AB8BA8080ADACF2B4EE7FC2333B8D54D2695F91A7AA162E3EC35FD1161F6083D008EB6CC234E3904685008DDDBB61DBD9F247CF8041591946BD33611BDB01544BE38DFC0C
            );

            -- Step 1: Store ICA certificate
            INSERT INTO eca_certificates (name, insert_date, file)
            VALUES (
                'intermediate_ca_cert',
                NOW(),
                0x80030080033F821E76587884008210D13590F825DA6A9FB09EA308A900EE2F01020300002727381986000A8080836FD6F9DC3F8F33FCE13C9B330176537BA4DCF93A8A10F94749D2BE151EB5CCF48080DFF4D0CABA9446D60958A6D93816E345202AD59B8AF8F2440D988DACE7C7357CEE98B66E234E0B1E5C3710E90F0189C6E12C76093664DCC01E039F3B71EC9B60
            );

            -- Step 1: Store debug canonical pair <canonical_id, canonical_public_key>
            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                '11D09FA5933BFE61EC4A26A06E024203A6699A5D4E39739CCDC5FCBB23759C66', 
                NOW(), 
                0x2CC5500F95F2124F0F032EABAFD91C150BC414BC1C303CF8416BADB09B6EEA5FF9FEB658B6ED6FB2F348C38CD9B1FED3325D3D81F46E3D164A5BA8EB252B4099
            );

            "#,
        )
        .await?;

        Ok(())
    }
}
