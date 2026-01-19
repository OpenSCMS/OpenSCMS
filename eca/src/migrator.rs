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

mod m20240619_000001_create_eca_certificates_table;
mod m20240619_000002_insert_eca_initial_certificates;
mod m20240625_000001_create_eca_request_management_table;
mod m20240626_000001_create_event_to_reset_request_table;
mod m20240723_000001_create_ee_canonical_pubkey_table;
mod m20240723_000002_insert_debug_canonical_entry;
mod m20240828_000001_insert_eca_signed_certificate;
mod m20241024_000001_update_all_certificates_and_keys;
mod m20241203_000001_insert_new_canonical_entry;
mod m20241227_000001_insert_more_canonical_entries;
mod m20250808_000001_drop_ee_canonical_table;
mod m20250818_000001_update_all_eca_certs_and_keys;
mod m20250919_000001_cleanup_all_eca_certs_and_keys;

pub mod setup_tables;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240619_000001_create_eca_certificates_table::Migration),
            Box::new(m20240619_000002_insert_eca_initial_certificates::Migration),
            Box::new(m20240625_000001_create_eca_request_management_table::Migration),
            Box::new(m20240626_000001_create_event_to_reset_request_table::Migration),
            Box::new(m20240723_000001_create_ee_canonical_pubkey_table::Migration),
            Box::new(m20240723_000002_insert_debug_canonical_entry::Migration),
            Box::new(m20240828_000001_insert_eca_signed_certificate::Migration),
            Box::new(m20241024_000001_update_all_certificates_and_keys::Migration),
            Box::new(m20241203_000001_insert_new_canonical_entry::Migration),
            Box::new(m20241227_000001_insert_more_canonical_entries::Migration),
            Box::new(m20250808_000001_drop_ee_canonical_table::Migration),
            Box::new(m20250818_000001_update_all_eca_certs_and_keys::Migration),
            Box::new(m20250919_000001_cleanup_all_eca_certs_and_keys::Migration),
        ]
    }
}
