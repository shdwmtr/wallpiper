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

use ash::vk::{self};
use mockall::predicate::{always, eq};
use once_cell::sync::Lazy;
use parking_lot::{Mutex, MutexGuard};
use std::{
    ffi::{CStr, CString},
    iter::zip,
    marker::PhantomData,
    mem::MaybeUninit,
    ptr::{null, null_mut},
    sync::{atomic::AtomicBool, Arc, LazyLock},
};
use vulkan_layer::{
    test_utils::{
        LayerManifestExt, MockTestLayer, Tag, TestGlobal, TestLayer, VkLayerDeviceCreateInfo,
        VkLayerDeviceLink, VkLayerFunction, VkLayerInstanceCreateInfo,
    },
    unstable_api::ApiVersion,
    DeviceInfo, Extension, ExtensionProperties, Global, InstanceInfo, Layer, LayerManifest,
    LayerResult, LayerVulkanCommand, StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
    VkLayerInstanceLink, VulkanBaseInStructChain,
};

pub mod utils;

use utils::{
    create_entry, DeviceContext, DeviceData, FromVulkanHandle, InstanceContext,
    InstanceCreateInfoExt, InstanceData, Layers,
};

use crate::utils::ArcDelInstanceContextExt;

mod get_instance_proc_addr {
    use super::*;
    mod null_instance {
        use super::*;
        #[test]
        fn test_should_return_fp_when_called_with_get_instance_proc_addr_name() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();

            let entry = create_entry::<TestLayer>();
            let get_instance_proc_addr_name = c"vkGetInstanceProcAddr".as_ptr();
            let get_instance_proc_addr = unsafe {
                entry.get_instance_proc_addr(vk::Instance::null(), get_instance_proc_addr_name)
            }
            .expect("fp should be returned");
            // Verify that this is a function pointer that can be called.
            let get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr =
                unsafe { std::mem::transmute(get_instance_proc_addr) };
            unsafe { get_instance_proc_addr(vk::Instance::null(), get_instance_proc_addr_name) };
        }

        #[test]
        fn test_should_return_fp_when_called_with_global_command_name() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();

