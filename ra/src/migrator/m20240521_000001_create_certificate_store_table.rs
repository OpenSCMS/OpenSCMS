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

// m20240521_000001_create_certificate_store_table.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240521_000001_create_certificate_store_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(CertificateStore::Table)
                    .col(
                        ColumnDef::new(CertificateStore::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(CertificateStore::HashId).string().not_null())
                    .col(
                        ColumnDef::new(CertificateStore::IndexI)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CertificateStore::IndexJ)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(CertificateStore::File)
                            .blob(BlobSize::Tiny)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the CertificateStore table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(CertificateStore::Table).to_owned())
            .await
    }
}

#[derive(Iden)]
pub enum CertificateStore {
    Table,
    Id,
    HashId,
    IndexI,
    IndexJ,
    File,
}
