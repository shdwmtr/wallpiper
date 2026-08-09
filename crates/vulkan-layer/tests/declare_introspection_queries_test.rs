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

use std::ffi::{c_char, CStr, CString};

use ash::vk::{self};

use mockall::automock;
use vulkan_layer_macros::declare_introspection_queries;

#[test]
fn test_declare_introspection_queries_should_just_forward_to_global() {
    #[automock]
    trait Global {
        unsafe extern "system" fn enumerate_instance_layer_properties(
            property_count: *mut u32,
            properties: *mut vk::LayerProperties,
        ) -> vk::Result;
        unsafe extern "system" fn enumerate_instance_extension_properties(
            layer_name: *const c_char,
            property_count: *mut u32,
            extension_properties: *mut vk::ExtensionProperties,
        ) -> vk::Result;
        unsafe extern "system" fn enumerate_device_layer_properties(
            physical_device: vk::PhysicalDevice,
            p_property_count: *mut u32,
            p_properties: *mut vk::LayerProperties,
        ) -> vk::Result;
        unsafe extern "system" fn enumerate_device_extension_properties(
            physical_device: vk::PhysicalDevice,
            p_layer_name: *const c_char,
            p_property_count: *mut u32,
            p_properties: *mut vk::ExtensionProperties,
        ) -> vk::Result;
        unsafe extern "system" fn get_instance_proc_addr(
            instance: vk::Instance,
            p_name: *const c_char,
        ) -> vk::PFN_vkVoidFunction;
        unsafe extern "system" fn get_device_proc_addr(
            device: vk::Device,
            p_name: *const c_char,
        ) -> vk::PFN_vkVoidFunction;
    }

    declare_introspection_queries!(MockGlobal);

    {
        const COUNT: usize = 3;
        let mut layer_properties: [vk::LayerProperties; COUNT] = Default::default();
        let mut out_count: u32 = COUNT as _;
        let return_val = vk::Result::ERROR_UNKNOWN;

        let ctx = MockGlobal::enumerate_instance_layer_properties_context();
        // Raw pointers are not Send, so we need to use `withf_st`.
        ctx.expect()
            .once()
            .withf_st({
                let expect_p_count = &mut out_count as *mut _;
                let expect_p_layer_properties = layer_properties.as_mut_ptr();
                move |p_count, p_layer_properties| {
                    (*p_count, *p_layer_properties) == (expect_p_count, expect_p_layer_properties)
                }
            })
            .return_const(return_val);

        assert_eq!(
            unsafe {
                vkEnumerateInstanceLayerProperties(&mut out_count, layer_properties.as_mut_ptr())
            },
            return_val
        );
    }
    {
        let layer_name = CString::new("VK_LAYER_UNKNOWN_test").unwrap();
        const COUNT: usize = 3;
        let mut property_count = COUNT as _;
        let mut extension_properties: [vk::ExtensionProperties; COUNT] = Default::default();
        let return_val = vk::Result::ERROR_UNKNOWN;

        let ctx = MockGlobal::enumerate_instance_extension_properties_context();
        ctx.expect()
            .once()
            .withf_st({
                let expect_layer_name = layer_name.as_ptr() as *const c_char;
                let expect_p_property_count = &mut property_count as *mut _;
                let expect_p_extension_properties = &mut extension_properties as *mut _;
                move |layer_name, property_count, extension_properties| {
                    (*layer_name, *property_count, *extension_properties)
                        == (
                            expect_layer_name,
                            expect_p_property_count,
                            expect_p_extension_properties,
                        )
                }
            })
            .return_const(return_val);

        assert_eq!(
            unsafe {
                vkEnumerateInstanceExtensionProperties(
                    layer_name.as_ptr(),
                    &mut property_count,
                    extension_properties.as_mut_ptr(),
                )
            },
            return_val
        );
    }
    {
        // We use transmute instead of Handle::from_raw here to avoid integer to pointer cast, and
        // allow the miri tests with tree borrows to work with this test. See
        // https://github.com/ash-rs/ash/issues/996 for details.
        let physical_device =
            unsafe { std::mem::transmute::<*const u8, vk::PhysicalDevice>(std::ptr::dangling()) };
        const COUNT: usize = 3;
        let mut count = COUNT as u32;
        let mut layer_properties: [vk::LayerProperties; COUNT] = Default::default();
        let return_val = vk::Result::ERROR_UNKNOWN;

        let ctx = MockGlobal::enumerate_device_layer_properties_context();
        ctx.expect()
            .once()
            .withf_st({
                let expect_physical_device = physical_device;
                let expect_p_count = &mut count as *mut _;
                let expect_p_layer_properties = &mut layer_properties as *mut _;
                move |physical_device, p_property_count, p_properties| {
                    (*physical_device, *p_property_count, *p_properties)
                        == (
                            expect_physical_device,
                            expect_p_count,
                            expect_p_layer_properties,
                        )
                }
            })
            .return_const(return_val);

        assert_eq!(
            unsafe {
                vkEnumerateDeviceLayerProperties(
                    physical_device,
                    &mut count,
                    layer_properties.as_mut_ptr(),
                )
            },
            return_val
        );
    }
    {
        // We use transmute instead of Handle::from_raw here to avoid integer to pointer cast, and
        // allow the miri tests with tree borrows to work with this test. See
        // https://github.com/ash-rs/ash/issues/996 for details.
        let instance =
            unsafe { std::mem::transmute::<*const u8, vk::Instance>(std::ptr::dangling()) };
        let name = CString::new("vkCreateInstance").unwrap();
        extern "system" fn fake_create_instance(
            _: *const vk::InstanceCreateInfo,
            _: *const vk::AllocationCallbacks,
            _: *mut vk::Instance,
        ) -> vk::Result {
            unimplemented!();
        }
        let fake_create_instance_fp: vk::PFN_vkVoidFunction =
            unsafe { std::mem::transmute(fake_create_instance as vk::PFN_vkCreateInstance) };

        let ctx = MockGlobal::get_instance_proc_addr_context();
        ctx.expect()
            .once()
            .withf_st({
                let expect_instance = instance;
                let expect_name = name.clone();
                move |instance, name| {
                    let name = unsafe { CStr::from_ptr(*name) };
                    (*instance, name) == (expect_instance, expect_name.as_c_str())
                }
            })
            .return_const(fake_create_instance_fp);

        assert_eq!(
            unsafe { vkGetInstanceProcAddr(instance, name.as_ptr()) },
            fake_create_instance_fp
        );
    }
    {
        // We use transmute instead of Handle::from_raw here to avoid integer to pointer cast, and
        // allow the miri tests with tree borrows to work with this test. See
        // https://github.com/ash-rs/ash/issues/996 for details.
        let device = unsafe { std::mem::transmute::<*const u8, vk::Device>(std::ptr::dangling()) };
        let name = CString::new("vkAllocateMemory").unwrap();
        extern "system" fn fake_allocate_memory(
            _: vk::Device,
            _: *const vk::MemoryAllocateInfo,
            _: *const vk::AllocationCallbacks,
            _: *mut vk::DeviceMemory,
        ) -> vk::Result {
            unimplemented!()
        }
        let fake_allocate_memory_fp: vk::PFN_vkVoidFunction =
            unsafe { std::mem::transmute(fake_allocate_memory as vk::PFN_vkAllocateMemory) };

        let ctx = MockGlobal::get_device_proc_addr_context();
        ctx.expect()
            .once()
            .withf_st({
                let expect_device = device;
                let expect_name = name.clone();
                move |device, name| {
                    let name = unsafe { CStr::from_ptr(*name) };
                    (*device, name) == (expect_device, expect_name.as_c_str())
                }
            })
            .return_const(fake_allocate_memory_fp);

        assert_eq!(
            unsafe { vkGetDeviceProcAddr(device, name.as_ptr()) },
            fake_allocate_memory_fp
        );
    }
}