            let entry = create_entry::<TestLayer>();
            let layer_name = CString::new(TestLayer::<Tag<0>>::manifest().name).unwrap();
            entry
                .enumerate_instance_extension_properties(Some(&layer_name))
                .unwrap();
            entry.enumerate_instance_layer_properties().unwrap();
            // Verify if the returned function pointer can be called in other tests.
            unsafe {
                entry.get_instance_proc_addr(vk::Instance::null(), c"vkCreateInstance".as_ptr())
            }
            .expect("vkCreateInstance should be a valid function pointer.");
        }

        #[test]
        fn test_should_return_null_when_called_with_core_instance_dispatchable_command() {
            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
                let _ctx = TEST_GLOBAL.create_context();
                let entry = create_entry::<TestLayer>();
                assert!(unsafe {
                    entry
                        .get_instance_proc_addr(vk::Instance::null(), c"vkDestroyInstance".as_ptr())
                }
                .is_none());
            }

            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                    .set_layer_mock_builder(|| {
                        let mut mock = MockTestLayer::default();
                        mock.expect_hooked_instance_commands().return_const(vec![
                            LayerVulkanCommand::GetPhysicalDeviceSparseImageFormatProperties2,
                        ]);
                        mock.set_default_expectations();
                        mock
                    })
                    .build();
                let _ctx = TEST_GLOBAL.create_context();
                let entry = create_entry::<TestLayer>();
                assert!(unsafe {
                    entry.get_instance_proc_addr(
                        vk::Instance::null(),
                        c"vkGetPhysicalDeviceSparseImageFormatProperties2".as_ptr(),
                    )
                }
                .is_none());
            }
        }

        #[test]
        #[ignore]
        fn test_should_return_null_when_called_with_available_instance_extension_command() {
            todo!(concat!(
                "This test is meaningless until we implement the mechanism for the layer to add ",
                "instance extensions"
            ))
        }

        #[test]
        fn test_should_return_null_when_called_with_available_device_extension_commands() {
            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
                let _ctx = TEST_GLOBAL.create_context();
                let entry = create_entry::<TestLayer>();
                assert!(unsafe {
                    entry.get_instance_proc_addr(
                        vk::Instance::null(),
                        c"vkDestroySwapchainKHR".as_ptr(),
                    )
                }
                .is_none());
            }

            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                    .set_layer_mock_builder(|| {
                        let mut mock = MockTestLayer::default();
                        mock.expect_manifest().return_const({
                            let mut layer_manifest = LayerManifest::test_default();
                            layer_manifest.device_extensions = &[ExtensionProperties {
                                name: Extension::KHRSwapchain,
                                spec_version: 1,
                            }];
                            layer_manifest
                        });
                        mock.set_default_expectations();
                        mock
                    })
                    .build();
                let _ctx = TEST_GLOBAL.create_context();
                let entry = create_entry::<TestLayer>();
                assert!(unsafe {
                    entry.get_instance_proc_addr(
                        vk::Instance::null(),
                        c"vkDestroySwapchainKHR".as_ptr(),
                    )
                }
                .is_none());
            }
        }
    }

    mod valid_instance {
        use super::*;

        #[test]
        #[cfg_attr(miri, ignore = "https://github.com/google/vk-layer-for-rust/issues/49")]
        fn test_should_return_fp_for_get_instance_proc_addr() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext {
                entry, instance, ..
            } = ctx.as_ref();
            let get_instance_proc_addr_name = c"vkGetInstanceProcAddr".as_ptr();
            let get_instance_proc_addr = unsafe {
                entry.get_instance_proc_addr(instance.handle(), get_instance_proc_addr_name)
            }
            .unwrap();
            let get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr =
                unsafe { std::mem::transmute(get_instance_proc_addr) };
            assert_eq!(
                get_instance_proc_addr,
                entry.static_fn().get_instance_proc_addr
            );
        }

        #[test]
        fn test_should_return_fp_for_core_dispatchable_command() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext {
                entry, instance, ..
            } = ctx.as_ref();
            let destroy_instance = unsafe {
                entry.get_instance_proc_addr(instance.handle(), c"vkDestroyInstance".as_ptr())
            };
            assert!(destroy_instance.is_some());
            // vkDestroyInstance will be called in the destructor of the instance context.
        }

        #[test]
        fn test_should_return_next_proc_addr_for_not_intercepted_command() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();

            let enabled_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
            let ctx = vk::InstanceCreateInfo::builder()
                .enabled_extension_names(&enabled_extensions)
                .default_instance::<(TestLayer,)>();

            let InstanceContext {
                entry,
                instance,
                icd_entry,
                ..
            } = ctx.as_ref();
            let destroy_surface_name = c"vkDestroySurfaceKHR".as_ptr();
            let destroy_surface =
                unsafe { entry.get_instance_proc_addr(instance.handle(), destroy_surface_name) }
                    .map(|fp| fp as usize);
            // We don't wrap the object, so the VkInstance should be the same.
            let next_destroy_surface = unsafe {
                icd_entry.get_instance_proc_addr(instance.handle(), destroy_surface_name)
            }
            .map(|fp| fp as usize);
            assert_eq!(destroy_surface, next_destroy_surface);
        }

        #[test]
        fn test_should_return_fp_for_enabled_instance_extension_command() {
            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
                let _ctx = TEST_GLOBAL.create_context();

                let application_info =
                    vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_1);
                let ctx = vk::InstanceCreateInfo::builder()
                    .application_info(&application_info)
                    .default_instance::<(TestLayer,)>();

                let InstanceContext {
                    entry, instance, ..
                } = ctx.as_ref();
                let fp = unsafe {
                    entry.get_instance_proc_addr(
                        instance.handle(),
                        c"vkGetPhysicalDeviceSparseImageFormatProperties2".as_ptr(),
                    )
                };
                assert!(fp.is_some());
            }

            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                    .set_layer_mock_builder(|| {
                        let mut mock = MockTestLayer::default();
                        mock.expect_hooked_instance_commands()
                            .return_const(vec![LayerVulkanCommand::DestroySurfaceKhr]);
                        mock.set_default_expectations();
                        mock
                    })
                    .build();
                let _ctx = TEST_GLOBAL.create_context();

                let enabled_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
                let ctx = vk::InstanceCreateInfo::builder()
                    .enabled_extension_names(&enabled_extensions)
                    .default_instance::<(TestLayer,)>();
                let InstanceContext {
                    entry, instance, ..
                } = ctx.as_ref();
                let destroy_surface = unsafe {
                    entry.get_instance_proc_addr(instance.handle(), c"vkDestroySurfaceKHR".as_ptr())
                };
                let destroy_surface: vk::PFN_vkDestroySurfaceKHR =
                    unsafe { std::mem::transmute(destroy_surface.unwrap()) };
                let instance_hooks_mock = &TestLayer::<Tag<0>>::global_instance()
                    .layer_info
                    .get_instance_info(instance.handle())
                    .unwrap()
                    .mock_hooks;
                instance_hooks_mock
                    .lock()
                    .unwrap()
                    .expect_destroy_surface_khr()
                    .times(1)
                    .return_const(LayerResult::Unhandled);
                unsafe { destroy_surface(instance.handle(), vk::SurfaceKHR::null(), null()) };
            }
            // TODO: when we allow customize instance layer extensions, also test with the extension
            // provided by the layer
        }

        #[test]
        fn test_if_extension_is_not_enabled_null_should_be_returned_for_not_hooked_proc() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext {
                entry, instance, ..
            } = ctx.as_ref();
            let fp = unsafe {
                entry.get_instance_proc_addr(instance.handle(), c"vkDestroySurfaceKHR".as_ptr())
            };
            assert!(fp.is_none());
            let fp = unsafe {
                entry.get_instance_proc_addr(
                    instance.handle(),
                    c"vkGetPhysicalDeviceSparseImageFormatProperties2".as_ptr(),
                )
            };
            assert!(fp.is_none());
        }

        #[test]
        fn test_if_extension_is_not_enabled_null_should_be_returned_for_hooked_proc() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                .set_layer_mock_builder(|| {
                    let mut mock = MockTestLayer::default();
                    mock.expect_hooked_instance_commands().return_const(vec![
                        LayerVulkanCommand::DestroySurfaceKhr,
                        LayerVulkanCommand::GetPhysicalDeviceSparseImageFormatProperties2,
                    ]);
                    mock.set_default_expectations();
                    mock
                })
                .build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext {
                entry, instance, ..
            } = ctx.as_ref();
            let fp = unsafe {
                entry.get_instance_proc_addr(instance.handle(), c"vkDestroySurfaceKHR".as_ptr())
            };
            assert!(fp.is_none());
            let fp = unsafe {
                entry.get_instance_proc_addr(
                    instance.handle(),
                    c"vkGetPhysicalDeviceSparseImageFormatProperties2".as_ptr(),
                )
            };
            assert!(fp.is_none());
        }

        #[test]
        fn test_commands_that_should_always_be_intercepted() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();
            let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_3);
            let ctx = vk::InstanceCreateInfo::builder()
                .application_info(&app_info)
                .default_instance::<(TestLayer,)>();
            let InstanceContext {
                entry,
                instance,
                icd_entry,
                ..
            } = ctx.as_ref();
            // TODO: test the actual logic of different functions to remove this test.
            // vkEnumerateInstanceLayerProperties,
            // vkEnumerateInstanceExtensionProperties, vkCreateInstance are global
            // commands, and can't be queried with a valid VkInstance.
            let commands_must_be_intercepted: &[&'static CStr] = &[
                // 4 Android introspection queries.
                c"vkEnumerateDeviceLayerProperties",
                c"vkEnumerateDeviceExtensionProperties",
                c"vkGetInstanceProcAddr",
                c"vkGetDeviceProcAddr",
                // 2 Physical device enumeration APIs.
                c"vkEnumeratePhysicalDevices",
                c"vkEnumeratePhysicalDeviceGroups",
                // 3 device, instance creation and destruction APIs.
                c"vkDestroyInstance",
                c"vkCreateDevice",
                c"vkDestroyDevice",
            ];
            for command in commands_must_be_intercepted {
                let fp =
                    unsafe { entry.get_instance_proc_addr(instance.handle(), command.as_ptr()) }
                        .map(|fp| fp as usize);
                let next_fp = unsafe {
                    icd_entry.get_instance_proc_addr(instance.handle(), command.as_ptr())
                }
                .map(|fp| fp as usize);
                assert_ne!(
                    fp,
                    next_fp,
                    "The function pointer to {} should be different.",
                    command.to_string_lossy()
                );
            }
        }

        #[test]
        fn test_should_return_fp_with_available_device_dispatch_command() {
            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
                let _ctx = TEST_GLOBAL.create_context();
                let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
                let InstanceContext {
                    instance, entry, ..
                } = ctx.as_ref();
                let instance_data = unsafe { InstanceData::from_handle(instance.handle()) };
                instance_data.set_available_device_extensions(&[Extension::KHRSwapchain]);
                let destroy_swapchain = unsafe {
                    entry.get_instance_proc_addr(
                        instance.handle(),
                        c"vkDestroySwapchainKHR".as_ptr(),
                    )
                };
                assert!(destroy_swapchain.is_some());
            }

            {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                    .set_layer_mock_builder(|| {
                        let mut mock = MockTestLayer::default();
                        mock.expect_hooked_device_commands()
                            .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                        mock.set_default_expectations();
                        mock
                    })
                    .build();
                let _ctx = TEST_GLOBAL.create_context();
                let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
                let InstanceContext {
                    instance, entry, ..
                } = ctx.as_ref();
                let instance_data = unsafe { InstanceData::from_handle(instance.handle()) };
                instance_data.set_available_device_extensions(&[Extension::KHRSwapchain]);
                let destroy_swapchain = unsafe {
                    entry.get_instance_proc_addr(
                        instance.handle(),
                        c"vkDestroySwapchainKHR".as_ptr(),
                    )
                };
                assert!(destroy_swapchain.is_some());
            }
        }

        mod icd_unavaiilable_device_dispatch_command {
            use super::*;
            #[test]
            fn test_should_return_null_if_layer_also_doesnt_implement_the_extension() {
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                    .set_layer_mock_builder(|| {
                        let mut mock = MockTestLayer::default();
                        mock.expect_hooked_device_commands()
                            .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                        mock.set_default_expectations();
                        mock
                    })
                    .build();
                let _ctx = TEST_GLOBAL.create_context();
                let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
                let InstanceContext {
                    instance,
                    entry,
                    icd_entry,
                    ..
                } = ctx.as_ref();
                let instance_data = unsafe { InstanceData::from_handle(instance.handle()) };
                instance_data.set_available_device_extensions(&[]);

                // The mock ICD should return NULL.
                let destroy_swapchain = unsafe {
                    icd_entry.get_instance_proc_addr(
                        instance.handle(),
                        c"vkDestroySwapchainKHR".as_ptr(),
                    )
                };
                assert!(destroy_swapchain.is_none());

                // The layer should also return NULL.
                let destroy_swapchain = unsafe {
                    entry.get_instance_proc_addr(
                        instance.handle(),
                        c"vkDestroySwapchainKHR".as_ptr(),
                    )
                };
                assert!(destroy_swapchain.is_none());
            }

            #[test]
            fn test_should_return_fp_if_layer_implements_the_extension() {
                let instance_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
                let device_extensions = [vk::KhrSwapchainFn::name().as_ptr()];
                static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                    .set_layer_mock_builder(|| {
                        let mut mock = MockTestLayer::default();
                        mock.expect_manifest().return_const({
                            let mut layer_manifest = LayerManifest::test_default();
                            layer_manifest.device_extensions = &[ExtensionProperties {
                                name: Extension::KHRSwapchain,
                                spec_version: 1,
                            }];
                            layer_manifest
                        });
                        mock.expect_hooked_device_commands()
                            .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                        mock.set_default_expectations();
                        mock
                    })
                    .build();
                let _ctx = TEST_GLOBAL.create_context();
                let instance_ctx = vk::InstanceCreateInfo::builder()
                    .enabled_extension_names(&instance_extensions)
                    .default_instance::<(TestLayer,)>();
                let InstanceContext { instance, .. } = instance_ctx.as_ref();
                unsafe { InstanceData::from_handle(instance.handle()) }
                    .set_available_device_extensions(&[]);
                let device_ctx = instance_ctx
                    .clone()
                    .create_device_context(|create_info_builder, create_device| {
                        let create_info_builder =
                            create_info_builder.enabled_extension_names(&device_extensions);
                        create_device(create_info_builder);
                    })
                    .unwrap();
                let DeviceContext { device, .. } = device_ctx.as_ref();
                let layer_device_info = TestLayer::<Tag<0>>::global_instance()
                    .layer_info
                    .get_device_info(device_ctx.device.handle())
                    .unwrap();
                layer_device_info
                    .hooks()
                    .expect_destroy_swapchain_khr()
                    .with(eq(vk::SwapchainKHR::null()), always())
                    .once()
                    .return_const(LayerResult::Handled(()));
                let destroy_swapchain_khr = unsafe {
                    instance
                        .get_device_proc_addr(device.handle(), c"vkDestroySwapchainKHR".as_ptr())
                }
                .unwrap();
                let destroy_swapchain_khr: vk::PFN_vkDestroySwapchainKHR =
                    unsafe { std::mem::transmute(destroy_swapchain_khr) };
                unsafe { destroy_swapchain_khr(device.handle(), vk::SwapchainKHR::null(), null()) };
            }
        }

        #[test]
        fn test_should_call_into_next_get_instance_proc_addr_with_unknown_name() {
            let new_command_name = "vkTestCommand";
            extern "system" fn test_command() {
                unimplemented!()
            }
            let test_command = test_command as unsafe extern "system" fn();

            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext {
                instance,
                entry,
                icd_entry,
                ..
            } = ctx.as_ref();
            let instance_data = unsafe { InstanceData::from_handle(instance.handle()) };
            instance_data.add_instance_command(new_command_name, Some(test_command));
            let new_command_name_cstr = CString::new(new_command_name).unwrap();

            // Check what mock ICD returns.
            let actual_new_command = unsafe {
                icd_entry.get_instance_proc_addr(instance.handle(), new_command_name_cstr.as_ptr())
            };
            assert_eq!(
                actual_new_command.map(|fp| fp as usize),
                Some(test_command as usize)
            );

            // Check what the layer returns.
            let actual_new_command = unsafe {
                entry.get_instance_proc_addr(instance.handle(), new_command_name_cstr.as_ptr())
            };
            assert_eq!(actual_new_command.unwrap(), test_command);
        }

        #[test]
        fn test_should_return_null_with_unsupported_device_commands() {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                .set_layer_mock_builder(|| {
                    let mut mock = MockTestLayer::default();
                    mock.expect_hooked_device_commands()
                        .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                    mock.set_default_expectations();
                    mock
                })
                .build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext {
                entry, instance, ..
            } = ctx.as_ref();
            unsafe { InstanceData::from_handle(instance.handle()) }
                .set_available_device_extensions(&[]);
            let destroy_swapchain = unsafe {
                entry.get_instance_proc_addr(instance.handle(), c"vkDestroySwapchainKHR".as_ptr())
            };
            assert!(destroy_swapchain.is_none());
        }

        #[test]
        #[ignore]
        fn test_should_return_fp_with_device_command_supported_by_the_layer_extension_only() {
            todo!()
        }
    }

    mod invalid_instance {
        use super::*;

        #[test]
        #[cfg_attr(miri, ignore = "https://github.com/google/vk-layer-for-rust/issues/49")]
        fn test_query_global_commands_should_still_succeed() {
            // See https://github.com/KhronosGroup/Vulkan-Loader/issues/1537.
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();

            let entry = create_entry::<TestLayer>();
            let expected = unsafe {
                entry.get_instance_proc_addr(vk::Instance::null(), c"vkCreateInstance".as_ptr())
            }
            .expect("vkCreateInstance should be a valid function pointer with NULL instance.");
            let actual = unsafe {
                entry.get_instance_proc_addr(
                    // We use transmute instead of Handle::from_raw here to avoid integer to
                    // pointer cast, and allow the miri tests with tree borrows
                    // to work with this test. See https://github.com/ash-rs/ash/issues/996 for details.
                    std::mem::transmute::<*mut u8, vk::Instance>(std::ptr::dangling_mut::<u8>()),
                    c"vkCreateInstance".as_ptr(),
                )
            }
            .expect(
                "vkCreateInstance should be a valid function pointer with an invalid instance.",
            );
            assert_eq!(expected, actual);
        }
    }

    #[test]
    #[ignore]
    fn test_layer_hooked_instance_commands_should_take_priority() {
        todo!()
    }

    #[test]
    #[ignore]
    fn test_layer_hooked_device_commands_should_take_priority() {
        todo!()
    }
}

