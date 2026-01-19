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

// m20241111_000001_update_electors_and_aca_certs.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20241111_000001_update_electors_and_aca_certs" // Make sure this matches with the file name
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
            DELETE FROM electors_store;

            -- Step 1: Store elector 1 certificate
            INSERT INTO electors_store (ctl_series_id, private_key, certificate) 
            VALUES (
                'FFFFFFFFFFFFFF00', 
                0x0EEC1A2464BBB84909E0530A9E2101028AE0E6BDE466417C9F062B3D31010101AC5FF67DC6A052030794D93E79D0D8AE, 
                0x8003008101008114656C6563746F72312E6C6773636D732E746573740102030000273EEA9886000A8082318375AFBA26E922B2E71568F1F3FDFBB1F0CA0250B41AE5FDB1E74C085E9C54364047C4F63124FC28698B1C2AC01D69E8B382618030AA0BDA5794B76C16C9CE3ABBED9B10F08F5C934A969C163193CF9F806C50BADADB2E5068B2DD02ACB970DC498410E44F05EBBA71D774BC9E81A9E8AC4CF82750D1C955A8665EA0FA9C650BD375FDE025399FCA424E57369E5E15F141CE84FB
            );

            -- Step 2: Store elector 2 certificate
            INSERT INTO electors_store (ctl_series_id, private_key, certificate) 
            VALUES (
                'FFFFFFFFFFFFFF00', 
                0x813A35C003C2599AF5FC0A40512856F325EB0695288AFACFACA309E699B1E11D23DA4C9906BE3DC48C5101E7A8C12A2D, 
                0x8003008101008114656C6563746F72322E6C6773636D732E746573740102030000273EEA9886000A8082318225AB1D321D195AA3A7B63EDA6AF504B4464BD10DAB39972CAB4BBC2D19AF82EE53ADC2D12EAAF2C2B8A9F6431F7DCF25826180756C5A0F9244427377C33BE30CF5E0CE919DC681760C34A973C7360D4D424F987454F1C6DB306F76FD97868AA667AF0F3E3EDD09EDB8ACC8D3E48588500E346BB4759B3ABFD96B27D484975894AB6E69B9A9DA11E5627981BF24C90A02DEE7B3
            );

            -- Step 3: Store elector 3 certificate
            INSERT INTO electors_store (ctl_series_id, private_key, certificate) 
            VALUES (
                'FFFFFFFFFFFFFF00', 
                0x10F89571465CE0CADEA64293432199A5498D0F1B2468AEE115E4AC32A86E362DD9B222BB7F4661D24B302AE9127E9DBD, 
                0x8003008101008114656C6563746F72332E6C6773636D732E746573740102030000273EEA9886000A808231823E1FE9ABB26528BE7C1F1963223703A880471F8908668F84720DB891EB0DF34FDBDC1814283D9EC3234F9672F24E4E9D8261806237F4AFDD3FFDE1FCAC4B263AD996B92D2A4B863518CF820D3D8DED516E0B644ACEC5C832031D3A7CEB7C5981F4595012C1815406860BD13AEEF5D7C27F02399A2A8104201F160AEDA096A951D9721387C3ABF217BB18C89C118DFA79CD85ED
            );
            
            DELETE FROM ra_certificates WHERE name = 'aca_certificate';

            INSERT INTO ra_certificates (name, insert_date, file, cert_id) 
            VALUES (
                'aca_certificate', 
                NOW(), 
                0x80030080A5E944214BE5B9E600810F6163612E6C6773636D732E746573740102030000273EEB7986000A8080829AF8F9040B3C5EDEA75D9D77049B53CCA62B79C324539DAEE124F7C1341EE2228080535EA9469835865A6F8C0CC791581DC2D8BE6DC93FF06EAFBEBBCA462102FE0DB96590B796A9CE3CC2EFAA3659B872DC8CDA4AE94823B7875BA430F0D9064681,
                'ae50e2c632cce941'
            );
            
            "#,
        )
        .await?;

        Ok(())
    }
}
