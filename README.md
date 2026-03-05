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
stack, and includes the rest of the stack through the use of relative submodules (see [Getting the Code](#getting-the-code) below).

OpenSCMS is a completely open source implementation of an IEEE1609.2.1-2022 Security Credential Management System (SCMS), built for modularity and high throughput. It's open source nature brings transparency to the implementation of a key component in the V2X world.

For a full description of the OpenSCMS project see the [main README](<https://github.com/OpenSCMS>) or the [website](https://openscms.net).

<!-- omit from toc -->
## Table of Contents

- [Development](#development)
  - [Getting the Code](#getting-the-code)
  - [Installing Dependencies for the C Code](#installing-dependencies-for-the-c-code)
  - [Setting up Rust](#setting-up-rust)
  - [Setting up Minikube for Testing](#setting-up-minikube-for-testing)
  - [Using Docker for Development](#using-docker-for-development)
  - [This is not the docker file you are looking for](#this-is-not-the-docker-file-you-are-looking-for)
- [Running the Server In Minikube](#running-the-server-in-minikube)
- [Exploring the Server](#exploring-the-server)
- [Contributing](#contributing)
- [License](#license)

## Development

We assume a debian-based development environment, ideally Ubuntu 24.04 or newer.

### Getting the Code

The OpenSCMS stack makes heavy use of submodules. Therefore, the clone command should ensure it pulls down all sub modules as follows.

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

Alternatively, you can use the docker image and file provided in the [oscms-ci-docker](https://github.com/OpenSCMS/oscms-ci-docker/blob/main/openscms-server-ci.dockerfile) repository. This is the image used for all CI jobs. It provides a basic environment for building, running clippy or formatting the source, and for running the unit tests. But it is not suitable for deploying to minikube.

This is not the same as the `Dockerfile` in the root of the repository - that one is used by `skaffold` when deploying to `minikube`. For that, you will still need to install Rust and the dependencies on your host machine.

The simplest approach is to pull the provided image into your local store

```bash
docker pull ghcr.io/openscms/openscms-server-ci-docker:latest
```

Now run the container thus

```bash
docker run -ti --rm --volume $PWD:/WORK --user $(id -u):$(id -g) \
       ghcr.io/openscms/oscms-server-ci-docker:latest
```

This will place you in a `bash` shell within the container, with your cloned source available at `/WORK`. Your user inside the container will have the same group and user id as on your host, so any changes you make will have the correct permissions.

It should also be possible to use this image as a `VS Code` development container. The development team has not used it in this way, so if it doesn't work ... contributions are welcome.

### This is not the docker file you are looking for

Ignore the file living in the `.gitlab-ci` sub-directory. This was used during development, which occurred in an environment using Gitlab instead of GitHub. It, and the related YAML files, are retained as examples for those who may be thinking of forking this onto Gitlab

The docker file in the `oscms-ci-docker` repository will always be the authoritative version.

## Running the Server In Minikube

OpenSCMS was developed and tested using `minikube`. Once you have performed the needed [MINIKUBE Setup](./MINIKUBE_SETUP.md), you can start the local server
with the simple command

``` shell
skaffold run
```

This will first build a docker image which is used for each microservice. If this step fails with the following cryptic message about checking the cache

<!--- cSpell:disable --->
``` bash
$ skaffold run
Generating tags...
 - rustscms -> rustscms:09371a4
Checking cache...
 - rustscms: Error checking cache.
getting hash for artifact "rustscms": getting dependencies for "rustscms": file pattern [./certs] must match at least one file
```
<!--- cSpell:enable --->

This means you have not run the script to generate the required certificates (and maybe didn't follow all the MINIKUBE set up instructions). Do so now by running the following

``` shell
scripts/run_test_scms_manager.sh
```

and then issue the `skaffold run` command again.

Once the command completes, and all deployments are ready, you can check the running system with the following command

<!--- cSpell:disable --->
```shell
$ kubectl get pods
NAME                              READY   STATUS    RESTARTS   AGE
aca-deployment-c7784bf4d-tm6jc    1/1     Running   0          14m
cam-deployment-5fb86f586d-v9khb   1/1     Running   0          14m
debugpod-787685b56d-49gdx         1/1     Running   0          14m
eca-deployment-949f95b4-gpdr4     1/1     Running   0          14m
la1-deployment-78468658dd-q5lxh   1/1     Running   0          14m
la2-deployment-85d965b74f-w5ck8   1/1     Running   0          14m
mysql-0                           1/1     Running   0          14m
pma-7487f78bdb-th4v5              1/1     Running   0          14m
ra-deployment-7865966b6b-dztlt    1/1     Running   0          14m
ra-worker-74fdbcb568-h79ht        1/1     Running   0          14m
scmsrabbitmq-server-0             1/1     Running   0          14m
```
<!--- cSpell:enable --->

## Exploring the Server

As deployed, OpenSCMS exposes a number of endpoints to enable access from your host and web browser. You can see these with the command `minikube service list` which should produce something like the following

<!--- cSpell:disable --->
```shell
$ minikube service list
┌─────────────┬──────────────────────┬───────────────────────────┬───────────────────────────┐
│  NAMESPACE  │         NAME         │        TARGET PORT        │            URL            │
├─────────────┼──────────────────────┼───────────────────────────┼───────────────────────────┤
│ default     │ aca                  │ No node port              │                           │
│ default     │ aca-external-service │ aca-external-service/5000 │ http://192.168.49.2:30082 │
│ default     │ cam                  │ No node port              │                           │
│ default     │ cam-external-service │ cam-external-service/5000 │ http://192.168.49.2:30084 │
│ default     │ eca                  │ No node port              │                           │
│ default     │ eca-external-service │ eca-external-service/5000 │ http://192.168.49.2:30085 │
│ default     │ kubernetes           │ No node port              │                           │
│ default     │ la1                  │ No node port              │                           │
│ default     │ la1-external-service │ la1-external-service/5000 │ http://192.168.49.2:30091 │
│ default     │ la2                  │ No node port              │                           │
│ default     │ la2-external-service │ la2-external-service/5000 │ http://192.168.49.2:30092 │
│ default     │ mysql                │ No node port              │                           │
│ default     │ mysql-external       │ mysql-external/3306       │ http://192.168.49.2:30306 │
│ default     │ pma                  │ 80                        │ http://192.168.49.2:30081 │
│ default     │ ra                   │ No node port              │                           │
│ default     │ ra-external-service  │ ra-external-service/5000  │ http://192.168.49.2:30080 │
│ default     │ scmsrabbitmq         │ No node port              │                           │
│ default     │ scmsrabbitmq-nodes   │ No node port              │                           │
│ kube-system │ kube-dns             │ No node port              │                           │
└─────────────┴──────────────────────┴───────────────────────────┴───────────────────────────┘
```
<!--- cSpell:enable --->

In addition to the endpoints for the various microservices, such as RA, you can gain external access to:

- the `mysql` server on port 30306.
- a useful graphical interface to the database via the `pma` service. Just open the provided URL in your browser.
- the OpenAPI JSON definition for all endpoints at `<Service URL>/api-docs/openapi.json` where `<Service URL>` is the exposed URL for the service you are interested in.
- a `Postman` style Swagger-UI API explorer at `<Service URL>/swagger-ui/` (Note the trailing `/`) for each service.

Finally, for really low level poking around or when things won't start up, you will notice a "debug pod" in the output of `kubectl get pods` (`debugpod-787685b56d-49gdx` in the output above). This pod is running a `busybox` image and you can effectively SSH into it with the following command

<!--- cSpell:disable --->
``` shell
 kubectl exec --stdin --tty debugpod-787685b56d-49gdx -- /bin/sh
```
<!--- cSpell:enable --->

Obviously, you will want to disable most of these tools in a production deployment.

## Contributing

Contributions are welcome. Please see the [CONTRIBUTING file](https://github.com/OpenSCMS/.github/blob/main/CONTRIBUTING.md) for details, including the Code of Conduct and C Style Guide.

## License

This project is licensed under the Apache-2.0 License. See the [LICENSE file](./LICENSE) for details.