mod create_destroy_instance {
    use super::*;
    #[test]
    #[cfg_attr(miri, ignore = "https://github.com/google/vk-layer-for-rust/issues/49")]
    fn test_should_move_layer_instance_link_forward() {
        static TEST_GLOBAL0: TestGlobal<Tag<0>> = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_global_commands()
                    .return_const(vec![LayerVulkanCommand::CreateInstance]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL0.create_context();
        static TEST_GLOBAL1: TestGlobal<Tag<1>> = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_global_commands()
                    .return_const(vec![LayerVulkanCommand::CreateInstance]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL1.create_context();

        let icd_layer_link = <()>::head_instance_link();
        let second_layer_link = <(TestLayer<Tag<1>>,)>::head_instance_link();

        fn layer_instance_link_equal(
            lhs: &Option<&VkLayerInstanceLink>,
            rhs: &Option<&VkLayerInstanceLink>,
        ) -> bool {
            match (lhs, rhs) {
                (Some(lhs), Some(rhs)) => {
                    if !std::ptr::fn_addr_eq(
                        lhs.pfnNextGetInstanceProcAddr,
                        rhs.pfnNextGetInstanceProcAddr,
                    ) {
                        return false;
                    }
                    match (
                        lhs.pfnNextGetPhysicalDeviceProcAddr,
                        rhs.pfnNextGetPhysicalDeviceProcAddr,
                    ) {
                        (Some(lhs), Some(rhs)) => std::ptr::fn_addr_eq(lhs, rhs),
                        (None, None) => true,
                        _ => false,
                    }
                }
                (None, None) => true,
                _ => false,
            }
        }
        fn match_create_instance(
            next_layer_link: Option<&VkLayerInstanceLink>,
            current_layer_link: &VkLayerInstanceLink,
        ) -> impl Fn(
            &vk::InstanceCreateInfo,
            &VkLayerInstanceLink,
            &Option<&vk::AllocationCallbacks>,
            &*mut vk::Instance,
        ) -> bool {
            let next_layer_link = next_layer_link.map(|layer_link| VkLayerInstanceLink {
                pNext: null_mut(),
                ..*layer_link
            });
            let current_layer_link = VkLayerInstanceLink {
                pNext: null_mut(),
                ..*current_layer_link
            };
            move |create_info, layer_instance_link, _, _| {
                let mut p_next_chain: VulkanBaseInStructChain =
                    unsafe { (create_info.p_next as *const vk::BaseInStructure).as_ref() }.into();
                let layer_link_head = p_next_chain.find_map(|in_struct| {
                    let in_struct = in_struct as *const vk::BaseInStructure;
                    let layer_instance_create_info = unsafe {
                        ash::match_in_struct!(match in_struct {
                            in_struct @ VkLayerInstanceCreateInfo => {
                                in_struct
                            }
                            _ => {
                                return None;
                            }
                        })
                    };
                    if layer_instance_create_info.function != VkLayerFunction::VK_LAYER_LINK_INFO {
                        return None;
                    }
                    unsafe { layer_instance_create_info.u.pLayerInfo.as_ref() }
                });
                layer_instance_link_equal(&layer_link_head, &next_layer_link.as_ref())
                    && layer_instance_link_equal(
                        &Some(layer_instance_link),
                        &Some(&current_layer_link),
                    )
            }
        }

        {
            let layer1_info = &TestLayer::<Tag<0>>::global_instance().layer_info;
            let layer2_info = &TestLayer::<Tag<1>>::global_instance().layer_info;
            {
                let mut layer1_global_hooks = layer1_info.global_hooks();
                layer1_global_hooks
                    .expect_create_instance()
                    .withf_st(match_create_instance(
                        Some(&icd_layer_link),
                        &second_layer_link,
                    ))
                    .once()
                    .return_const(LayerResult::Unhandled);
            }
            {
                let mut layer2_global_hooks = layer2_info.global_hooks();
                layer2_global_hooks
                    .expect_create_instance()
                    .withf_st(match_create_instance(None, &icd_layer_link))
                    .once()
                    .return_const(LayerResult::Unhandled);
            }

            let instance = vk::InstanceCreateInfo::builder()
                .default_instance::<(TestLayer<Tag<0>>, TestLayer<Tag<1>>)>();
            {
                layer1_info.global_hooks().checkpoint();
                layer2_info.global_hooks().checkpoint();
            }
            drop(instance);
        }
    }

    #[test]
    fn test_create_instance_with_0_api_version() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
        let _ctx = TEST_GLOBAL.create_context();
        let app_info = vk::ApplicationInfo::builder().api_version(0);
        let ctx = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .default_instance::<(TestLayer,)>();
        let InstanceContext {
            entry, instance, ..
        } = ctx.as_ref();
        let destroy_instance = unsafe {
            entry.get_instance_proc_addr(instance.handle(), c"vkDestroyInstance".as_ptr())
        };
        assert!(destroy_instance.is_some());
    }

    #[test]
    fn test_destroy_instance_with_null_handle() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
        let _ctx = TEST_GLOBAL.create_context();
        let app_info = vk::ApplicationInfo::builder().api_version(0);
        let ctx = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .default_instance::<(TestLayer,)>();
        let InstanceContext { instance, .. } = ctx.as_ref();
        let destroy_instance = instance.fp_v1_0().destroy_instance;
        unsafe { destroy_instance(vk::Instance::null(), null()) };
    }

