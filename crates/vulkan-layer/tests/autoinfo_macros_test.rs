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

use ash::vk;
use mockall::automock;
use once_cell::sync::Lazy;
use std::{marker::PhantomData, sync::Arc};
use vulkan_layer::{
    auto_deviceinfo_impl, auto_globalhooksinfo_impl, auto_instanceinfo_impl,
    test_utils::LayerManifestExt, DeviceHooks, DeviceInfo, Global, GlobalHooks, GlobalHooksInfo,
    InstanceHooks, InstanceInfo, Layer, LayerManifest, LayerResult, LayerVulkanCommand,
    StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
};

#[automock]
pub trait GlobalInstanceProvider<T: Layer> {
    fn instance() -> &'static Global<T>;
}

#[derive(Default)]
struct TestLayer<T, U, V>
where
    T: GlobalHooksInfo + Default + 'static,
    U: InstanceInfo + Default + 'static,
    V: DeviceInfo + Default + 'static,
{
    global_hooks: T,
    _marker: PhantomData<fn(T, U, V)>,
}

impl<T, U, V> Layer for TestLayer<T, U, V>
where
    T: GlobalHooksInfo + Default + 'static,
    U: InstanceInfo + Default + 'static,
    V: DeviceInfo + Default + 'static,
{
    type GlobalHooksInfo = T;
    type InstanceInfo = U;
    type DeviceInfo = V;
    type InstanceInfoContainer = U;
    type DeviceInfoContainer = V;

    fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
        MockGlobalInstanceProvider::<Self>::instance()
    }

    fn manifest() -> LayerManifest {
        LayerManifest::test_default()
    }

    fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
        &self.global_hooks
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

    fn create_instance_info(
        &self,
        _: &vk::InstanceCreateInfo,
        _: Option<&vk::AllocationCallbacks>,
        _: Arc<ash::Instance>,
        _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ) -> Self::InstanceInfoContainer {
        Default::default()
    }
}

#[test]
fn test_auto_globalhooksinfo_should_intercept_hooked_proc() {
    type MockLayer = TestLayer<TestGlobalHooks, StubInstanceInfo, StubDeviceInfo>;
    static GLOBAL: Lazy<Global<MockLayer>> = Lazy::new(Default::default);
    let ctx = MockGlobalInstanceProvider::instance_context();
    ctx.expect().return_const(&*GLOBAL);
    #[derive(Default)]
    struct TestGlobalHooks;
    #[auto_globalhooksinfo_impl]
    impl GlobalHooks for TestGlobalHooks {
        fn create_instance(
            &self,
            _p_create_info: &vk::InstanceCreateInfo,
            _layer_instance_link: &vulkan_layer::VkLayerInstanceLink,
            _p_allocator: Option<&vk::AllocationCallbacks>,
            _: *mut vk::Instance,
        ) -> LayerResult<ash::prelude::VkResult<()>> {
            unimplemented!()
        }
    }
    let hooked_commands = <MockLayer as Layer>::GlobalHooksInfo::hooked_commands().to_vec();
    assert_eq!(hooked_commands, &[LayerVulkanCommand::CreateInstance]);
}

#[test]
fn test_auto_instanceinfo_should_intercept_hooked_proc() {
    type MockLayer = TestLayer<StubGlobalHooks, TestInstanceInfo, StubDeviceInfo>;
    static GLOBAL: Lazy<Global<MockLayer>> = Lazy::new(Default::default);
    let ctx = MockGlobalInstanceProvider::instance_context();
    ctx.expect().return_const(&*GLOBAL);
    #[derive(Default)]
    struct TestInstanceInfo;
    #[auto_instanceinfo_impl]
    impl InstanceHooks for TestInstanceInfo {
        fn destroy_surface_khr(
            &self,
            _: vk::SurfaceKHR,
            _: Option<&vk::AllocationCallbacks>,
        ) -> LayerResult<()> {
            unimplemented!()
        }
    }
    let hooked_commands = MockLayer::global_instance()
        .layer_info
        .hooked_instance_commands(&Default::default())
        .collect::<Vec<_>>();
    assert_eq!(hooked_commands, &[LayerVulkanCommand::DestroySurfaceKhr]);
}

#[test]
fn test_auto_deviceinfo_should_intercept_hooked_proc() {
    type MockLayer = TestLayer<StubGlobalHooks, StubInstanceInfo, TestDeviceInfo>;
    static GLOBAL: Lazy<Global<MockLayer>> = Lazy::new(Default::default);
    let ctx = MockGlobalInstanceProvider::instance_context();
    ctx.expect().return_const(&*GLOBAL);
    #[derive(Default)]
    struct TestDeviceInfo;
    #[auto_deviceinfo_impl]
    impl DeviceHooks for TestDeviceInfo {
        fn destroy_image(
            &self,
            _image: vk::Image,
            _p_allocator: Option<&vk::AllocationCallbacks>,
        ) -> LayerResult<()> {
            unimplemented!()
        }
    }
    let hooked_commands = MockLayer::global_instance()
        .layer_info
        .hooked_device_commands(&Default::default(), None)
        .collect::<Vec<_>>();
    assert_eq!(hooked_commands, &[LayerVulkanCommand::DestroyImage]);
}
