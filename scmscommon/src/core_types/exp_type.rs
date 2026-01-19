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

use derive_more::Display;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Display, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, ToSchema)]
#[display("{}")]
pub enum ExpansionType {
    #[display("Obk")]
    Original,
    #[display("Ubk")]
    Unified,
    #[display("Cubk")]
    Compact,
    #[display("NonButterfly")]
    NonButterfly,
    #[display("NonButterflyEncrypted")]
    NonButterflyEncrypted,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exp_type_to_string() {
        let exp_type = ExpansionType::Compact;
        let s = exp_type.to_string();
        assert_eq!(s, "Cubk");
    }
}
