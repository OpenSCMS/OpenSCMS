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

use celery::error::TaskError;
use celery::task::TaskResult;
use scmscommon::{CaterpillarStatus, ExpansionType, NonButterflyRequest, errors};
use sea_orm::DatabaseConnection;

use crate::core_logic::gen_request_id;
use crate::persistence;
use crate::persistence::successor_enrollment_certificate_store::{
    fetch_request_message_from_enrollment_request_by_hash_id,
    set_enrollment_certificate_by_hash_id, set_enrollment_certificate_error_by_hash_id,
};
use crate::ra_endpoint::worker::{aca_requests, eca_requests, la_requests};

#[celery::task(max_retries = 1, name = "run_authorization_request_processing")]
pub async fn run_authorization_request_processing(
    exp_type: ExpansionType,
    non_butterfly_request: Option<NonButterflyRequest>,
) -> TaskResult<()> {
    log::info!(
        "Starting task: run_authorization_request_processing: {:?}",
        exp_type
    );

    if !matches!(
        exp_type,
        ExpansionType::NonButterfly | ExpansionType::NonButterflyEncrypted
    ) {
        log::error!("Invalid ExpansionType for this task: {:?}", exp_type);
        return run_caterpillar_processing(exp_type).await;
    }

    log::info!("Starting task: non butterfly: {:?}", exp_type);

    let db = scmscommon::connect_to_db().await.map_err(|e| {
        let message = format!("Failed to connect to database: {}", e);
        handle_db_error(message)
    })?;

    aca_requests::handle_aca_request(vec![], vec![], non_butterfly_request, exp_type, &db)
        .await
        .map_err(|e| {
            let message = format!("Failed to handle ACA requests: {}", e.message);
            handle_db_error(message)
        })?;

    log::debug!("Successfully handled ACA requests");

    db.close().await.map_err(|e| {
        let message = format!("Failed to close database connection: {}", e);
        TaskError::UnexpectedError(message)
    })?;

    log::info!("Task completed successfully");
    Ok(())
}

async fn run_caterpillar_processing(exp_type: ExpansionType) -> TaskResult<()> {
    log::info!("Starting task: run_caterpillar_processing: {:?}", exp_type);

    // Connecting to Database
    let db = match scmscommon::connect_to_db().await {
        Ok(db) => db,
        Err(e) => {
            let message = format!("Failed to connect to database: {}", e);
            return Err(handle_db_error(message));
        }
    };

    // Fetch caterpillars to be processed by the Task's ExpansionType if enough
    let (ids, caterpillars) = match persistence::caterpillar::fetch_caterpillars_if_enough(
        &db, exp_type,
    )
    .await
    {
        Ok((ids, caterpillars)) => {
            log::debug!(
                "Task got {:?} caterpillars to be processed.",
                caterpillars.len()
            );
            if caterpillars.is_empty() {
                log::info!(
                    "Not enough caterpillars to be processed for expansion type ({:?}). Finishing task.",
                    exp_type
                );

                // Closing DB connection
                match db.close().await {
                    Ok(_) => (),
                    Err(e) => {
                        let message = format!("Failed to close database connection: {}", e);
                        return Err(TaskError::UnexpectedError(message));
                    }
                }
                return Ok(());
            }
            (ids, caterpillars)
        }
        Err(e) => {
            let message = format!("Failed to fetch caterpillars: {}", e.message);
            return Err(handle_db_error(message));
        }
    };

    // Generating request id based on fetched caterpillars
    let request_id = gen_request_id(&caterpillars);
    log::debug!("Generated request id: {:?}", request_id);

    let payloads_la_to_ra =
        match la_requests::handle_multiple_la_requests(&caterpillars, request_id).await {
            Ok(payloads) => payloads,
            Err(e) => {
                let message = format!("Failed to handle LA requests: {}", e.message);
                return Err(handle_exec_error(message, db, ids).await);
            }
        };
    log::debug!(
        "Successfully handled LA requests: payloads found {}",
        payloads_la_to_ra.len()
    );

    match aca_requests::handle_aca_request(caterpillars, payloads_la_to_ra, None, exp_type, &db)
        .await
    {
        Ok(_) => (),
        Err(e) => {
            let message = format!("Failed to handle ACA requests: {}", e.message);
            return Err(handle_exec_error(message, db, ids).await);
        }
    }
    log::debug!("Successfully handled ACA requests");

    // Finishing processing
    match persistence::caterpillar::delete_by_ids(&db, ids.clone()).await {
        Ok(_) => (),
        Err(e) => {
            let message = format!(
                "Failed to delete caterpillar ({}). Inconsistent status for caterpillars [{:?}]",
                e.message, ids
            );
            return Err(handle_db_error(message));
        }
    }

    // Closing DB connection
    match db.close().await {
        Ok(_) => (),
        Err(e) => {
            let message = format!("Failed to close database connection: {}", e);
            return Err(TaskError::UnexpectedError(message));
        }
    }

    log::info!("Task completed successfully");
    Ok(())
}

