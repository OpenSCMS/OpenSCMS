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

// m20240613_000001_create_successor_enrollment_certificate_table.rs

use sea_orm_migration::prelude::*;

pub struct Migration;

impl MigrationName for Migration {
    fn name(&self) -> &str {
        "m20240613_000001_create_successor_enrollment_certificate_table" // Make sure this matches with the file name
    }
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(SuccessorEnrollmentCertificateStore::Table)
                    .col(
                        ColumnDef::new(SuccessorEnrollmentCertificateStore::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SuccessorEnrollmentCertificateStore::HashId)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(SuccessorEnrollmentCertificateStore::RequestTimePeriod)
                            .big_unsigned()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SuccessorEnrollmentCertificateStore::RequestMessage)
                            .blob(BlobSize::Blob(None))
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SuccessorEnrollmentCertificateStore::EnrollmentStatus)
                            .enumeration(
                                Alias::new("request_status"),
                                [
                                    Alias::new("ToBeProcessed"),
                                    Alias::new("Processing"),
                                    Alias::new("Processed"),
                                    Alias::new("Downloaded"),
                                ],
                            )
                            .not_null(),
                    )
                    .col(ColumnDef::new(SuccessorEnrollmentCertificateStore::FileName).string())
                    .col(
                        ColumnDef::new(SuccessorEnrollmentCertificateStore::File)
                            .blob(BlobSize::Blob(None)),
                    )
                    .to_owned(),
            )
            .await
    }

    // Define how to rollback this migration: Drop the SuccessorEnrollmentCertificateStore table.
    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(SuccessorEnrollmentCertificateStore::Table)
                    .to_owned(),
            )
            .await
    }
}

#[derive(Iden)]
pub enum SuccessorEnrollmentCertificateStore {
    Table,
    Id,
    HashId,
    RequestTimePeriod,
    EnrollmentStatus,
    RequestMessage,
    FileName,
    File,
}
