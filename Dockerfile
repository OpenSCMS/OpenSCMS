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

FROM rust:1-trixie

# Install all basic tools - not build dependencies. We do them last to maximize layer caching
RUN DEBIAN_FRONTEND=noninteractive apt-get update -qy && \
    DEBIAN_FRONTEND=noninteractive apt-get -qy install \
    clang \
    cmake \
    iputils-ping

# Install anything needed to compile successfully
RUN DEBIAN_FRONTEND=noninteractive apt-get -qy install \
    libclang-dev \
    llvm-dev \
    libzip-dev \
    libssl-dev

# Cleanup the apt cache. If you need to install anything else, you will need to re-run apt update
RUN rm -rf /var/lib/apt/lists

RUN useradd --create-home --home /home/app scmsuser

WORKDIR /home/app

USER scmsuser
COPY --chown=scmsuser ./certs /certs
COPY --chown=scmsuser --exclude=./certs . .

ENV OSCMS_INSTALL_TARGET_DIR=/home/app/oscms_install
ENV LD_LIBRARY_PATH=$OSCMS_INSTALL_TARGET_DIR/lib:$LD_LIBRARY_PATH

RUN cargo fetch --locked
RUN cargo build --frozen

