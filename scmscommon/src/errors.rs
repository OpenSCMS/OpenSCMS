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

pub mod endpoint;
pub use endpoint::*;

pub mod persistence;
pub use persistence::*;

pub mod conversion;
pub use conversion::*;

pub enum ScmsError {
    PersistenceLoad(persistence::PersistenceLoadError),
    PersistenceStore(persistence::PersistenceStoreError),
    HandleResponse(endpoint::HandleResponseError),
}

pub mod endpoint_error_codes;
pub use endpoint_error_codes::*;

pub mod internal;
pub use internal::*;