    #[test]
    fn test_destroy_instance_will_actually_destroy_underlying_instance_info() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
        let _ctx = TEST_GLOBAL.create_context();
        let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        {
            let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
            let InstanceContext { instance, .. } = ctx.as_ref();
            let instance_data = TestLayer::<Tag<0>>::global_instance()
                .layer_info
                .get_instance_info(instance.handle())
                .unwrap();
            instance_data.with_mock_drop(|mock_drop| {
                mock_drop.expect_drop().once().return_once({
                    let dropped = Arc::clone(&dropped);
                    move || {
                        dropped.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                });
            });
            // Calling vkDestroyInstance through RAII.
        }
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }

    // Requirements of Well-Behaved Layers number LLP_LAYER_21.
    #[test]
    fn layer_shouldnt_modify_the_pinstance_pointer_when_calling_create_instance() {
        static TEST_GLOBAL1: TestGlobal<Tag<0>> = TestGlobal::<Tag<0>>::builder().build();
        static TEST_GLOBAL2: TestGlobal<Tag<1>> = TestGlobal::<Tag<1>>::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_global_commands()
                    .return_const(vec![LayerVulkanCommand::CreateInstance]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL1.create_context();
        let _ctx = TEST_GLOBAL2.create_context();
        let mut instance = MaybeUninit::<vk::Instance>::uninit();
        TestLayer::<Tag<1>>::global_instance()
            .layer_info
            .global_hooks()
            .expect_create_instance()
            .once()
            .withf_st({
                let instance_ptr = instance.as_mut_ptr();
                move |_, _, _, p_instance| *p_instance == instance_ptr
            })
            .return_const(LayerResult::Unhandled);
        let res = unsafe {
            vk::InstanceCreateInfo::builder()
                .create_instance::<(TestLayer<Tag<0>>, TestLayer<Tag<1>>)>(instance.as_mut_ptr())
        };
        assert_eq!(res, vk::Result::SUCCESS);
        let instance = unsafe { instance.assume_init() };
        let entry = <(TestLayer<Tag<0>>, TestLayer<Tag<1>>) as Layers>::create_entry();
        let instance = unsafe { ash::Instance::load(entry.static_fn(), instance) };
        unsafe { instance.destroy_instance(None) };
    }
}

mod get_device_proc_addr {
    use super::*;
    #[test]
    fn test_should_return_null_for_global_commands() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();

        let global_commands = [
            "vkEnumerateInstanceVersion",
            "vkEnumerateInstanceExtensionProperties",
            "vkEnumerateInstanceLayerProperties",
            "vkCreateInstance",
        ];
        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        for global_command in &global_commands {
            let global_command = CString::new(*global_command).unwrap();
            let proc_addr = unsafe {
                (instance_context.instance.fp_v1_0().get_device_proc_addr)(
                    device.handle(),
                    global_command.as_ptr(),
                )
            };
            assert!(
                proc_addr.is_none(),
                "{} should be null",
                global_command.to_string_lossy()
            );
        }
    }

