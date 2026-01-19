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

use crate::laconfig::LaConfig;
use scmscommon::core_types::PlvReq;
use scmscommon::core_types::linkage_seed::Ls;
use scmscommon::gen_random_bytes_size_16;
use sha2::{Digest, Sha256};

/// This function runs for each caterpillar that the RA received.
/// And each LA is doing the same thing as in here.
/// In the function we'll not be using the j_max yet.
/// The j_max is used when generating the PLV.
pub fn gen_linkage_seeds(plv_req: PlvReq) -> Vec<Ls> {
    let mut ls_list: Vec<Ls> = Vec::new(); // Vector to hold the Ls values

    let linkage_seed_0 = Ls::new(plv_req.i_min, gen_random_bytes_size_16());
    ls_list.push(linkage_seed_0.clone()); // Push the first Ls value into the vector
    let mut previous_ls = linkage_seed_0;

    // In this for loop, jump the first value and include the last one
    for _ in (plv_req.i_min + 1)..=plv_req.i_max {
        let current_ls = gen_next_ls(previous_ls);
        ls_list.push(current_ls.clone()); // Push the current Ls value into the vector
        previous_ls = current_ls;
    }

    ls_list // Return the vector containing all generated Ls values
}

// Create new LS by using previous LS
// This function is not using security strings nor linkage hooks
fn gen_next_ls(previous_ls: Ls) -> Ls {
    let la_id = LaConfig::global().la_id.to_le_bytes();
    let previous_ls_value = previous_ls.value; // Assuming previous_ls is an instance of Ls

    let mut concat_vec: Vec<u8> = Vec::new();
    concat_vec.extend_from_slice(&la_id);
    concat_vec.extend_from_slice(&previous_ls_value);

    let hashed_value = {
        let mut hasher = Sha256::new();
        hasher.update(&concat_vec);
        hasher.finalize()
    };

    // Convert the hashed value into a [u8; 16] array by taking the first 16 bytes
    let mut hashed_array = [0u8; 16];
    hashed_array.copy_from_slice(&hashed_value[..16]);

    let next_time_index = previous_ls.i_index + 1;

    Ls::new(next_time_index, hashed_array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helper::setup_la_conf;

    #[test]
    fn test_gen_next_ls() {
        setup_la_conf();

        // Create a sample previous Ls instance
        let previous_ls = Ls::new(
            123456,
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        );

        // Generate the next Ls instance using the gen_next_ls function
        let next_ls = gen_next_ls(previous_ls.clone());

        // Calculate the expected time index for the next Ls instance
        let expected_next_time_index = previous_ls.i_index + 1;

        // Create the expected Ls instance using the expected time index and the hashed value
        let mut hasher = Sha256::new();
        let mut concat_vec: Vec<u8> = Vec::new();
        concat_vec.extend_from_slice(&LaConfig::global().la_id.to_le_bytes());
        concat_vec.extend_from_slice(&previous_ls.value);
        hasher.update(&concat_vec);
        let hashed_value = hasher.finalize();
        let mut expected_hashed_array = [0u8; 16];
        expected_hashed_array.copy_from_slice(&hashed_value[..16]);
        let expected_next_ls = Ls::new(expected_next_time_index, expected_hashed_array);

        // Assert that the generated next Ls instance matches the expected next Ls instance
        assert_eq!(next_ls, expected_next_ls);
    }

    #[test]
    fn test_gen_linkage_seeds() {
        setup_la_conf();

        // Define a PlvReq instance for testing
        let plv_req = PlvReq {
            req_id: 1,
            ra_id: 2,
            i_min: 0,
            i_max: 5,
            j_max: 0, // We're not using j_max yet in this function
        };

        // Generate linkage seeds using the gen_linkage_seeds function
        let ls_list = gen_linkage_seeds(plv_req.clone());

        // Assert that the length of ls_list is equal to i_max - i_min
        assert_eq!(ls_list.len(), (plv_req.i_max - plv_req.i_min) as usize + 1);

        // Assert that the time index of the first Ls value in the list is equal to i_min
        assert_eq!(ls_list[0].i_index, plv_req.i_min);

        // Assert that each subsequent Ls value in the list has a time index incremented by 1 compared to the previous one
        for i in 1..ls_list.len() {
            assert_eq!(ls_list[i].i_index, ls_list[i - 1].i_index + 1);
        }
    }
}
