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

use super::certificate_type::CertificateType;
use super::exp_type::ExpansionType;
use derive_more::Display;
use p256::ecdsa::{SigningKey, VerifyingKey};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::serdes::{deserialize_verifying_key, serialize_verifying_key};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub enum Caterpillar {
    Obk(CaterpillarObk),
    Ubk(CaterpillarUbk),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct NonButterflyRequest {
    pub vid: u64,
    pub verifying_key_raw: Vec<u8>,
    pub encryption_key_raw: Option<Vec<u8>>,
    pub hash_id: String,
    pub exp_type: ExpansionType,
    pub certificate_type: CertificateType,
}

impl NonButterflyRequest {
    pub fn new(
        vid: u64,
        verifying_key_raw: Vec<u8>,
        encryption_key_raw: Option<Vec<u8>>,
        hash_id: String,
        exp_type: ExpansionType,
        certificate_type: CertificateType,
    ) -> Self {
        NonButterflyRequest {
            vid,
            verifying_key_raw,
            encryption_key_raw,
            hash_id,
            exp_type,
            certificate_type,
        }
    }

    pub fn get_verifying_key_raw(&self) -> Vec<u8> {
        self.verifying_key_raw.clone()
    }

    pub fn get_hash_id(&self) -> String {
        self.hash_id.clone()
    }

    pub fn get_encryption_key_raw(&self) -> Vec<u8> {
        if let Some(enc_key) = &self.encryption_key_raw {
            enc_key.clone()
        } else {
            vec![]
        }
    }

    pub fn get_exp_type(&self) -> &ExpansionType {
        &self.exp_type
    }

    pub fn get_certificate_type(&self) -> CertificateType {
        self.certificate_type
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CaterpillarObk {
    pub vid: u64,
    #[serde(
        serialize_with = "serialize_verifying_key",
        deserialize_with = "deserialize_verifying_key"
    )]
    pub pub_key_sign: VerifyingKey,
    #[serde(
        serialize_with = "serialize_verifying_key",
        deserialize_with = "deserialize_verifying_key"
    )]
    pub pub_key_encrypt: VerifyingKey,
    pub f_sign: [u8; 16],
    pub f_encrypt: [u8; 16],
    pub exp_type: ExpansionType,
    pub hash_id: String,
    pub certificate_type: CertificateType,
}

impl Caterpillar {
    pub fn get_hash_id(&self) -> String {
        match &self {
            Caterpillar::Obk(x) => x.hash_id.to_owned(),
            Caterpillar::Ubk(x) => x.hash_id.to_owned(),
        }
    }

    pub fn get_exp_type(&self) -> &ExpansionType {
        match &self {
            Caterpillar::Obk(x) => &x.exp_type,
            Caterpillar::Ubk(x) => &x.exp_type,
        }
    }

    pub fn vid(&self) -> u64 {
        match &self {
            Caterpillar::Obk(x) => x.vid,
            Caterpillar::Ubk(x) => x.vid,
        }
    }

    pub fn pub_key_encrypt(&self) -> Option<VerifyingKey> {
        match &self {
            Caterpillar::Obk(x) => Some(x.pub_key_encrypt),
            Caterpillar::Ubk(_) => None,
        }
    }

    pub fn pub_key_sign(&self) -> Option<VerifyingKey> {
        match &self {
            Caterpillar::Obk(x) => Some(x.pub_key_sign),
            Caterpillar::Ubk(x) => Some(x.pub_key_sign),
        }
    }

    pub fn f_encrypt(&self) -> Option<[u8; 16]> {
        match &self {
            Caterpillar::Obk(x) => Some(x.f_encrypt),
            Caterpillar::Ubk(_) => None,
        }
    }

    pub fn f_sign(&self) -> Option<[u8; 16]> {
        match &self {
            Caterpillar::Obk(x) => Some(x.f_sign),
            Caterpillar::Ubk(x) => Some(x.f_sign),
        }
    }

    pub fn get_certificate_type(&self) -> CertificateType {
        match &self {
            Caterpillar::Obk(x) => x.certificate_type,
            Caterpillar::Ubk(x) => x.certificate_type,
        }
    }
}