    #[test]
    fn test_should_return_null_for_instance_commands() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_instance_commands().return_const(vec![
                    LayerVulkanCommand::GetPhysicalDeviceSparseImageFormatProperties2,
                ]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_2);
        let ctx = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();

        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        let get_physical_device_sparse_image_format_properties2 = unsafe {
            (instance_context.instance.fp_v1_0().get_device_proc_addr)(
                device.handle(),
                c"vkGetPhysicalDeviceSparseImageFormatProperties2".as_ptr(),
            )
        };
        assert!(get_physical_device_sparse_image_format_properties2.is_none());
    }

    #[test]
    fn test_should_return_null_for_available_but_not_enabled_commands() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_device_commands()
                    .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
        let InstanceContext {
            instance, entry, ..
        } = ctx.as_ref();
        unsafe { InstanceData::from_handle(instance.handle()) }
            .set_available_device_extensions(&[Extension::KHRSwapchain]);

        let destroy_swapchain_name = CString::new("vkDestroySwapchainKHR").unwrap();

        // vkGetInstanceAddr should return fp for available extensions.
        let destroy_swapchain = unsafe {
            entry.get_instance_proc_addr(instance.handle(), destroy_swapchain_name.as_ptr())
        };
        assert!(destroy_swapchain.is_some());

        let ctx = ctx.default_device().unwrap();
        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        let destroy_swapchain = unsafe {
            (instance_context
                .next_instance_dispatch
                .fp_v1_0()
                .get_device_proc_addr)(device.handle(), destroy_swapchain_name.as_ptr())
        };
        assert!(
            destroy_swapchain.is_none(),
            "The mock ICD should return NULL for vkDestroySwapchainKHR."
        );
        let destroy_swapchain = unsafe {
            (instance_context.instance.fp_v1_0().get_device_proc_addr)(
                device.handle(),
                destroy_swapchain_name.as_ptr(),
            )
        };
        assert!(destroy_swapchain.is_none());
    }

    #[test]
    fn test_should_return_fp_for_requested_core_version_device_commands() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_device_commands()
                    .return_const(vec![LayerVulkanCommand::DestroySamplerYcbcrConversion]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_1);
        let ctx = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();
        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        let destroy_sampler_ycbcr_conversion = unsafe {
            (instance_context.instance.fp_v1_0().get_device_proc_addr)(
                device.handle(),
                c"vkDestroySamplerYcbcrConversion".as_ptr(),
            )
        };
        let destroy_sampler_ycbcr_conversion: vk::PFN_vkDestroySamplerYcbcrConversion =
            unsafe { std::mem::transmute(destroy_sampler_ycbcr_conversion.unwrap()) };
        let layer_device_info = TestLayer::<Tag<0>>::global_instance()
            .layer_info
            .get_device_info(device.handle())
            .unwrap();
        layer_device_info
            .mock_hooks
            .lock()
            .unwrap()
            .expect_destroy_sampler_ycbcr_conversion()
            .with(eq(vk::SamplerYcbcrConversion::null()), always())
            .once()
            .return_const(LayerResult::Unhandled);
        unsafe {
            destroy_sampler_ycbcr_conversion(
                device.handle(),
                vk::SamplerYcbcrConversion::null(),
                null(),
            )
        };
        layer_device_info.mock_hooks.lock().unwrap().checkpoint();
    }

    #[test]
    fn test_should_return_null_for_unrequested_core_version_device_commands() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_device_commands()
                    .return_const(vec![LayerVulkanCommand::DestroySamplerYcbcrConversion]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        // For this test case, we specify a low `VkApplicationInfo::apiVersion` where
        // `vkDestroySamplerYcbcrConversion` isn't introduced.
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_0);
        let ctx = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();
        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        let destroy_sampler_ycbcr_conversion = unsafe {
            (instance_context.instance.fp_v1_0().get_device_proc_addr)(
                device.handle(),
                c"vkDestroySamplerYcbcrConversion".as_ptr(),
            )
        };
        assert!(destroy_sampler_ycbcr_conversion.is_none());
    }

    #[test]
    fn test_should_return_null_for_unsupported_core_version_device_commands() {
        // For this test case, the specified `VkApplicationInfo::apiVersion` is high enough where
        // `vkDestroySamplerYcbcrConversion` is already introduced, but the
        // `VkPhysicalDeviceProperties::apiVersion`, i.e. the underlying ICD Vulkan version, is low.
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_device_commands()
                    .return_const(vec![LayerVulkanCommand::DestroySamplerYcbcrConversion]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_1);
        let ctx = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .default_instance::<(TestLayer,)>();
        unsafe { InstanceData::from_handle(ctx.instance.handle()) }
            .set_supported_device_version(&ApiVersion::V1_0);

        let ctx = ctx.default_device().unwrap();
        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        let InstanceContext {
            instance,
            icd_entry,
            next_instance_dispatch,
            ..
        } = instance_context.as_ref();

        let destroy_sampler_ycbcr_conversion_name =
            CString::new("vkDestroySamplerYcbcrConversion").unwrap();

        // Make sure that the mock ICD will return null for unsupported device command in
        // vkGetInstanceProcAddr.
        let destroy_sampler_ycbcr_conversion = unsafe {
            icd_entry.get_instance_proc_addr(
                instance.handle(),
                destroy_sampler_ycbcr_conversion_name.as_ptr(),
            )
        };
        assert!(destroy_sampler_ycbcr_conversion.is_none());

        // Make sure that the mock ICD will return null for unsupported device command in
        // vkGetDeviceProcAddr.
        let destroy_sampler_ycbcr_conversion = unsafe {
            next_instance_dispatch.get_device_proc_addr(
                device.handle(),
                destroy_sampler_ycbcr_conversion_name.as_ptr(),
            )
        };
        assert!(destroy_sampler_ycbcr_conversion.is_none());

        let destroy_sampler_ycbcr_conversion = unsafe {
            instance.get_device_proc_addr(
                device.handle(),
                destroy_sampler_ycbcr_conversion_name.as_ptr(),
            )
        };
        assert!(destroy_sampler_ycbcr_conversion.is_none());
    }

    #[test]
    fn test_should_return_fp_for_enabled_extension_device_commands() {
        let instance_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
        let device_extensions = [vk::KhrSwapchainFn::name().as_ptr()];
        let destroy_swapchain_name = CString::new("vkDestroySwapchainKHR").unwrap();
        {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder()
                .enabled_extension_names(&instance_extensions)
                .default_instance::<(TestLayer,)>()
                .create_device_context(|create_info_builder, create_device| {
                    let create_info_builder =
                        create_info_builder.enabled_extension_names(&device_extensions);
                    create_device(create_info_builder)
                })
                .unwrap();
            let DeviceContext {
                device,
                instance_context,
                ..
            } = ctx.as_ref();
            let destroy_swapchain = unsafe {
                instance_context
                    .instance
                    .get_device_proc_addr(device.handle(), destroy_swapchain_name.as_ptr())
            };
            assert!(destroy_swapchain.is_some());
        }

        {
            static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
                .set_layer_mock_builder(|| {
                    let mut mock = MockTestLayer::default();
                    mock.expect_hooked_device_commands()
                        .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                    mock.set_default_expectations();
                    mock
                })
                .build();
            let _ctx = TEST_GLOBAL.create_context();
            let ctx = vk::InstanceCreateInfo::builder()
                .enabled_extension_names(&instance_extensions)
                .default_instance::<(TestLayer,)>()
                .create_device_context(|create_info_builder, create_device| {
                    let create_info_builder =
                        create_info_builder.enabled_extension_names(&device_extensions);
                    create_device(create_info_builder)
                })
                .unwrap();
            let DeviceContext {
                device,
                instance_context,
                ..
            } = ctx.as_ref();
            let destroy_swapchain = unsafe {
                instance_context
                    .instance
                    .get_device_proc_addr(device.handle(), destroy_swapchain_name.as_ptr())
            };
            assert!(destroy_swapchain.is_some());
        }
    }

    #[test]
    fn test_should_return_fp_for_enabled_layer_extension_device_commands() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRSwapchain,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.expect_hooked_device_commands()
                    .return_const(vec![LayerVulkanCommand::DestroySwapchainKhr]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let instance_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
        let device_extensions = [vk::KhrSwapchainFn::name().as_ptr()];
        let ctx = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(&instance_extensions)
            .default_instance::<(TestLayer,)>();
        unsafe { InstanceData::from_handle(ctx.instance.handle()) }
            .set_available_device_extensions(&[]);

        let ctx = ctx
            .create_device_context(|create_info_builder, create_device| {
                let create_info_builder =
                    create_info_builder.enabled_extension_names(&device_extensions);
                create_device(create_info_builder)
            })
            .unwrap();
        let DeviceContext {
            device,
            instance_context,
            ..
        } = ctx.as_ref();
        let InstanceContext {
            instance,
            next_instance_dispatch,
            ..
        } = instance_context.as_ref();
        let destroy_swapchain_name = CString::new("vkDestroySwapchainKHR").unwrap();
        // The mock ICD should return null for this command.
        let destroy_swapchain = unsafe {
            next_instance_dispatch
                .get_device_proc_addr(device.handle(), destroy_swapchain_name.as_ptr())
        };
        assert!(destroy_swapchain.is_none());
        let destroy_swapchain = unsafe {
            instance.get_device_proc_addr(device.handle(), destroy_swapchain_name.as_ptr())
        };
        assert!(destroy_swapchain.is_some());
    }

    #[test]
    #[ignore]
    fn test_layer_hooked_device_commands_should_take_priority() {
        todo!()
    }

    #[test]
    #[ignore]
    fn test_should_return_the_same_function_pointers_if_the_command_has_multiple_names() {
        todo!()
    }
}

mod device_commands {
    use super::*;
    #[test]
    fn test_hooked_device_proc_should_be_called() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_device_commands()
                    .return_const(vec![LayerVulkanCommand::DestroyImage]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();
        let DeviceContext { device, .. } = ctx.as_ref();
        let device_info = TestLayer::<Tag<0>>::global_instance()
            .layer_info
            .get_device_info(device.handle())
            .unwrap();
        device_info
            .mock_hooks
            .lock()
            .unwrap()
            .expect_destroy_image()
            .withf(|image, allocator| *image == vk::Image::null() && allocator.is_none())
            .once()
            .return_const(LayerResult::Unhandled);
        unsafe { device.destroy_image(vk::Image::null(), None) };
    }

    #[test]
    fn test_unhooked_device_proc_should_not_be_called() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_device_commands().return_const(vec![]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();
        let DeviceContext { device, .. } = ctx.as_ref();
        let device_info = TestLayer::<Tag<0>>::global_instance()
            .layer_info
            .get_device_info(device.handle())
            .unwrap();
        device_info
            .mock_hooks
            .lock()
            .unwrap()
            .expect_destroy_image()
            .never();
        unsafe { device.destroy_image(vk::Image::null(), None) };
    }
}

