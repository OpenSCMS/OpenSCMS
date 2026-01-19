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

use crate::core_logic::gen_linkage_seed::gen_linkage_seeds;
use crate::core_logic::gen_pre_linkage_values::gen_pre_linkage_values;
use crate::core_types::PlvToLsMap;
use crate::errors::EncryptPlvError;
use crate::errors::GenSharedSecretError;
use crate::errors::ProcessPlvRequestError;
use crate::persistence::la_certificates::latest_aca_public_key;
use aes_gcm::Aes256Gcm;
use aes_gcm::KeyInit;
use aes_gcm::aead::Aead;
use aes_gcm::aead::generic_array;
use aes_gcm::aead::generic_array::GenericArray;
use openssl::bn::BigNumContext;
use openssl::derive::Deriver;
use openssl::ec::{EcGroup, EcKey, EcPoint, PointConversionForm};
use openssl::nid::Nid;
use openssl::pkey::{PKey, Private};
use rand::Rng;
use rand::rngs::OsRng;
use scmscommon::PayloadLaToRa;
use scmscommon::PayloadRaToLa;
use scmscommon::core_types::pre_linkage_value::Eplv;
use scmscommon::core_types::pre_linkage_value::Plv;
use scmscommon::plv_payload::PlvPayload;
use sea_orm::DatabaseConnection;

pub async fn process_plv_request(
    payload_ra_to_la: PayloadRaToLa,
    db: &DatabaseConnection,
) -> Result<(PayloadLaToRa, Vec<PlvToLsMap>), ProcessPlvRequestError> {
    // There is one PlvReq for each caterpillar that was sent to RA.
    // Note that each LA is doing the same thing as in here.
    let mut plv_to_ls_mapping: Vec<PlvToLsMap> = Vec::new();
    let mut plv_payloads: Vec<PlvPayload> = Vec::new();
    let request_id = payload_ra_to_la.request_id;
    for plv_req in payload_ra_to_la.plv_reqs {
        // There is one PlvReq per caterpillar.
        // One PlvReq generates a list of plvs and hence eplvs.
        let ls_list = gen_linkage_seeds(plv_req.clone());
        let (plvs, plv_to_ls_map_list) =
            gen_pre_linkage_values(ls_list, plv_req.clone(), request_id.clone());
        let eplvs = encrypt_plvs(plvs, db).await.map_err(|e| {
            ProcessPlvRequestError::new(&format!(
                "Error encrypting PLVs for request_id {}: {:?}",
                request_id, e
            ))
        })?;
        let plv_payload = PlvPayload::new(plv_req.req_id, eplvs);
        plv_to_ls_mapping.extend(plv_to_ls_map_list);
        plv_payloads.push(plv_payload);
    }

    Ok((
        PayloadLaToRa::new(request_id.clone(), plv_payloads),
        plv_to_ls_mapping,
    ))
}

async fn encrypt_plvs(
    plvs: Vec<Plv>,
    db: &DatabaseConnection,
) -> Result<Vec<Eplv>, EncryptPlvError> {
    let mut eplvs = Vec::new();
    for plv in plvs {
        let eplv = encrypt_plv(plv, db).await?;
        eplvs.push(eplv);
    }
    Ok(eplvs)
}

/// In the ACA we should have a function called decrypt_plv.
/// The it sends the PLV (descrypted) to the ACA-C library.
async fn encrypt_plv(plv: Plv, db: &DatabaseConnection) -> Result<Eplv, EncryptPlvError> {
    let aca_pub_key_uncompressed: Vec<u8> = latest_aca_public_key(db).await.map_err(|e| {
        EncryptPlvError::new(&format!(
            "Error fetching latest ACA public key from DB: {:?}",
            e
        ))
    })?;
    let (shared_secret, ephemeral_public_key) = gen_shared_secret(&aca_pub_key_uncompressed)?;
    let (enc_value, nonce) = encrypt_plv_value(shared_secret, plv.value);

    // return the Eplv to be sent to RA
    let eplv = Eplv::new(
        enc_value,
        ephemeral_public_key,
        nonce,
        plv.i_index,
        plv.j_index,
    );

    Ok(eplv)
}

fn gen_nonce() -> GenericArray<u8, generic_array::typenum::U12> {
    let mut rng = OsRng;
    let mut nonce = [0u8; 12];
    rng.fill(&mut nonce);
    GenericArray::clone_from_slice(&nonce)
}

fn encrypt_plv_value(shared_secret: Vec<u8>, plv_value: [u8; 16]) -> ([u8; 32], [u8; 12]) {
    // Convert the shared secret into a 256-bit key
    let key = GenericArray::from_slice(&shared_secret);
    // Create a new AES-GCM cipher with the key
    let cipher = Aes256Gcm::new(key);

    // Encrypt the PLV value using AES-GCM
    // let nonce = GenericArray::from_slice(&[0u8; 12]); // Use a random nonce in production
    let nonce = gen_nonce();
    let ciphertext = cipher
        .encrypt(&nonce, plv_value.as_ref())
        .expect("Encryption failed");

    let eplv_value: [u8; 32] = ciphertext[..]
        .try_into()
        .expect("Vec<u8> is not of length 32");

    let nonce: [u8; 12] = nonce.into();

    (eplv_value, nonce)
}

