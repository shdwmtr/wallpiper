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

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(dead_code)]

use std::mem::MaybeUninit;

use ash::vk;

mod generated;

/// Bindings for the C `PFN_vkLayerCreateDevice` type defined in the `vk_layer.h` file.
pub use generated::PFN_vkLayerCreateDevice;
/// Bindings for the C `PFN_vkLayerDestroyDevice` type defined in the `vk_layer.h` file.
pub use generated::PFN_vkLayerDestroyDevice;
/// Bindings for the C `PFN_vkSetInstanceLoaderData` type defined in the `vk_layer.h` file.
pub use generated::PFN_vkSetInstanceLoaderData;
/// Sub type of structure for instance and device loader ext of CreateInfo. Bindings for the C
/// `VkLayerDeviceCreateInfo` type defined in the `vk_layer.h` file.
///
/// When `sType` is
/// [`VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkStructureType.html)
/// then [`VkLayerFunction`] indicates struct type pointed to by pNext.
pub use generated::VkLayerDeviceCreateInfo;
/// A list node that contains the next entity's `vkGetInstanceProcAddr` and
/// `vkGetDeviceProcAddr` used by a layer. One possible payload of [`VkLayerDeviceCreateInfo`]
pub use generated::VkLayerDeviceLink;
/// Bindings for the C `VkLayerFunction` type defined in the `vk_layer.h` file.
///
/// Used to distinguish the payload of the loader extension of CreateInfo:
/// [`VkLayerDeviceCreateInfo`], [`VkLayerInstanceCreateInfo`].
pub use generated::VkLayerFunction;
/// A list node that contains the next entity's vkGetInstanceProcAddr used by a layer. One
/// possible payload of [`VkLayerInstanceCreateInfo`].
pub use generated::VkLayerInstanceLink;

type VkInstance = vk::Instance;
type VkPhysicalDevice = vk::PhysicalDevice;
type VkDevice = vk::Device;
type VkStructureType = vk::StructureType;
type VkResult = vk::Result;
type VkDeviceCreateInfo = vk::DeviceCreateInfo;
type VkAllocationCallbacks = vk::AllocationCallbacks;

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VkLoaderFeatureFlags(vk::Flags);
ash::vk_bitflags_wrapped!(VkLoaderFeatureFlags, vk::Flags);
impl VkLoaderFeatureFlags {
    pub const _PHYSICAL_DEVICE_SORTING: Self = Self(0x00000001);
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone)]
pub struct VkLayerInstanceCreateInfoUFieldLayerDeviceField {
    pub pfnLayerCreateDevice: PFN_vkLayerCreateDevice,
    pub pfnLayerDestroyDevice: PFN_vkLayerDestroyDevice,
}

#[repr(C)]
#[derive(Copy, Clone)]
pub union VkLayerInstanceCreateInfoUField {
    pub pLayerInfo: *mut VkLayerInstanceLink,
    pub pfnSetInstanceLoaderData: PFN_vkSetInstanceLoaderData,
    pub layerDevice: VkLayerInstanceCreateInfoUFieldLayerDeviceField,
    pub loaderFeatures: VkLoaderFeatureFlags,
}

impl Default for VkLayerInstanceCreateInfoUField {
    fn default() -> Self {
        let mut s = MaybeUninit::<Self>::uninit();
        unsafe {
            std::ptr::write_bytes(s.as_mut_ptr(), 0, 1);
            s.assume_init()
        }
    }
}

#[repr(C)]
/// Sub type of structure for instance and device loader ext of CreateInfo.
///
/// When `sType` is
/// [`VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkStructureType.html)
/// then [`VkLayerFunction`] indicates struct type pointed to by pNext.
pub struct VkLayerInstanceCreateInfo {
    /// A `VkStructureType` value identifying this struct. Must be
    /// `VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO`.
    pub sType: VkStructureType,
    /// Either `NULL` or a pointer to a structure extending this structure.
    pub pNext: *const ::std::os::raw::c_void,
    /// A [`VkLayerFunction`] value identifying the payload in the `u` field.
    pub function: VkLayerFunction,
    /// The actual payload.
    pub u: VkLayerInstanceCreateInfoUField,
}

unsafe impl vk::TaggedStructure for VkLayerInstanceCreateInfo {
    const STRUCTURE_TYPE: vk::StructureType = vk::StructureType::LOADER_INSTANCE_CREATE_INFO;
}

unsafe impl vk::ExtendsInstanceCreateInfo for VkLayerInstanceCreateInfo {}

unsafe impl vk::TaggedStructure for VkLayerDeviceCreateInfo {
    const STRUCTURE_TYPE: vk::StructureType = vk::StructureType::LOADER_DEVICE_CREATE_INFO;
}

unsafe impl vk::ExtendsDeviceCreateInfo for VkLayerDeviceCreateInfo {}