mod enumerate_device_extensions {
    use super::*;
    #[test]
    fn test_should_return_defined_extensions() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRExternalMemory,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
        let InstanceContext { instance, .. } = ctx.as_ref();
        unsafe { InstanceData::from_handle(instance.handle()) }
            .set_available_device_extensions(&[]);
        let physical_device = *unsafe { instance.enumerate_physical_devices() }
            .unwrap()
            .first()
            .unwrap();
        let layer_name = CString::new(TestLayer::<Tag<0>>::manifest().name).unwrap();
        let properties = {
            let mut property_count = MaybeUninit::uninit();
            let res = unsafe {
                (instance.fp_v1_0().enumerate_device_extension_properties)(
                    physical_device,
                    layer_name.as_ptr(),
                    property_count.as_mut_ptr(),
                    null_mut(),
                )
            };
            assert_eq!(res, vk::Result::SUCCESS);
            let mut property_count = unsafe { property_count.assume_init() };
            let mut properties = Vec::<vk::ExtensionProperties>::new();
            properties.resize_with(property_count.try_into().unwrap(), Default::default);
            let res = unsafe {
                (instance.fp_v1_0().enumerate_device_extension_properties)(
                    physical_device,
                    layer_name.as_ptr(),
                    &mut property_count,
                    properties.as_mut_ptr(),
                )
            };
            assert_eq!(res, vk::Result::SUCCESS);
            properties.into_boxed_slice()
        };
        let expected_properties = TestLayer::<Tag<0>>::manifest()
            .device_extensions
            .iter()
            .cloned()
            .map(Into::<vk::ExtensionProperties>::into)
            .collect::<Vec<_>>();
        assert_eq!(expected_properties.len(), properties.len());
        for (expected_property, property) in zip(expected_properties.iter(), properties.iter()) {
            assert_eq!(expected_property.spec_version, property.spec_version);
            let expected_name_bytes = expected_property
                .extension_name
                .iter()
                .take_while(|byte| **byte != b'\0'.try_into().unwrap())
                .collect::<Vec<_>>();
            let name_bytes = property
                .extension_name
                .iter()
                .take_while(|byte| **byte != b'\0'.try_into().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(expected_name_bytes, name_bytes);
        }
    }

    // TODO: test if the loader will add the implicit layer extension in e2e test.

    #[test]
    fn test_should_call_into_the_next_chain_if_layer_name_doesnt_match() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRExternalMemory,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
        let InstanceContext { instance, .. } = ctx.as_ref();
        let available_device_extensions = &[Extension::KHRSwapchain];
        unsafe { InstanceData::from_handle(instance.handle()) }
            .set_available_device_extensions(available_device_extensions);
        let physical_device = *unsafe { instance.enumerate_physical_devices() }
            .unwrap()
            .first()
            .unwrap();
        let properties = unsafe { instance.enumerate_device_extension_properties(physical_device) }
            .unwrap()
            .into_boxed_slice();
        let property_names = properties
            .iter()
            .map(|vk::ExtensionProperties { extension_name, .. }| {
                unsafe { CStr::from_ptr(extension_name.as_ptr()) }
                    .to_str()
                    .unwrap()
                    .try_into()
                    .unwrap()
            })
            .collect::<Vec<Extension>>();
        assert_eq!(property_names, available_device_extensions);
    }

    #[test]
    fn test_should_not_crash_with_unmatched_layer_name_and_null_physical_device() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRExternalMemory,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();

        let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
        let InstanceContext { instance, .. } = ctx.as_ref();
        let mut property_count = 0;
        let _ = unsafe {
            (instance.fp_v1_0().enumerate_device_extension_properties)(
                vk::PhysicalDevice::null(),
                null(),
                &mut property_count,
                null_mut(),
            )
        };
        let _ = unsafe {
            (instance.fp_v1_0().enumerate_device_extension_properties)(
                vk::PhysicalDevice::null(),
                c"VK_LAYER_VENDOR_test_112434".as_ptr() as *const _,
                &mut property_count,
                null_mut(),
            )
        };
    }

    #[test]
    fn test_should_return_defined_extensions_with_null_physical_device() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRExternalMemory,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();

        let ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
        let InstanceContext { instance, .. } = ctx.as_ref();
        unsafe { InstanceData::from_handle(instance.handle()) }
            .set_available_device_extensions(&[]);
        let layer_name = CString::new(TestLayer::<Tag<0>>::manifest().name).unwrap();
        let properties = {
            let mut property_count = MaybeUninit::uninit();
            let res = unsafe {
                (instance.fp_v1_0().enumerate_device_extension_properties)(
                    vk::PhysicalDevice::null(),
                    layer_name.as_ptr(),
                    property_count.as_mut_ptr(),
                    null_mut(),
                )
            };
            assert_eq!(res, vk::Result::SUCCESS);
            let mut property_count = unsafe { property_count.assume_init() };
            let mut properties = Vec::<vk::ExtensionProperties>::new();
            properties.resize_with(property_count.try_into().unwrap(), Default::default);
            let res = unsafe {
                (instance.fp_v1_0().enumerate_device_extension_properties)(
                    vk::PhysicalDevice::null(),
                    layer_name.as_ptr(),
                    &mut property_count,
                    properties.as_mut_ptr(),
                )
            };
            assert_eq!(res, vk::Result::SUCCESS);
            properties.into_boxed_slice()
        };
        let expected_properties = TestLayer::<Tag<0>>::manifest()
            .device_extensions
            .iter()
            .cloned()
            .map(Into::<vk::ExtensionProperties>::into)
            .collect::<Vec<_>>();
        assert_eq!(expected_properties.len(), properties.len());
        for (expected_property, property) in zip(expected_properties.iter(), properties.iter()) {
            assert_eq!(expected_property.spec_version, property.spec_version);
            let expected_name_bytes = expected_property
                .extension_name
                .iter()
                .take_while(|byte| **byte != b'\0'.try_into().unwrap())
                .collect::<Vec<_>>();
            let name_bytes = property
                .extension_name
                .iter()
                .take_while(|byte| **byte != b'\0'.try_into().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(expected_name_bytes, name_bytes);
        }
    }
}

mod create_destroy_device {
    use super::*;
    #[test]
    fn test_should_always_pass_through_non_layer_extensions() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRGetPhysicalDeviceProperties2,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let instance_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
        let instance_ctx = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(&instance_extensions)
            .default_instance::<(TestLayer,)>();
        unsafe { InstanceData::from_handle(instance_ctx.instance.handle()) }
            .set_available_device_extensions(&[Extension::KHRSwapchain]);
        let enabled_device_extensions = [
            // Available extension.
            vk::KhrSwapchainFn::name().as_ptr(),
            vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
        ];
        let device_ctx = instance_ctx
            .clone()
            .create_device_context(|create_info, create_device| {
                create_device(create_info.enabled_extension_names(&enabled_device_extensions))
            })
            .unwrap();
        let enabled_icd_device_extensions =
            unsafe { DeviceData::from_handle(device_ctx.device.handle()) }
                .enabled_extensions
                .iter()
                .cloned()
                .collect::<Vec<_>>();
        assert_eq!(enabled_icd_device_extensions, vec![Extension::KHRSwapchain]);

