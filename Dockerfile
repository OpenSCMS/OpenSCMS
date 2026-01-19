# syntax=docker/dockerfile:1.7-labs

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

FROM rust:1-bookworm

# Installing dependencies for bindgen
RUN apt-get update && \
    apt-get install -y llvm-dev && \
    apt-get install -y libclang-dev && \
    apt-get install -y clang

RUN wget -q https://github.com/Kitware/CMake/releases/download/v3.31.5/cmake-3.31.5-linux-x86_64.sh && \
    chmod +x ./cmake-3.31.5-linux-x86_64.sh && \
    sh ./cmake-3.31.5-linux-x86_64.sh --skip-license --prefix=/usr/local && \
    rm ./cmake-3.31.5-linux-x86_64.sh

RUN apt-get install -y iputils-ping

RUN useradd --create-home --home /home/app scmsuser

WORKDIR /home/app

USER scmsuser
COPY --chown=scmsuser ./certs /certs

ENV OSCMS_INSTALL_TARGET_DIR=/home/app/oscms_install
ENV LD_LIBRARY_PATH=$OSCMS_INSTALL_TARGET_DIR/lib:$LD_LIBRARY_PATH

COPY --chown=scmsuser --exclude=lgasn/lib1609 --exclude=./certs . .
RUN cargo fetch --locked
RUN cargo build --frozen

