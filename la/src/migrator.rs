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

mod m20240411_000001_create_table_plv_to_ls_map;
mod m20240508_000001_include_more_fields_plv_to_ls_map;
mod m20240509_000001_rename_i_value_to_i_index;
mod m20240604_000001_alter_request_id_to_64;
mod m20240605_000001_create_event_scheduler;
mod m20250915_000001_create_la_certificates_table;
pub mod setup_tables;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240411_000001_create_table_plv_to_ls_map::Migration),
            Box::new(m20240508_000001_include_more_fields_plv_to_ls_map::Migration),
            Box::new(m20240509_000001_rename_i_value_to_i_index::Migration),
            Box::new(m20240604_000001_alter_request_id_to_64::Migration),
            Box::new(m20240605_000001_create_event_scheduler::Migration),
            Box::new(m20250915_000001_create_la_certificates_table::Migration),
        ]
    }
}