        let enabled_device_extensions = [
            // Unavailable extension.
            vk::KhrExternalMemoryCapabilitiesFn::name().as_ptr(),
            vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr(),
        ];
        let device_create_res = instance_ctx
            .create_device_context(|create_info, create_device| {
                create_device(create_info.enabled_extension_names(&enabled_device_extensions))
            })
            .map(drop);
        assert_eq!(
            device_create_res.unwrap_err(),
            vk::Result::ERROR_EXTENSION_NOT_PRESENT
        );
    }

    #[test]
    fn test_should_always_filter_out_layer_extensions() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRGetPhysicalDeviceProperties2,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();

        let instance_ctx = vk::InstanceCreateInfo::builder().default_instance::<(TestLayer,)>();
        let enabled_device_extensions = [vk::KhrGetPhysicalDeviceProperties2Fn::name().as_ptr()];

        unsafe { InstanceData::from_handle(instance_ctx.instance.handle()) }
            .set_available_device_extensions(&[Extension::KHRGetPhysicalDeviceProperties2]);
        let device_ctx = instance_ctx
            .clone()
            .create_device_context(|create_info, create_device| {
                create_device(create_info.enabled_extension_names(&enabled_device_extensions))
            })
            .unwrap();
        let icd_enabled_extensions = unsafe { DeviceData::from_handle(device_ctx.device.handle()) }
            .enabled_extensions
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(icd_enabled_extensions, vec![]);

        unsafe { InstanceData::from_handle(instance_ctx.instance.handle()) }
            .set_available_device_extensions(&[]);
        let device_ctx = instance_ctx
            .create_device_context(|create_info, create_device| {
                create_device(create_info.enabled_extension_names(&enabled_device_extensions))
            })
            .unwrap();
        let icd_enabled_extensions = unsafe { DeviceData::from_handle(device_ctx.device.handle()) }
            .enabled_extensions
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(icd_enabled_extensions, vec![]);
    }

    #[test]
    fn test_should_always_pass_through_unknown_extensions() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRGetPhysicalDeviceProperties2,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();

        let device_extensions = [CString::new("VK_UNKNOWN_unknown_extension").unwrap()];
        let device_extensions = device_extensions
            .iter()
            .map(|extension| extension.as_ptr())
            .collect::<Vec<_>>();
        let device_create_res = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer,)>()
            .create_device_context(|create_info, create_device| {
                create_device(create_info.enabled_extension_names(&device_extensions))
            })
            .map(drop);
        assert_eq!(
            device_create_res.unwrap_err(),
            vk::Result::ERROR_EXTENSION_NOT_PRESENT
        );
    }

    #[test]
    #[cfg_attr(miri, ignore = "https://github.com/google/vk-layer-for-rust/issues/49")]
    fn test_should_move_layer_device_link_forward() {
        static TEST_GLOBAL0: TestGlobal<Tag<0>> = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_instance_commands()
                    .return_const(vec![LayerVulkanCommand::CreateDevice]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL0.create_context();
        static TEST_GLOBAL1: TestGlobal<Tag<1>> = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_instance_commands()
                    .return_const(vec![LayerVulkanCommand::CreateDevice]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL1.create_context();

        let instance_ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer<Tag<0>>, TestLayer<Tag<1>>)>();
        let InstanceContext { instance, .. } = instance_ctx.as_ref();

        let icd_layer_link = <()>::head_device_link(instance.handle());
        let second_layer_link = <(TestLayer<Tag<1>>,)>::head_device_link(instance.handle());

        fn layer_device_link_equal(
            lhs: &Option<&VkLayerDeviceLink>,
            rhs: &Option<&VkLayerDeviceLink>,
        ) -> bool {
            match (lhs, rhs) {
                (Some(lhs), Some(rhs)) => {
                    std::ptr::fn_addr_eq(
                        lhs.pfnNextGetInstanceProcAddr,
                        rhs.pfnNextGetInstanceProcAddr,
                    ) && std::ptr::fn_addr_eq(
                        lhs.pfnNextGetDeviceProcAddr,
                        rhs.pfnNextGetDeviceProcAddr,
                    )
                }
                (None, None) => true,
                _ => false,
            }
        }

        fn match_create_device(
            next_layer_link: Option<&VkLayerDeviceLink>,
            current_layer_link: &VkLayerDeviceLink,
        ) -> impl Fn(
            &vk::PhysicalDevice,
            &vk::DeviceCreateInfo,
            &VkLayerDeviceLink,
            &Option<&vk::AllocationCallbacks>,
            &MaybeUninit<vk::Device>,
        ) -> bool {
            let next_layer_link = next_layer_link.map(|layer_link| VkLayerDeviceLink {
                pNext: null_mut(),
                ..*layer_link
            });
            let current_layer_link = VkLayerDeviceLink {
                pNext: null_mut(),
                ..*current_layer_link
            };
            move |_, create_info, layer_instance_link, _, _| {
                let mut p_next_chain: VulkanBaseInStructChain =
                    unsafe { (create_info.p_next as *const vk::BaseInStructure).as_ref() }.into();
                let layer_link_head = p_next_chain.find_map(|in_struct| {
                    let in_struct = in_struct as *const vk::BaseInStructure;
                    let layer_device_create_info = unsafe {
                        ash::match_in_struct!(match in_struct {
                            in_struct @ VkLayerDeviceCreateInfo => {
                                in_struct
                            }
                            _ => {
                                return None;
                            }
                        })
                    };
                    if layer_device_create_info.function != VkLayerFunction::VK_LAYER_LINK_INFO {
                        return None;
                    }
                    let layer_link_head = unsafe { layer_device_create_info.u.pLayerInfo };
                    unsafe { layer_link_head.as_ref() }
                });
                layer_device_link_equal(&layer_link_head, &next_layer_link.as_ref())
                    && layer_device_link_equal(
                        &Some(layer_instance_link),
                        &Some(&current_layer_link),
                    )
            }
        }
        let layer1_instance_info = &TestLayer::<Tag<0>>::global_instance()
            .layer_info
            .get_instance_info(instance.handle())
            .unwrap();
        let layer2_instance_info = &TestLayer::<Tag<1>>::global_instance()
            .layer_info
            .get_instance_info(instance.handle())
            .unwrap();
        layer1_instance_info
            .hooks()
            .expect_create_device()
            .withf_st(match_create_device(
                Some(&icd_layer_link),
                &second_layer_link,
            ))
            .once()
            .return_const(LayerResult::Unhandled);
        layer2_instance_info
            .hooks()
            .expect_create_device()
            .withf_st(match_create_device(None, &icd_layer_link))
            .once()
            .return_const(LayerResult::Unhandled);
        let device_ctx = instance_ctx.default_device().unwrap();
        layer1_instance_info.hooks().checkpoint();
        layer2_instance_info.hooks().checkpoint();
        drop(device_ctx);
    }

    #[test]
    fn test_should_receive_the_original_extension_list() {
        let instance_extensions = [vk::KhrSurfaceFn::name().as_ptr()];
        let expected_device_extensions: Vec<&'static CStr> = vec![vk::KhrSwapchainFn::name()];
        let device_extensions = expected_device_extensions
            .iter()
            .map(|s| s.as_ptr())
            .collect::<Vec<_>>();

        static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_manifest().return_const({
                    let mut layer_manifest = LayerManifest::test_default();
                    layer_manifest.device_extensions = &[ExtensionProperties {
                        name: Extension::KHRSwapchain,
                        spec_version: 1,
                    }];
                    layer_manifest
                });
                mock.expect_hooked_instance_commands()
                    .return_const(vec![LayerVulkanCommand::CreateDevice]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL.create_context();
        let instance_ctx = vk::InstanceCreateInfo::builder()
            .enabled_extension_names(&instance_extensions)
            .default_instance::<(TestLayer,)>();
        let instance_info = TestLayer::<Tag<0>>::global_instance()
            .layer_info
            .get_instance_info(instance_ctx.instance.handle())
            .unwrap();
        instance_info
            .hooks()
            .expect_create_device()
            .withf(move |_, create_info, _, _, _| {
                let extensions = unsafe {
                    std::slice::from_raw_parts(
                        create_info.pp_enabled_extension_names,
                        create_info.enabled_extension_count.try_into().unwrap(),
                    )
                };
                let extensions = extensions
                    .iter()
                    .map(|extension| unsafe { CStr::from_ptr(*extension) })
                    .collect::<Vec<_>>();
                extensions == expected_device_extensions
            })
            .once()
            .return_const(LayerResult::Unhandled);
        let device_ctx = instance_ctx
            .create_device_context(|create_info, create_device| {
                create_device(create_info.enabled_extension_names(&device_extensions))
            })
            .unwrap();
        instance_info.hooks().checkpoint();
        drop(device_ctx);
    }

    #[test]
    fn test_destroy_device_with_null_handle() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
        let _ctx = TEST_GLOBAL.create_context();
        let ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer,)>()
            .default_device()
            .unwrap();
        unsafe { (ctx.device.fp_v1_0().destroy_device)(vk::Device::null(), null()) };
    }

    #[test]
    fn test_destroy_device_will_actually_destroy_underlying_device_info() {
        static TEST_GLOBAL: TestGlobal = TestGlobal::builder().build();
        let _ctx = TEST_GLOBAL.create_context();
        let dropped: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
        {
            let ctx = vk::InstanceCreateInfo::builder()
                .default_instance::<(TestLayer,)>()
                .default_device()
                .unwrap();
            let DeviceContext { device, .. } = ctx.as_ref();
            let device_data = TestLayer::<Tag<0>>::global_instance()
                .layer_info
                .get_device_info(device.handle())
                .unwrap();
            device_data.with_mock_drop(|mock_drop| {
                mock_drop.expect_drop().once().return_once({
                    let dropped = Arc::clone(&dropped);
                    move || {
                        dropped.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                });
            });
            // Calling vkDestroyDevice through RAII.
        }
        assert!(dropped.load(std::sync::atomic::Ordering::SeqCst));
    }

    #[test]
    fn layer_shouldnt_modify_the_pdevice_pointer_when_calling_create_device() {
        static TEST_GLOBAL1: TestGlobal<Tag<0>> = TestGlobal::<Tag<0>>::builder().build();
        static TEST_GLOBAL2: TestGlobal<Tag<1>> = TestGlobal::<Tag<1>>::builder()
            .set_layer_mock_builder(|| {
                let mut mock = MockTestLayer::default();
                mock.expect_hooked_instance_commands()
                    .return_const(vec![LayerVulkanCommand::CreateDevice]);
                mock.set_default_expectations();
                mock
            })
            .build();
        let _ctx = TEST_GLOBAL1.create_context();
        let _ctx = TEST_GLOBAL2.create_context();
        let instance_ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(TestLayer<Tag<0>>, TestLayer<Tag<1>>)>();
        let mut device = MaybeUninit::<vk::Device>::uninit();
        TestLayer::<Tag<1>>::global_instance()
            .layer_info
            .get_instance_info(instance_ctx.instance.handle())
            .unwrap()
            .hooks()
            .expect_create_device()
            .once()
            .withf_st({
                let device_ptr = device.as_ptr();
                move |_, _, _, _, p_device| p_device.as_ptr() == device_ptr
            })
            .return_const(LayerResult::Unhandled);
        let res = unsafe {
            instance_ctx.create_device(
                |create_info_builder, create_device| {
                    create_device(create_info_builder);
                },
                device.as_mut_ptr(),
            )
        };
        let device = unsafe { res.assume_init_on_success(device) }.unwrap();
        let device = unsafe { ash::Device::load(instance_ctx.instance.fp_v1_0(), device) };
        unsafe { device.destroy_device(None) };
    }
}

