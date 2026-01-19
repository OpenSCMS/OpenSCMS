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

// m20240823_000003_add_initial_root_cas_certs.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240823_000003_add_initial_root_cas_certs" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Step 1: Store initial ra certificate
            INSERT INTO ra_certificates (name, insert_date, file) 
            VALUES (
                'root_ca_cert', 
                NOW(), 
                0x80030081000082101B5898FD464B8366B39E88C9335EC54D010203000026D2733A840018808084E83BAE6A83C4F860FCFCDCF5AE6A0BF40952DA4C89A8B2DC7005130606AB10B362C2644986EE7A7F6B53A820241E37AB0D9C09E87CCE4B17CD5B597A8312000B8080EF5E5ED57F8EB435895404BA45C444DC23895D879477B8DB324091F2ED34E36DE4C15ED0F4D7316C02A8366DF3C768E0BD03238E3CC873CBF2ABE9E570F28DED
            );

            -- Step 1: Store initial ra private key
            INSERT INTO ra_certificates (name, insert_date, file) 
            VALUES (
                'intermediate_ca_cert', 
                NOW(), 
                0x80030080310BF2FDE7FE81C1008210D7BC90B9579E98ED683483E33BF6FA24010203000026D273D084001880808403A656E13013CEE082465BD5529783339629FC918E3F56F2A2356455848F40E3F810C5B12A7E8B9063DC3738C2FE9D6767765532B16EAD10646E2DF1E34F6E7C8080E5D263EE901B8179C984C91CB49C55295959DF9A62962F3F4B91002B7E104E96DDB9FFE30C0FB3196BC023298A6B70743FACA73D1B1190A25B0B78A477164C8F
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
