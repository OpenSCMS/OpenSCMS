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

// m20240723_000002_insert_debug_canonical_entry

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241203_000001_insert_new_canonical_entry" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                'B57CB7E74B6D3BC60D303F7D5AA25F1E44B9B40FD68CF9A28678456CF9B3DDA0', 
                NOW(), 
                0xA77C6E54428209A70473E092DEB87E1184AFBCEF5E2A5B96B67173F06C20E9E01408EAE5931F1640498ADD8773ECDDE2D92E1A6CA640F062083D95425A6B9977
            );

            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                '545B1B516DBD1362E7AC741688B84C71D17C875EA315988C772FE3273EE2E4F5', 
                NOW(), 
                0xA8632D9187B456C54752E50112E1D5DAF8495928D8405F9B5FE081139B813994A9F50BC26C6EA04E98DCB19AFD0B16242C29FCA699FF512E2712AAF2B09B9FE1
            );

            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                '5B237575776A521FB5A382F92FC51188FE59AB0C407455B332357C60B8994A2F', 
                NOW(), 
                0x55C7A492F336BA845C5CA8D83FCEDB776AC15B19CDC6576BED7CC794B8DB0846AACE400556978C83F75A4091C4ECA4879416937B0EF40EF74F1BEB309DEA3A6A
            );

            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                '8B4FC47F16828D98B05C5F637F806A14C3401190DF525B343FB2C4B4C9C4BC85', 
                NOW(), 
                0xF1F7E59ED024E2F576F813AFB4CDD02BA60F666030574AC10D85327EFD441B8CBFA4748405BA7816FB1EF1BE15CC5820828AF815DEAAE0DF67700CF0FD2245FD
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