#[test]
fn enumerate_instance_layer_properties_should_return_correct_properties() {
    static LAYER_MANIFEST: Lazy<LayerManifest> = Lazy::new(|| {
        let mut layer_manifest = LayerManifest::test_default();
        layer_manifest.name = "VK_LAYER_UNKNOWN_TESTLAYER";
        layer_manifest.description = "A test description.";
        layer_manifest.spec_version = vk::API_VERSION_1_3;
        layer_manifest.implementation_version = 42;
        layer_manifest
    });

    static TEST_GLOBAL: TestGlobal = TestGlobal::builder()
        .set_layer_mock_builder(|| {
            let mut mock = MockTestLayer::default();
            mock.expect_manifest().return_const(LAYER_MANIFEST.clone());
            mock.set_default_expectations();
            mock
        })
        .build();
    let _ctx = TEST_GLOBAL.create_context();
    let mut property_count = MaybeUninit::<u32>::uninit();
    assert_eq!(
        unsafe {
            Global::<TestLayer>::enumerate_instance_layer_properties(
                property_count.as_mut_ptr(),
                null_mut(),
            )
        },
        vk::Result::SUCCESS
    );
    let mut property_count = unsafe { property_count.assume_init() };
    assert_eq!(property_count, 1);
    let mut property = MaybeUninit::<vk::LayerProperties>::uninit();
    assert_eq!(
        unsafe {
            Global::<TestLayer>::enumerate_instance_layer_properties(
                &mut property_count,
                property.as_mut_ptr(),
            )
        },
        vk::Result::SUCCESS
    );
    assert_eq!(property_count, 1);
    let property = unsafe { property.assume_init() };

    let property_layer_name = unsafe { CStr::from_ptr(property.layer_name.as_ptr()) }
        .to_str()
        .unwrap();
    let property_description = unsafe { CStr::from_ptr(property.description.as_ptr()) }
        .to_str()
        .unwrap();
    assert_eq!(property_layer_name, LAYER_MANIFEST.name);
    assert_eq!(property.spec_version, LAYER_MANIFEST.spec_version);
    assert_eq!(
        property.implementation_version,
        LAYER_MANIFEST.implementation_version
    );
    assert_eq!(property_description, LAYER_MANIFEST.description);
}

#[test]
fn global_should_be_sync() {
    #[derive(Default)]
    struct Test<T: Sync>(PhantomData<T>);
    let _: Test<Global<TestLayer>> = Default::default();
}

#[test]
fn global_should_be_trivially_destructible() {
    #[derive(Default)]
    struct MyLayer(StubGlobalHooks);

    static GLOBAL: LazyLock<Mutex<Option<Global<MyLayer>>>> = LazyLock::new(Default::default);

    impl Layer for MyLayer {
        type GlobalHooksInfo = StubGlobalHooks;
        type InstanceInfo = StubInstanceInfo;
        type DeviceInfo = StubDeviceInfo;
        type InstanceInfoContainer = StubInstanceInfo;
        type DeviceInfoContainer = StubDeviceInfo;

        fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
            MutexGuard::map(GLOBAL.lock(), |global| global.as_mut().unwrap())
        }

        fn manifest() -> LayerManifest {
            LayerManifest::test_default()
        }

        fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
            &self.0
        }

        fn create_instance_info(
            &self,
            _: &vk::InstanceCreateInfo,
            _: Option<&vk::AllocationCallbacks>,
            _: Arc<ash::Instance>,
            _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
        ) -> Self::InstanceInfoContainer {
            Default::default()
        }

        fn create_device_info(
            &self,
            _: vk::PhysicalDevice,
            _: &vk::DeviceCreateInfo,
            _: Option<&vk::AllocationCallbacks>,
            _: Arc<ash::Device>,
            _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
        ) -> Self::DeviceInfoContainer {
            Default::default()
        }
    }

    // We rely on Miri to catch the memory leak here to make sure Global is trivially destructible.
    {
        let global = Global::<MyLayer>::default();
        std::mem::forget(global);
    }
    {
        let global = Global::<MyLayer>::default();
        assert!(GLOBAL.lock().replace(global).is_none());

        let instance_ctx = vk::InstanceCreateInfo::builder().default_instance::<(MyLayer,)>();
        drop(instance_ctx);

        let global = GLOBAL.lock().take();
        std::mem::forget(global);
    }
    {
        let global = Global::<MyLayer>::default();
        assert!(GLOBAL.lock().replace(global).is_none());

        let device_ctx = vk::InstanceCreateInfo::builder()
            .default_instance::<(MyLayer,)>()
            .default_device()
            .unwrap();
        drop(device_ctx);

        let global = GLOBAL.lock().take();
        std::mem::forget(global);
    }
}

#[test]
#[ignore]
fn global_should_only_call_layer_ctor_once_even_if_multiple_instance_device_created() {
    todo!()
}

#[test]
#[ignore]
fn global_should_only_call_layer_ctor_once_even_if_multiple_calls_to_global_commands() {
    todo!()
}

#[test]
#[ignore]
fn global_enumerate_device_extension_properties_should_never_call_into_the_next_chain() {
    todo!("Use NULL physical device, and 2 layers with non-null physical device")
}
