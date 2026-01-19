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

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240626_000001_create_event_to_reset_request_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        // Use `execute_unprepared` if the SQL statement doesn't have value bindings
        db.execute_unprepared(
            r#"
            -- Drop the event if it exists
            DROP EVENT IF EXISTS ResetRequestTableWeekly;

            -- Create event to delete expired entries every hour
            CREATE EVENT ResetRequestTableWeekly
            ON SCHEDULE EVERY 1 DAY
            DO
            BEGIN
                DECLARE initial_epoch_2004_01_01_00_00_00 INT DEFAULT 1072915200;
                DECLARE epoch_now INT;
                DECLARE current_time_seconds INT;
                DECLARE seven_days_in_seconds INT DEFAULT 7 * 24 * 60 * 60; -- 7 days in seconds

                -- Get the current epoch time in seconds
                SET epoch_now = UNIX_TIMESTAMP();

                -- Calculate current time in seconds since 2004-01-01
                SET current_time_seconds = epoch_now - initial_epoch_2004_01_01_00_00_00;

                -- Delete expired entries
                DELETE FROM eca_request_management 
                WHERE (current_time_seconds - period_started_at) > seven_days_in_seconds;
            END;
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                DROP EVENT IF EXISTS ResetRequestTableWeekly;
                "#,
            )
            .await?;
        Ok(())
    }
}
