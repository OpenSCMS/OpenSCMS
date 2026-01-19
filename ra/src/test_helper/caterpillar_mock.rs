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

use p256::ecdsa::VerifyingKey;
use scmscommon::core_types::{Caterpillar, CaterpillarObk, CertificateType, ExpansionType};

#[allow(dead_code)]
pub fn mock_caterpillars() -> Vec<Caterpillar> {
    vec![
        scmscommon::Caterpillar::Obk(CaterpillarObk::new(
            11332788824947850967,
            VerifyingKey::from_sec1_bytes(&[
                4, 62, 11, 4, 76, 200, 121, 135, 245, 75, 103, 253, 214, 221, 72, 161, 183, 166,
                178, 26, 188, 203, 74, 1, 255, 3, 180, 114, 166, 89, 53, 226, 107, 165, 253, 88,
                244, 102, 197, 208, 255, 169, 84, 75, 151, 6, 211, 158, 96, 73, 122, 173, 15, 174,
                100, 96, 55, 244, 180, 51, 51, 10, 112, 47, 206,
            ])
            .unwrap(),
            VerifyingKey::from_sec1_bytes(&[
                4, 7, 173, 42, 164, 40, 169, 26, 224, 248, 109, 157, 1, 101, 61, 65, 175, 48, 63,
                205, 179, 233, 37, 255, 103, 28, 30, 182, 244, 78, 112, 172, 215, 184, 195, 209,
                227, 172, 184, 39, 12, 7, 57, 19, 104, 153, 97, 43, 225, 112, 251, 11, 254, 180,
                223, 161, 90, 79, 202, 75, 40, 144, 127, 97, 93,
            ])
            .unwrap(),
            [
                160, 53, 232, 59, 165, 249, 90, 201, 102, 22, 181, 0, 121, 26, 88, 209,
            ],
            [
                177, 153, 113, 189, 17, 88, 40, 143, 179, 59, 7, 189, 41, 202, 30, 51,
            ],
            ExpansionType::Original,
            "c65fc8b49de32b6a".to_string(),
            CertificateType::Explicit,
        )),
        scmscommon::Caterpillar::Obk(CaterpillarObk::new(
            17534142257129900197,
            VerifyingKey::from_sec1_bytes(&[
                4, 206, 198, 49, 242, 62, 134, 29, 166, 214, 153, 153, 2, 189, 167, 152, 78, 212,
                125, 70, 91, 255, 180, 216, 249, 167, 237, 168, 197, 131, 147, 81, 180, 71, 3, 37,
                98, 82, 7, 81, 247, 114, 8, 222, 125, 193, 124, 43, 237, 96, 94, 31, 212, 154, 244,
                42, 101, 115, 172, 191, 0, 103, 84, 223, 254,
            ])
            .unwrap(),
            VerifyingKey::from_sec1_bytes(&[
                4, 24, 204, 128, 205, 63, 168, 236, 111, 80, 186, 163, 165, 234, 147, 131, 68, 92,
                218, 37, 56, 54, 89, 68, 183, 185, 140, 255, 177, 148, 210, 56, 123, 158, 145, 216,
                133, 122, 81, 1, 23, 225, 129, 124, 110, 56, 51, 117, 22, 155, 66, 77, 164, 96, 93,
                242, 60, 246, 90, 182, 46, 220, 15, 47, 124,
            ])
            .unwrap(),
            [
                12, 213, 49, 239, 153, 81, 88, 169, 175, 255, 100, 95, 255, 220, 218, 194,
            ],
            [
                249, 121, 181, 197, 176, 37, 169, 43, 141, 245, 239, 75, 93, 224, 35, 152,
            ],
            ExpansionType::Original,
            "35320f14a4bc2f0f".to_string(),
            CertificateType::Explicit,
        )),
    ]
}
