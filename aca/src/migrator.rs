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

mod m20240717_000001_create_table_lv_to_plv_map;
mod m20240918_000001_create_aca_certificates_table;
mod m20240918_000002_insert_aca_initial_certificates;
mod m20241024_000001_update_all_certificates_and_keys;
mod m20250221_000001_update_all_certificates_and_keys;
mod m20250820_000001_update_all_certificates_and_keys;
mod m20250919_000001_cleanup_all_certificates_and_keys;

pub mod setup_tables;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240717_000001_create_table_lv_to_plv_map::Migration),
            Box::new(m20240918_000001_create_aca_certificates_table::Migration),
            Box::new(m20240918_000002_insert_aca_initial_certificates::Migration),
            Box::new(m20241024_000001_update_all_certificates_and_keys::Migration),
            Box::new(m20250221_000001_update_all_certificates_and_keys::Migration),
            Box::new(m20250820_000001_update_all_certificates_and_keys::Migration),
            Box::new(m20250919_000001_cleanup_all_certificates_and_keys::Migration),
        ]
    }
}
