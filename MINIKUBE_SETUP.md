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
# Setting up your environment for working with minikube

- [Install the tools](#install-the-tools)
- [Configure minikube](#configure-minikube)
- [Start up minikube and check configuration](#start-up-minikube-and-check-configuration)
- [Setup a hostname for accessing minikube](#setup-a-hostname-for-accessing-minikube)

## Install the tools

We use `binenv` to manage the installed versions of all the kubernetes related tools. To install `binenv` follow the instructions at <https://github.com/devops-works/binenv?tab=readme-ov-file#user-install> which boil down to the following

``` shell
wget -q  -O binenv https://github.com/devops-works/binenv/releases/download/v0.19.0/binenv_linux_amd64
chmod +x ./binenv
./binenv update
./binenv install binenv
rm ./binenv
echo 'export PATH=$HOME/.binenv:$PATH' >> $HOME/.bashrc
echo 'source <(binenv completion bash)'
exec $SHELL
```

Next install the tools we use. Their versions can be seen in `.binenv.lock` in the repository root.

``` shell
binenv install -l
```

From this point, any time you enter the directory these tools will be in your path at the selected versions.

## Configure minikube

We use the docker driver for `minikube`, and this should be configured permanently by editing (or creating) to contain the following

``` json
{
  "vm-driver": "docker"
}
```

Now install the `krew` plugin manager for `kubectl`

<!--- cSpell:disable --->
``` shell
(
  set -x; cd "$(mktemp -d)" &&
  OS="$(uname | tr '[:upper:]' '[:lower:]')" &&
  ARCH="$(uname -m | sed -e 's/x86_64/amd64/' -e 's/\(arm\)\(64\)\?.*/\1\2/' -e 's/aarch64$/arm64/')" &&
  KREW="krew-${OS}_${ARCH}" &&
  curl -fsSLO "https://github.com/kubernetes-sigs/krew/releases/latest/download/${KREW}.tar.gz" &&
  tar zxvf "${KREW}.tar.gz" &&
  ./"${KREW}" install krew
)
echo 'export PATH="$HOME/.krew/bin:$PATH"' >> $HOME/.bashrc
exec $SHELL
```
<!--- cSpell:enable --->

Test the installation with the command

```shell
kubectl krew --help`
```

Finally install a plugin to make configuring `RabbitMq` easier.

```shell
kubectl krew install rabbitmq
```

## Start up minikube and check configuration

To start minikube, enter the command:

```shell
minikube start
```

This will take a while the first time as it downloads some large images. Once it completes, do a test deployment of the SCMS.

From the repository root enter the following, which will deploy the app and then close it down again:

```shell
scripts/run_test_scms_manager.sh # to generate the certificates
skaffold run && skaffold delete
```

Once this completes, we can check that the `RabbitMQ` extensions were installed during deployment with the following:

```shell
kubectl get ns
```

Which should list a namespace called `rabbitmq-system`.

## Setup a hostname for accessing minikube

`Minikube` exposes services via ports on an ip address. This IP address can be found by the command `minikube ip`. For convenience, you may wish to add a line to your `/etc/hosts` file to allow it to be accessed easily from the command line and scripts.

```shell
sudo echo "$(minikube ip) minikube" >>/etc/hosts
ping minikube
```
