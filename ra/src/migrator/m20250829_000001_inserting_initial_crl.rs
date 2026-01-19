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

// m20250829_000001_inserting_initial_crl.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20250829_000001_inserting_initial_crl" // Make sure this matches with the file name
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
            DELETE FROM crl_store;

            -- Store ra enc private key
            INSERT INTO crl_store (craca_id, crl_series, updated_time, file)
            VALUES (
                '15E298FE7B1D9199',
                1,
                NOW(),
                0x0381004003801D01000115E298FE7B1D919928BE710028BFC280400080000000000601004001010000000028BE7100810103800300803631EEE885298CCC118210E61F790557A4CD9CE8B57B97348FE216010203000028B1A04C86000A010180012380038B0002008082B79FF17F059BAA90A2BF8401A6D4AF824AFCF78203B9D01FB58FD4C5948A9F3C808083AC755B830636493D81575B4F15FF114F8848D778C923A4CF5D805AA6D03337E68080B6D1C11D8656D8643DB8CA7CBFA80688A6EE5BE12AD926B06020B421B54EBF78E4B1CF37F40498C012C239CF603AD1B20FE38DB86EFE3E0B961F117B9FA3029080030080D163E79063BEEF6610821064B8CA758AF9D051E6AEC5327775A611010203000028B1A00986000A01018001238003830002808083BF5D281BA78D6827F0E5FE7B7855A6AC0D3D86FC840EA7A45EC24002E6B9C43D808030ABE803EFF631EBDDF1D71EBD7918E251A3B82669712F125F8BBF3D7AC31DB998C265D16F3A8C8320CD77015B85A6533F243EFDA8963EF444C10D24318941038003008100108210FBA26C725AA2FCA027FC9ED8079DBDD7010203000028B19F7E86000A010180012380038100028080828FA35C637BEDDF1B711ED8EE5C742CEC434CD6D176289DB1A425A8E26586A0A78080D12C28F474AB8EE25C755F6A7A8C0B6B808B57ACE6725513ED9019EEAA70709F2543644C0262458F52213823929ABBBF4DE61B4E131ED7546C472FDCF346E45B8080CBB4A370E2B3229ADAAB1E66FB4D6E6675C744A039EDE69335B6992909784470CD3C2C10628BE642982B8CFFEB2C2851775F6DCC24F56690C2FB8E943AB347DA
            );            
            "#,
        )
        .await?;

        Ok(())
    }
}
