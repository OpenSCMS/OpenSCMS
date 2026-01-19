#! /bin/bash

# Copyright (c) 2025 LG Electronics, Inc.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
# http:#www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.
#
# SPDX-License-Identifier: Apache-2.0

LGSCMS_PATH="$PWD"

## Exporting Env Vars
export CERTIFICATES_PER_BATCH=10
export PERIOD_LENGTH_DAYS=1
export DEPLOY_ENVIRONMENT="unit-tests"
export LA_COUNT=2
export LA_PORT=5000
export LA_PREFIX="la"

export CTL_SERIES_ID="FFFFFFFFFFFFFF00"

export ACA_CERTS_FOLDER="${LGSCMS_PATH}/certs/aca"
export ACA_CERTIFICATE_FILE="acaCertificate.coer"
export ACA_PRIVATE_KEY_FILE="acaPrivateKey.dat"
export ACA_PUBLIC_KEY_FILE="acaPublicKey.dat"

export RA_CERTS_FOLDER="${LGSCMS_PATH}/certs/ra"
export RA_CERTIFICATE_FILE="raCertificate.coer"
export RA_PRIVATE_KEY_FILE="raPrivateKey.dat"
export RA_PUBLIC_KEY_FILE="raPublicKey.dat"
export RA_ENC_PUBLIC_KEY_FILE="raEncPublicKey.dat"
export RA_ENC_PRIVATE_KEY_FILE="raEncPrivateKey.dat"

export ECA_CERTS_FOLDER="${LGSCMS_PATH}/certs/eca"
export ECA_CERTIFICATE_FILE="ecaCertificate.coer"
export ECA_PRIVATE_KEY_FILE="ecaPrivateKey.dat"
export ECA_PUBLIC_KEY_FILE="ecaPublicKey.dat"

export ICA_CERTS_FOLDER="${LGSCMS_PATH}/certs/ica"
export ICA_CERTIFICATE_FILE="icaCertificate.coer"

export ROOTCA_CERTS_FOLDER="${LGSCMS_PATH}/certs/rootca"
export ROOTCA_CERTIFICATE_FILE="rootcaCertificate.coer"

export CTL_FOLDER="${LGSCMS_PATH}/certs/ctl"
export CTL_FILE="ctl.coer"
export CTL_CCF_FILE="ccf.coer"

export ROOT_PROTOCOL_FILES_PATH="${LGSCMS_PATH}/protocol_files"

export PARAM_WEBAPI_NAME="scmsV3"
export PARAM_WEBAPI_EE_AUTH="None"

export PARAM_SCMSV3_EE_AUTH="None"
export PARAM_SCMSV3_ERROR="fine"
export PARAM_SCMSV3_OPTIONS=""

export PARAM_ECA_MAX_AGE="0"
export PARAM_ECA_MAX_REQS="10"
export PARAM_ECA_MAX_WAIT="50"
export PARAM_ECA_MIN_WAIT="30"

export PARAM_RA_ACPC_SUPPORT="true"
export PARAM_RA_BUTTERFLY_TYPE="compact unified"
export PARAM_RA_MAX_AGE="0"
export PARAM_RA_MAX_GEN_DELAY="0"
export PARAM_RA_MAX_REQS="5"
export PARAM_RA_MAX_RELOAD_TIME="0"
export PARAM_RA_MIN_WAIT="30"

export PARAM_DOWNLOAD_MAX_AGE="1000"
export PARAM_DOWNLOAD_MIN_WAIT="30"

export MIN_NUMBER_CERTS_RA="1"
export LA_ID="1"

export CAM_PORT=5000
export CAM_PREFIX="cam"

export ACA_PORT=5000
export ACA_PREFIX="aca"

export PARAM_SUCCESSOR_NEXT_DL_TIME=5
export PARAM_CERT_NEXT_DL_TIME=5

## RA CONFIG
export MINIMUM_NUMBER_OF_REQUESTS="1"
export NUMBER_OF_BATCHES="1"
export RABBITMQ_PASSWORD="password"
export RABBITMQ_USERNAME="rmqadmin"
export RABBITMQ_CONNECTION_STRING="amqp://rmqadmin:password@scmsrabbitmq:5672"
export RA_ID="1"
export RA_PATH="RA"

export NUMBER_CERT_BATCHES="1"
export TIME_I_PERIOD_EPOCH="0"
export TIME_I_PERIOD_INIT="0"

# Cleaning and tidying the cargo environment
# to get the most up-to-date version of the c libraries for creating bindings
cargo clean
cargo build

## Running tests
RUST_BACKTRACE=full cargo test --workspace --no-fail-fast ${1}

