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

mod m20240313_000001_create_caterpillar_table;
mod m20240402_000001_change_string_to_enum;
mod m20240403_000001_populate_caterpillar_status_enum;
mod m20240403_000002_replace_new_caterpillar_status;
mod m20240403_000003_make_exp_type_be_enum;
mod m20240424_000004_add_new_caterpillar_status_option;
mod m20240521_000001_create_certificate_store_table;
mod m20240521_000002_create_x_dot_info_store_table;
mod m20240531_000001_alter_tinyblob_to_blob;
mod m20240531_000002_create_ra_certificates_table;
mod m20240603_000001_insert_ra_initial_certificates;
mod m20240605_000001_add_new_column_to_store_tables;
mod m20240613_000001_create_successor_enrollment_certificate_table;
mod m20240626_000001_create_event_scheduler;
mod m20240701_000001_add_error_coluns_to_se_cert_table;
mod m20240702_000001_create_ra_request_management_table;
mod m20240702_000002_create_event_to_reset_request_table;
mod m20240711_000001_create_ee_registration_table;
mod m20240823_000001_create_electors_table;
mod m20240823_000002_create_ctl_file_table;
mod m20240823_000003_add_initial_root_cas_certs;
mod m20240823_000004_add_initial_electors_certs;
mod m20240826_000001_create_ccf_file_table;
mod m20240828_000001_insert_signed_ra_certificate;
mod m20240904_000001_update_electors_certs;
mod m20240904_000002_update_all_certificates;
mod m20240924_000001_insert_ra_enc_keys;
mod m20241024_000001_update_all_certificates_and_keys;
mod m20241105_000001_alter_table_add_cert_id;
mod m20241105_000002_add_cert_id;
mod m20241111_000001_update_electors_and_aca_certs;
mod m20241121_000001_insert_ctl_and_ccf_files;
mod m20241210_000001_add_canonical_id_column_to_ee_registration;
mod m20241219_000001_add_updated_time_column_to_files_tables;
mod m20250108_000001_create_composite_crl_store_table;
mod m20250108_000002_create_crl_store_table;
mod m20250224_000001_update_aca_cert_and_clean_ccf_store;
mod m20250808_000001_change_ee_registration_table;
mod m20250814_000001_update_ca_certs_hash_id;
mod m20250818_000001_update_all_ca_certs_and_keys;
mod m20250829_000001_inserting_initial_crl;
mod m20250919_000001_cleanup_all_ca_certs_and_keys;
mod m20251125_000001_drop_electors_store;

pub mod setup_tables;
pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240313_000001_create_caterpillar_table::Migration),
            Box::new(m20240402_000001_change_string_to_enum::Migration),
            Box::new(m20240403_000001_populate_caterpillar_status_enum::Migration),
            Box::new(m20240403_000002_replace_new_caterpillar_status::Migration),
            Box::new(m20240403_000003_make_exp_type_be_enum::Migration),
            Box::new(m20240424_000004_add_new_caterpillar_status_option::Migration),
            Box::new(m20240521_000001_create_certificate_store_table::Migration),
            Box::new(m20240521_000002_create_x_dot_info_store_table::Migration),
            Box::new(m20240531_000001_alter_tinyblob_to_blob::Migration),
            Box::new(m20240531_000002_create_ra_certificates_table::Migration),
            Box::new(m20240603_000001_insert_ra_initial_certificates::Migration),
            Box::new(m20240605_000001_add_new_column_to_store_tables::Migration),
            Box::new(m20240613_000001_create_successor_enrollment_certificate_table::Migration),
            Box::new(m20240626_000001_create_event_scheduler::Migration),
            Box::new(m20240701_000001_add_error_coluns_to_se_cert_table::Migration),
            Box::new(m20240702_000001_create_ra_request_management_table::Migration),
            Box::new(m20240702_000002_create_event_to_reset_request_table::Migration),
            Box::new(m20240711_000001_create_ee_registration_table::Migration),
            Box::new(m20240823_000001_create_electors_table::Migration),
            Box::new(m20240823_000002_create_ctl_file_table::Migration),
            Box::new(m20240823_000003_add_initial_root_cas_certs::Migration),
            Box::new(m20240823_000004_add_initial_electors_certs::Migration),
            Box::new(m20240826_000001_create_ccf_file_table::Migration),
            Box::new(m20240828_000001_insert_signed_ra_certificate::Migration),
            Box::new(m20240904_000001_update_electors_certs::Migration),
            Box::new(m20240904_000002_update_all_certificates::Migration),
            Box::new(m20240924_000001_insert_ra_enc_keys::Migration),
            Box::new(m20241024_000001_update_all_certificates_and_keys::Migration),
            Box::new(m20241105_000001_alter_table_add_cert_id::Migration),
            Box::new(m20241105_000002_add_cert_id::Migration),
            Box::new(m20241111_000001_update_electors_and_aca_certs::Migration),
            Box::new(m20241121_000001_insert_ctl_and_ccf_files::Migration),
            Box::new(m20241210_000001_add_canonical_id_column_to_ee_registration::Migration),
            Box::new(m20241219_000001_add_updated_time_column_to_files_tables::Migration),
            Box::new(m20250108_000001_create_composite_crl_store_table::Migration),
            Box::new(m20250108_000002_create_crl_store_table::Migration),
            Box::new(m20250224_000001_update_aca_cert_and_clean_ccf_store::Migration),
            Box::new(m20250808_000001_change_ee_registration_table::Migration),
            Box::new(m20250814_000001_update_ca_certs_hash_id::Migration),
            Box::new(m20250818_000001_update_all_ca_certs_and_keys::Migration),
            Box::new(m20250829_000001_inserting_initial_crl::Migration),
            Box::new(m20250919_000001_cleanup_all_ca_certs_and_keys::Migration),
            Box::new(m20251125_000001_drop_electors_store::Migration),
        ]
    }
}
