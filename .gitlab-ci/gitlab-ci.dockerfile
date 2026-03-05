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

# This image is used for all CI jobs. It may load a lot of stuff. but not having
# to rebuild it for every job in a pipeline saves a lot of time.
#
# It can also be pulled from the main repo to use as a base image for a development
# environment in vscode.

FROM rust:1-trixie

# Install all basic tools - not build dependencies. We do them last to maximize layer cacheing
RUN DEBIAN_FRONTEND=noninteractive apt-get update -qy && \
    DEBIAN_FRONTEND=noninteractive apt-get -qy install \
    clang \
    cmake

# Install anything needed to compile successfully
RUN DEBIAN_FRONTEND=noninteractive apt-get -qy install \
    libclang-dev \
    llvm-dev \
    libzip-dev \
    libssl-dev

# Add Rust components
RUN rustup component add rustfmt && \
    rustup component add clippy

# Cleanup the apt cache. If you need to install anything else, you will need to re-run apt update
RUN rm -rf /var/lib/apt/lists

# Install binenv and the tools we use
#
# Note we install, and use, it globally (see https://github.com/devops-works/binenv/blob/develop/SYSTEM.md)
#
ENV BINENV_GLOBAL=true
RUN wget -q https://github.com/devops-works/binenv/releases/download/v0.19.0/binenv_linux_amd64 && \
    wget -q https://github.com/devops-works/binenv/releases/download/v0.19.0/checksums.txt && \
    sha256sum  --check --ignore-missing checksums.txt && \
    mv binenv_linux_amd64 binenv && \
    chmod +x binenv && \
    ./binenv -g update && \
    ./binenv -g install binenv && \
    rm ./binenv

# Do this last, as .binenv.lock can change with each MR. That maximizes layer reuse
COPY .binenv.lock ./.binenv.lock
RUN binenv -g install -l

# Cleanup the apt cache. If you need to install anything else, you will need to re-run apt update
RUN rm -rf /var/lib/apt/lists