#[celery::task(max_retries = 1, name = "run_successor_enrollment_request")]
pub async fn run_successor_enrollment_request(hash_id: String) -> TaskResult<()> {
    log::info!("Starting task: run_successor_enrollment_request");

    // Connecting to Database
    let db = match scmscommon::connect_to_db().await {
        Ok(db) => db,
        Err(e) => {
            let message = format!("Failed to connect to database: {}", e);
            return Err(handle_db_error(message));
        }
    };

    // 1. Fetch request message to be processed
    log::debug!("Fetching request message to be processed: {}", hash_id);
    let request_message = match fetch_request_message_from_enrollment_request_by_hash_id(
        &db,
        hash_id.clone(),
    )
    .await
    {
        Ok(request_message) => request_message,
        Err(e) => {
            let message = format!("Failed to fetch request message: {}", e.message);
            return Err(handle_db_error(message));
        }
    };

    // 2. Send request to ECA and capture response
    log::debug!("Sending request to ECA and capturing response");
    let (filename, content) = match eca_requests::handle_eca_enrollment_certificate_request(
        request_message,
        hash_id.clone(),
    )
    .await
    {
        Ok((filename, content)) => (filename, content),
        Err(e) => {
            let message = format!("Failed to handle ECA requests: {}", e.message);
            let status_code = e.error_code as i32;
            let detailed_code = e.detailed_code.unwrap_or(0);
            set_enrollment_certificate_error_by_hash_id(
                &db,
                hash_id.clone(),
                status_code,
                detailed_code,
            )
            .await
            .unwrap_or_else(|e| {
                log::error!(
                    "Failed to store enrollment certificate error code: {}",
                    e.message
                )
            });
            return Err(TaskError::UnexpectedError(message));
        }
    };

    // 3. Verify hash_id against response filename
    log::debug!(
        "Got file from ECA size: {}: Verifying hash_id {} against response filename {} ",
        content.len(),
        hash_id,
        filename
    );
    let filename_to_compare = format!("{}_enroll.oer", hash_id);
    if filename != filename_to_compare {
        let message = format!(
            "Filename mismatch: expected {} but got {}",
            filename_to_compare, filename
        );
        return Err(TaskError::UnexpectedError(message));
    }

    if content.is_empty() {
        let message = "Empty content received from ECA".to_string();
        return Err(TaskError::UnexpectedError(message));
    }

    // 4. Store enrollment certificate and filename
    log::debug!(
        "Storing enrollment certificate and filename: content size {:?}",
        content.len()
    );
    match set_enrollment_certificate_by_hash_id(&db, hash_id, filename, content).await {
        Ok(_) => (),
        Err(e) => {
            let message = format!("Failed to store enrollment certificate: {}", e.message);
            return Err(handle_db_error(message));
        }
    }

    // 5. Done
    // Closing DB connection
    match db.close().await {
        Ok(_) => (),
        Err(e) => {
            let message = format!("Failed to close database connection: {}", e);
            return Err(TaskError::UnexpectedError(message));
        }
    }

    log::info!("Task run_successor_enrollment_request completed successfully");
    Ok(())
}

pub async fn eca_public_key_download_and_store(
    db: &DatabaseConnection,
) -> Result<[u8; 64], errors::ScmsInternalCommError> {
    log::info!("Running eca_public_key_download_and_store");
    // 1. Download ECA public key
    let eca_public_key = eca_requests::download_eca_public_key().await?;
    log::debug!("Downloaded ECA public key: size {}", eca_public_key.len());

    // 2. Store eca public key at ra certificate table
    persistence::ra_certificates::store_new_eca_public_key(eca_public_key.clone(), db)
        .await
        .map_err(|e| {
            let message = format!("Failed to store ECA public key: {}", e.message);
            errors::ScmsInternalCommError::new(&message, errors::InternalCommWire::EcaToRa, 500)
        })?;

    log::info!("Periodic Task run_periodic_eca_public_key_download completed successfully");
    let mut array = [0; 64];
    array.copy_from_slice(&eca_public_key);
    Ok(array)
}

#[celery::task(max_retries = 1, name = "run_periodic_eca_public_key_download")]
pub async fn run_periodic_eca_public_key_download() -> TaskResult<()> {
    log::info!("Starting periodic task: run_periodic_eca_public_key_download");
    // Connecting to Database
    let db = match scmscommon::connect_to_db().await {
        Ok(db) => db,
        Err(e) => {
            let message = format!("Failed to connect to database: {}", e);
            return Err(handle_db_error(message));
        }
    };

    // 1. Download ECA public key and store
    match eca_public_key_download_and_store(&db).await {
        Ok(_) => (),
        Err(e) => {
            let message = format!("Failed to download and store ECA public key: {}", e.message);
            return Err(TaskError::UnexpectedError(message));
        }
    }

    log::info!("Periodic Task run_periodic_eca_public_key_download completed successfully");
    Ok(())
}

fn handle_db_error(message: String) -> TaskError {
    log::error!("{}", message);
    TaskError::UnexpectedError(message)
}

async fn handle_exec_error(
    err_message: String,
    db: DatabaseConnection,
    ids: Vec<i32>,
) -> TaskError {
    // Rollback caterpillar status
    match persistence::caterpillar::update_status_by_ids(
        &db,
        ids.clone(),
        CaterpillarStatus::ToBeProcessed,
    )
    .await
    {
        Ok(_) => (),
        Err(e) => {
            let message = format!(
                "Failed to rollback caterpillar status to ToBeProcessed ({}). Inconsistent status for caterpillars [{:?}]",
                e.message, ids
            );
            return handle_db_error(message);
        }
    }
    TaskError::UnexpectedError(err_message)
}