impl CaterpillarObk {
    pub fn new(
        vid: u64,
        pub_key_sign: VerifyingKey,
        pub_key_encrypt: VerifyingKey,
        f_sign: [u8; 16],
        f_encrypt: [u8; 16],
        exp_type: ExpansionType,
        hash_id: String,
        certificate_type: CertificateType,
    ) -> Self {
        CaterpillarObk {
            vid,
            pub_key_sign,
            pub_key_encrypt,
            f_sign,
            f_encrypt,
            exp_type,
            hash_id,
            certificate_type,
        }
    }

    /// Method to generate a random instance
    pub fn gen_random_example() -> Self {
        let mut rng = OsRng;

        // Generate random values
        let vid = rng.next_u64();
        let signing_key_sign = SigningKey::random(&mut rng);
        let signing_key_encrypt = SigningKey::random(&mut rng);
        let pub_key_sign = VerifyingKey::from(&signing_key_sign);
        let pub_key_encrypt = VerifyingKey::from(&signing_key_encrypt);
        let mut f_sign = [0u8; 16];
        let mut f_encrypt = [0u8; 16];
        rng.fill_bytes(&mut f_sign);
        rng.fill_bytes(&mut f_encrypt);
        let exp_type = ExpansionType::Original;
        let hash_id = "random_hash_id".to_string();
        let certificate_type = CertificateType::Explicit;
        // Return a new instance of the struct
        CaterpillarObk {
            vid,
            pub_key_sign,
            pub_key_encrypt,
            f_sign,
            f_encrypt,
            exp_type,
            hash_id,
            certificate_type,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, ToSchema)]
pub struct CaterpillarUbk {
    pub vid: u64,
    #[serde(
        serialize_with = "serialize_verifying_key",
        deserialize_with = "deserialize_verifying_key"
    )]
    pub pub_key_sign: VerifyingKey,
    pub f_sign: [u8; 16],
    pub exp_type: ExpansionType,
    pub hash_id: String,
    pub certificate_type: CertificateType,
}

impl CaterpillarUbk {
    pub fn new(
        vid: u64,
        pub_key_sign: VerifyingKey,
        f_sign: [u8; 16],
        hash_id: String,
        certificate_type: CertificateType,
        exp_type: ExpansionType,
    ) -> Self {
        CaterpillarUbk {
            vid,
            pub_key_sign,
            f_sign,
            exp_type,
            hash_id,
            certificate_type,
        }
    }

    /// Method to generate a random instance
    pub fn gen_random_example() -> Self {
        let mut rng = OsRng;

        // Generate random values
        let vid = rng.next_u64();
        let signing_key_encrypt = SigningKey::random(&mut rng);
        let pub_key_sign = VerifyingKey::from(&signing_key_encrypt);
        let mut f_sign = [0u8; 16];
        rng.fill_bytes(&mut f_sign);
        let hash_id = "random_hash_id".to_owned();
        let certificate_type = CertificateType::Explicit;

        // Return a new instance of the struct
        Self::new(
            vid,
            pub_key_sign,
            f_sign,
            hash_id,
            certificate_type,
            ExpansionType::Unified,
        )
    }
}

pub enum CaterpillarPriv {
    CaterpillarObk,
    CaterpillarPrivUbk,
}

pub struct CaterpillarPrivObk {
    pub vid: u64,
    pub priv_key_sign: SigningKey,
    pub priv_key_encrypt: SigningKey,
    pub f_sign: [u8; 16],
    pub f_encrypt: [u8; 16],
    pub exp_type: ExpansionType,
    pub certificate_type: CertificateType,
}

pub struct CaterpillarPrivUbk {
    pub vid: u64,
    pub priv_key_sign: SigningKey,
    pub f_sign: [u8; 16],
    pub exp_type: ExpansionType,
    pub certificate_type: CertificateType,
}

/// Initially all caterpillar are set as `ToBeProcessed`.
/// This status controls which records should be filtered when loading the
/// caterpillars for processing.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Display, PartialEq, Eq, ToSchema)]
#[display("{}")]
pub enum CaterpillarStatus {
    #[display("Processed")]
    Processed,
    #[display("Processing")]
    Processing,
    #[display("Queued")]
    Queued,
    #[display("ToBeProcessed")]
    ToBeProcessed,
}
