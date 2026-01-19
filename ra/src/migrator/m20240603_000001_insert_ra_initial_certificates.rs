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

// m20240603_000001_insert_ra_initial_certificates.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240603_000001_insert_ra_initial_certificates" // Make sure this matches with the file name
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
                'ra_certificate', 
                NOW(), 
                0x80030080456C0777DC0EBF290082086ECB90FC1D75489F01020300002394726E84FFFF8080827C802567960204CA5E7DAE217925AE569A43CE213427EB5058F0C2AF361E13668080BF2004DC879ACA952BB7CB1D0E5E20110D577FE282F7B77FC06BFDF3F58CFF7A94CDD8392ECCC00D7C065FD1682347AEE0AD0B58828B6C433170371D16B741F0
            );

            -- Step 1: Store initial ra private key
            INSERT INTO ra_certificates (name, insert_date, file) 
            VALUES (
                'ra_private_key', 
                NOW(), 
                0xD6F775B173EB9941E72B3A0250F69C6DB6A4F3BB6465CC50A2391E9635765993
            );
            "#,
        )
        .await?;

        Ok(())
    }
}
