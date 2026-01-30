<!--
Copyright (c) 2025 LG Electronics, Inc.

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.

SPDX-License-Identifier: Apache-2.0
-->

<!-- omit from toc -->
# OpenSCMS Main Server

This repository contains the source code for the OpenSCMS server implementation, written in Rust. It sits at the very top of the OpenSCMS
stack, and includes the rest of the stack through the use of relative submodules (see [Getting the Code](#getting-the-code) below.

OpenSCMS is a completely open source implementation of an IEEE1609.2.1-2022 Security Credential Management System (SCMS), built for modularity and high throughput. It's open source nature brings transparency to the implementation of a key component in the V2X world.

For a full description of the OpenSCMS project see the [main README](<https://github.com/OpenSCMS>) or the [website](https://openscms.net).

<!-- omit from toc -->
## Table of Contents

- [Development](#development)
  - [Installing Dependencies for the C Code](#installing-dependencies-for-the-c-code)
  - [Setting up Rust](#setting-up-rust)
  - [Setting up Minikube for Testing](#setting-up-minikube-for-testing)
  - [Using Docker for Development](#using-docker-for-development)
  - [Getting the Code](#getting-the-code)
- [Contributing](#contributing)
- [License](#license)

## Development

We assume a debian-based development environment, ideally Ubuntu 24.04 or newer.

### Installing Dependencies for the C Code

The following libraries need to be installed on your development machine.

```bash
sudo apt install -y \
    libzip-dev \
    libssl-dev \
    libcurl4-openssl-dev
```

You will also need a set of tools

```bash
sudo apt-get -qy install \
    build-essential \
    clang-format-18 \
    cmake \
    cppcheck \
    curl \
    git \
    gzip \
    pkg-config \
    valgrind \
    wget
```

### Setting up Rust

Once you have all the basic dependencies installed, it's time to add Rust. It's always wise to check the instructions at the [Rust Site](<https://rust-lang.org/tools/install/>), but in short form they are as follows:

<!--- cSpell:disable --->
```bash
# Install Rustup and activate the environment
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Optionally install these addons for database work
cargo install sea-orm-cli
cargo install sqlx-cli
```
<!--- cSpell:enable --->

### Setting up Minikube for Testing

See the separate [Minikube Setup Instructions](./MINIKUBE_SETUP.md)

### Using Docker for Development

Alternatively, you can use the docker file provided in the repository. This is the image used for all CI jobs during the initial development. It provides a basic environment for building, running clippy or formatting the source, and for running the unit tests. But it is not suitable for deploying to minikube.

For that, you will need to install Rust and the dependencies on your host machine.

Now change to the directory where you cloned this repository and build the image as follows

```bash
git clone git@github.com:OpenSCMS/oscms-ci-docker.git
cd oscms-ci-docker
docker build -t oscms-server-docker . -f docker/openscms-server-ci.dockerfile
```

 Now run the container thus

```bash
docker run -ti --rm --volume $PWD:/WORK --user $(id -u):$(id -g) \
       oscms-server-docker
```

This will place you in a `bash` shell within the container, with your cloned source available at `/WORK`. Your user inside the container will have the same group and user id as on your host, so any changes you make will have the correct permissions.

It should also be possible to use this image as a `VS Code` development container. The development team has not used it in this way, so if it doesn't work ... contributions are welcome.

### Getting the Code

The Open SCMS stack makes heavy use of submodules. Therefore, the clone command should ensure it pulls down all sub modules as follows.

```bash
git clone --recurse-submodules git@github.com:OpenSCMS/OpenSCMS.git
```

Obviously, replace the URL with your own if you are cloning a fork.

Due to the use of relative submodule paths, if you are going to fork one repository you will need to fork them all. The alternative is for you to modify the paths in `.gitmodules`, but **DO NOT** commit these changes. Such a pull request will not be accepted.

The list of repositories, and their relative submodule dependencies is as follows

- [OpenSCMS](<https://github.com/OpenSCMS/OpenSCMS>)
  - [oscms-codecs-bridge](<https://github.com/OpenSCMS/oscms-codecs-bridge>)
    - [oscms-asn1c-codecs](<https://github.com/OpenSCMS/oscms-asn1c-codecs>)
      - [oscms-codecs-api](<https://github.com/OpenSCMS/oscms-codecs-api>)
      - [oscms-asn1c-generated](<https://github.com/OpenSCMS/oscms-asn1c-generated>)
        - [etsi_ts103097-asn](<https://github.com/OpenSCMS/etsi_ts103097-asn>)
        - [ieee1609dot2dot1-asn](<https://github.com/OpenSCMS/ieee1609dot2dot1-asn>)

## Contributing

Contributions are welcome. Please see the [CONTRIBUTING file](https://github.com/OpenSCMS/.github/blob/main/CONTRIBUTING.md) for details, including the Code of Conduct and C Style Guide.

## License

This project is licensed under the Apache-2.0 License. See the [LICENSE file](./LICENSE) for details.
