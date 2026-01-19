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

#[derive(Debug, PartialEq, Clone)]
pub struct Ls {
    pub i_index: u64,
    pub value: [u8; 16],
}

impl Ls {
    pub fn new(i_index: u64, value: [u8; 16]) -> Self {
        Ls { i_index, value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_ls_instance() {
        let i_index = 123456;
        let value = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

        let ls_instance = Ls::new(i_index, value);

        // Define the expected instance
        let expected_ls_instance = Ls {
            i_index: 123456,
            value: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        };

        assert_eq!(ls_instance, expected_ls_instance);
    }
}