fn gen_shared_secret(
    peer_public_key_uncompressed: &[u8],
) -> Result<(Vec<u8>, Vec<u8>), GenSharedSecretError> {
    let group = EcGroup::from_curve_name(Nid::X9_62_PRIME256V1)?;

    log::debug!(
        "Peer public key (uncompressed) length: {:?} bytes",
        peer_public_key_uncompressed
    );

    // Check if starts with 0x04 (uncompressed) if not append it
    let peer_public_key_uncompressed = if peer_public_key_uncompressed.len() == 64 {
        let mut uncompressed = vec![0x04];
        uncompressed.extend_from_slice(peer_public_key_uncompressed);
        uncompressed
    } else {
        peer_public_key_uncompressed.to_vec()
    };

    let mut ctx = BigNumContext::new()?;
    let peer_point = EcPoint::from_bytes(&group, &peer_public_key_uncompressed, &mut ctx)?;

    let peer_ec_key = EcKey::from_public_key(&group, &peer_point)?;
    let peer_pkey: PKey<_> = peer_ec_key.try_into()?;

    let shared_key = EcKey::generate(&group)?;

    let ephemeral_public_key =
        shared_key
            .public_key()
            .to_bytes(&group, PointConversionForm::COMPRESSED, &mut ctx)?;

    let shared_pkey: PKey<Private> = shared_key.try_into()?;

    let mut deriver = Deriver::new(&shared_pkey)?;
    deriver.set_peer(&peer_pkey)?;
    let secret = deriver.derive_to_vec()?;

    Ok((secret, ephemeral_public_key))
}

#[cfg(test)]
mod tests {
    use super::*;
    use scmscommon::setup_global_config;

    #[test]
    fn test_gen_shared_secret() {
        let public_key: Vec<u8> = vec![
            0x60, 0x90, 0x18, 0xfe, 0x23, 0x24, 0x0e, 0x28, 0xa3, 0xe2, 0xb6, 0xfb, 0x6c, 0xea,
            0xbe, 0x26, 0x9f, 0x3f, 0xf6, 0xe4, 0x9a, 0xcd, 0x94, 0x74, 0x8d, 0xf5, 0x57, 0x1c,
            0xcb, 0x84, 0x21, 0x69, 0x18, 0xe0, 0x99, 0x29, 0xdd, 0x84, 0x74, 0x70, 0x11, 0xbf,
            0x7b, 0x3f, 0xba, 0x4a, 0x83, 0xed, 0x48, 0x18, 0xa8, 0x89, 0x01, 0x63, 0x86, 0x0e,
            0x8d, 0xe4, 0xb4, 0x7d, 0xf5, 0x98, 0xe7, 0x7d,
        ];

        let (_secret, _shared_public) = gen_shared_secret(&public_key).unwrap();
    }

    #[test]
    fn test_gen_shared_secret_using_aca_pub_key() {
        setup_global_config();

        let aca_public_key: Vec<u8> = vec![
            0x60, 0x90, 0x18, 0xfe, 0x23, 0x24, 0x0e, 0x28, 0xa3, 0xe2, 0xb6, 0xfb, 0x6c, 0xea,
            0xbe, 0x26, 0x9f, 0x3f, 0xf6, 0xe4, 0x9a, 0xcd, 0x94, 0x74, 0x8d, 0xf5, 0x57, 0x1c,
            0xcb, 0x84, 0x21, 0x69, 0x18, 0xe0, 0x99, 0x29, 0xdd, 0x84, 0x74, 0x70, 0x11, 0xbf,
            0x7b, 0x3f, 0xba, 0x4a, 0x83, 0xed, 0x48, 0x18, 0xa8, 0x89, 0x01, 0x63, 0x86, 0x0e,
            0x8d, 0xe4, 0xb4, 0x7d, 0xf5, 0x98, 0xe7, 0x7d,
        ];
        let (_, _) = gen_shared_secret(&aca_public_key).unwrap();
    }

    #[test]
    fn test_encrypt_plv_value() {
        // Define a shared secret and a PLV value for testing
        let shared_secret = vec![
            1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10,
            11, 12, 13, 14, 15, 16,
        ];
        let plv = Plv::new([11; 16], 42, 99);

        // Encrypt the PLV value
        let (_, _) = encrypt_plv_value(shared_secret, plv.value);
    }

    #[test]
    fn test_encrypt_plv_value_2() {
        let public_key: Vec<u8> = vec![
            0x60, 0x90, 0x18, 0xfe, 0x23, 0x24, 0x0e, 0x28, 0xa3, 0xe2, 0xb6, 0xfb, 0x6c, 0xea,
            0xbe, 0x26, 0x9f, 0x3f, 0xf6, 0xe4, 0x9a, 0xcd, 0x94, 0x74, 0x8d, 0xf5, 0x57, 0x1c,
            0xcb, 0x84, 0x21, 0x69, 0x18, 0xe0, 0x99, 0x29, 0xdd, 0x84, 0x74, 0x70, 0x11, 0xbf,
            0x7b, 0x3f, 0xba, 0x4a, 0x83, 0xed, 0x48, 0x18, 0xa8, 0x89, 0x01, 0x63, 0x86, 0x0e,
            0x8d, 0xe4, 0xb4, 0x7d, 0xf5, 0x98, 0xe7, 0x7d,
        ];
        let (shared_secret, _ephemeral_pub_key) = gen_shared_secret(&public_key).unwrap();
        let plv = Plv::new([11; 16], 42, 99);

        // apply encyption
        let (_eplv_value, _nonce) = encrypt_plv_value(shared_secret.clone(), plv.value);
    }
}
