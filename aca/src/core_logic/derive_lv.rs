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

use scmscommon::xor_16_bytes_array;

pub fn derive_lv(plvs: Vec<[u8; 16]>) -> [u8; 16] {
    plvs.into_iter()
        .fold([0u8; 16], |lv, plv| xor_16_bytes_array(&plv, &lv))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_lv() {
        // Define some test vectors
        let input = vec![[0x00; 16], [0xFF; 16], [0xAA; 16]];

        // Expected result is the XOR of all the input vectors
        let expected = [
            0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55, 0x55,
            0x55, 0x55,
        ];

        // Call the function
        let result = derive_lv(input);

        // Assert the result is as expected
        assert_eq!(result, expected);
    }
}
