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

use crate::migrator::Migrator;
use sea_orm::{ConnectionTrait, DatabaseConnection};
use sea_orm_migration::prelude::{DbErr, MigratorTrait, SchemaManager};

pub async fn setup_tables(db: &DatabaseConnection) -> Result<(), DbErr> {
    let schema_manager = SchemaManager::new(db); // To investigate the schema

    Migrator::up(db, None).await?;
    assert!(schema_manager.has_table("caterpillar").await?);
    assert!(schema_manager.has_table("certificate_store").await?);
    assert!(schema_manager.has_table("x_dot_info_store").await?);
    assert!(schema_manager.has_table("ra_certificates").await?);
    assert!(
        schema_manager
            .has_table("successor_enrollment_certificate_store")
            .await?
    );

    setup_stored_procedures(db).await?;
    Ok(())
}

async fn setup_stored_procedures(db: &DatabaseConnection) -> Result<(), DbErr> {
    let create_procedure_stms = vec![
        "DROP PROCEDURE IF EXISTS SelectCaterpillarsIfEnough ",
        r#"
        CREATE PROCEDURE SelectCaterpillarsIfEnough(IN threshold INT, IN exp_type_param VARCHAR(255))
        BEGIN
            DECLARE count INT;

            START TRANSACTION;

            SELECT COUNT(*) INTO count FROM caterpillar WHERE exp_type = exp_type_param AND caterpillar_status = 'ToBeProcessed' FOR UPDATE;

            -- Check if the count is greater than or equal to the threshold
            IF count >= threshold THEN
                -- Get the count of rows that meet the criteria
                DROP TEMPORARY TABLE IF EXISTS caterpillar_temp;
                CREATE TEMPORARY TABLE caterpillar_temp AS
                (
                    SELECT id FROM caterpillar
                    WHERE exp_type = exp_type_param AND caterpillar_status = 'ToBeProcessed'
                );

                -- Update the rows that meet the criteria
                UPDATE caterpillar c1
                JOIN caterpillar_temp c2
                ON c1.id = c2.id
                SET c1.caterpillar_status = 'Processing';

                -- Select the updated rows
                SELECT * FROM caterpillar c1
                JOIN caterpillar_temp c2
                ON c1.id = c2.id;

                COMMIT;
            ELSE
                -- Rollback if the count is less than the threshold
                ROLLBACK;
            END IF;
        END
    "#,
    ];

    // Execute the creation of the procedure
    for stmt in create_procedure_stms {
        match db.execute_unprepared(stmt).await {
            Ok(_) => {}
            Err(e) => {
                log::error!("Error creating stored procedure: {}", stmt);
                return Err(e);
            }
        }
    }

    Ok(())
}
