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

# Compiling oscms-bridge using cmake
cd "$LGSCMS_PATH/oscms_bridge/oscms-codecs-bridge/scms_manager" || exit 1
mkdir -p build/
cd build/ || exit 1
cmake .. || exit 1
make DESTDIR=install install


# Running tests for lib1609

$LGSCMS_PATH/oscms_bridge/oscms-codecs-bridge/scms_manager/build/scms_manager "$LGSCMS_PATH"

## Output files are in $LGSCMS_PATH/certs
## $LGSCMS_PATH/certs
## ├── aca
## │   ├── acaCertificate.coer
## │   ├── acaEncPrivateKey.dat
## │   ├── acaEncPublicKey.dat
## │   ├── acaPrivateKey.dat
## │   └── acaPublicKey.dat
## ├── ctl
## │   ├── ccf.coer
## │   └── ctl.coer
## ├── eca
## │   ├── ecaCertificate.coer
## │   ├── ecaEncPrivateKey.dat
## │   ├── ecaEncPublicKey.dat
## │   ├── ecaPrivateKey.dat
## │   └── ecaPublicKey.dat
## ├── electors -> Move to ../electors_certificates
## │   ├── 1_electorCertificate.coer
## │   ├── 1_electorPrivateKey.dat
## │   ├── 1_electorPublicKey.dat
## │   ├── 2_electorCertificate.coer
## │   ├── 2_electorPrivateKey.dat
## │   ├── 2_electorPublicKey.dat
## │   ├── 3_electorCertificate.coer
## │   ├── 3_electorPrivateKey.dat
## │   └── 3_electorPublicKey.dat
## ├── ica 
## │   ├── icaCertificate.coer
## │   ├── icaPrivateKey.dat
## │   └── icaPublicKey.dat
## ├── ra
## │   ├── raCertificate.coer
## │   ├── raEncPrivateKey.dat
## │   ├── raEncPublicKey.dat
## │   ├── raPrivateKey.dat
## │   └── raPublicKey.dat
## └── rootca
##     ├── rootcaCertificate.coer
##     ├── rootcaPrivateKey.dat
##     └── rootcaPublicKey.dat

## Organize $LGSCMS_PATH/certs/ as:
## 1. Create a new folder $LGSCMS_PATH/external_service_files
mkdir -p "$LGSCMS_PATH/external_service_files"

## 2. Copy electors folder to $LGSCMS_PATH/external_service_files/electors_files
mkdir -p "$LGSCMS_PATH/external_service_files/electors_files"
cp -r "$LGSCMS_PATH/certs/electors/"* "$LGSCMS_PATH/external_service_files/electors_files/"

## 3. Copy rootCA folder to $LGSCMS_PATH/external_service_files/root_ca_files
mkdir -p "$LGSCMS_PATH/external_service_files/root_ca_files"
cp -r "$LGSCMS_PATH/certs/rootca/"* "$LGSCMS_PATH/external_service_files/root_ca_files/"

## 4. Copy ica folder to $LGSCMS_PATH/external_service_files/ica_files
mkdir -p "$LGSCMS_PATH/external_service_files/ica_files"
cp -r "$LGSCMS_PATH/certs/ica/"* "$LGSCMS_PATH/external_service_files/ica_files/"

## 5. Remove: aca/acaEncPrivateKey.dat and aca/acaEncPublicKey.dat
rm "$LGSCMS_PATH/certs/aca/acaEncPrivateKey.dat"
rm "$LGSCMS_PATH/certs/aca/acaEncPublicKey.dat"

## 6. Remove: eca/ecaEncPrivateKey.dat and eca/ecaEncPublicKey.dat
rm "$LGSCMS_PATH/certs/eca/ecaEncPrivateKey.dat"
rm "$LGSCMS_PATH/certs/eca/ecaEncPublicKey.dat"

## 7. Remove: ica/icaPrivateKey.dat and ica/icaPublicKey.dat
rm "$LGSCMS_PATH/certs/ica/icaPrivateKey.dat"
rm "$LGSCMS_PATH/certs/ica/icaPublicKey.dat"

## 8. Remove: rootca/rootcaPrivateKey.dat and rootca/rootcaPublicKey.dat
rm "$LGSCMS_PATH/certs/rootca/rootcaPrivateKey.dat"
rm "$LGSCMS_PATH/certs/rootca/rootcaPublicKey.dat"

## 9. Remove: electors/ folder
rm -r "$LGSCMS_PATH/certs/electors/"
