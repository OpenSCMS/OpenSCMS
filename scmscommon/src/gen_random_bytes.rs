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

use rand::RngCore;

pub fn gen_random_bytes(size: usize) -> Vec<u8> {
    let mut rng = rand::thread_rng();
    let mut random_bytes = vec![0u8; size];

    rng.fill_bytes(&mut random_bytes);

    random_bytes
}

pub fn gen_random_bytes_size_16() -> [u8; 16] {
    let mut rng = rand::thread_rng();
    let mut random_bytes = [0u8; 16];
    rng.fill_bytes(&mut random_bytes);
    random_bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gen_random_bytes_size_16() {
        let random_bytes = gen_random_bytes_size_16();

        // Ensure that the length of the generated array is 16
        assert_eq!(random_bytes.len(), 16);

        // Ensure that the generated array contains random values
        // by comparing it with another array generated using the same function
        let another_random_bytes = gen_random_bytes_size_16();
        assert_ne!(random_bytes, another_random_bytes);
    }
}
