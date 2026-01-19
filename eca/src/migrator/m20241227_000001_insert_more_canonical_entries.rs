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

// m20241227_000001_insert_more_canonical_entries

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241227_000001_insert_more_canonical_entries" // Make sure this matches with the file name
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
                '6CA79922E9EA37DBE2A72B19CB5403E949C80F8B3A925F1A4826F7A094F213D9', 
                NOW(), 
                0xDA25A3D04026482B2C1F5BF6C6663A00E65E2793544C2DA912A4A9461D70FA94B91ED96AF33676C52513DAB9630C2B813464F4D410196A469EBD236A3C26CB42
            );

            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                '138760339A6D8B46DC2A41B45987315A760B72CB7E31BDD4BC1FA0ACA77BCB04', 
                NOW(), 
                0x6A35B108F99BA08B0BA66930C68BFE4C4F1CF8320852DE7491390E2F7F57CE010EBD7002FB9E2941B0EDE0D5C834D280019FECA25C696BE1B28C28775C9936A5
            );

            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                'FFBBF9B2595CEFFAFCB7D40D249E83DE89786B5789144C329C0C9E18FB662E98', 
                NOW(), 
                0x700328184992152D9DF39DAF1F6F56B2F5553982621E495AFD3B138986663463447D3313B1D8316E0D67A6154EAF6F150001280F35688A2E129C79646A2E8FB9
            );

            INSERT INTO ee_canonical_public_key (canonical_id, insert_date, public_key) 
            VALUES (
                'CA169CB0CE92876D82617A4E7E3D519B64E32B5305F62340C46E5B7266889E3F', 
                NOW(), 
                0xE642A1D966E29ED1A6678163BE48CCAE0AFFB97A79B9333994775B2650AF0D69A99F68471260B96FBE1B4861A8AB157374ABF7694089AF470CE1603D753EF8D6
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
