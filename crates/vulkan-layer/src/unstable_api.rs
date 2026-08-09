// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Vulkan layer utilities that are used in the integration tests to implement mock layers. The
//! utilities may benefit the layer implementation, but there is no stability guarantee on those
//! APIs.

#[allow(unused_imports)]
pub use crate::{
    global_simple_intercept::{ApiVersion, Feature},
    lazy_collection::{CheckEmpty, LazyCollection},
    vk_utils::IsCommandEnabled,
};
