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

use mockall::mock;

use crate::{DeviceHooks, LayerResult};
use ash::vk;

mock! {
    pub DeviceHooks {}
    impl DeviceHooks for DeviceHooks {
        fn destroy_image<'a>(
            &self,
            _image: vk::Image,
            _p_allocator: Option<&'a vk::AllocationCallbacks>,
        ) -> LayerResult<()>;

        fn destroy_sampler_ycbcr_conversion<'a>(
            &self,
            _ycbcr_conversion: vk::SamplerYcbcrConversion,
            _p_allocator: Option<&'a vk::AllocationCallbacks>,
        ) -> LayerResult<()>;

        fn destroy_swapchain_khr<'a>(
            &self,
            _swapchain: vk::SwapchainKHR,
            _p_allocator: Option<&'a vk::AllocationCallbacks>,
        ) -> LayerResult<()>;
    }
}
