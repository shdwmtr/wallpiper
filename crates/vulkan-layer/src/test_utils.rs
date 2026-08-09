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

//! Utilities used in the integration tests of this crate.
//!
//! The Vulkan layer essentially provides a bunch of function pointers to the loader. To mock
//! multiple layers in the same process, it is required to use different types so different
//! function pointers will be generated. To avoid extremely long compile time, we shouldn't create
//! a new layer type for every test case. Instead we run test cases in different processes, and
//! share the same layer type as possible. If a test case needs multiple layers at the same time,
//! multiple layer types should be used.
//!
//! Follow the following steps to write a unit test:
//! 1. Use [`TestGlobal::builder`] to construct the static resources needed to mock one layer. Mocks
//!    to static methods can be set with different builder functions.
//! 2. Call [`TestGlobal::create_context`] to set up the scope for expectations.
//! 3. Use [`TestLayer`] as a layer implementation.

use crate::{
    DeviceInfo, Global, GlobalHooksInfo, InstanceInfo, Layer, LayerManifest, LayerVulkanCommand,
};
use ash::vk;
use mockall::mock;
use once_cell::sync::Lazy;
use std::{
    any::Any,
    borrow::Borrow,
    collections::HashMap,
    marker::PhantomData,
    ops::Deref,
    sync::{Arc, Mutex, MutexGuard, Weak},
};

pub use crate::bindings::vk_layer::{
    VkLayerDeviceCreateInfo, VkLayerDeviceLink, VkLayerFunction, VkLayerInstanceCreateInfo,
};

mod device_hooks_mock;
mod global_hooks_mock;
mod instance_hooks_mock;

use device_hooks_mock::MockDeviceHooks;
use global_hooks_mock::MockGlobalHooks;
use instance_hooks_mock::MockInstanceHooks;

type Deleter<T> = Box<dyn FnOnce(&mut T) + Send + Sync>;

/// A wrapper for T but with a user defined deleter. 'Del' stands for 'deleter'.
///
/// The deleter will be called on drop. Used to mock the [`Drop`] trait.
pub struct Del<T> {
    data: T,
    deleter: Option<Deleter<T>>,
}

impl<T> Del<T> {
    /// Constructs a new [`Del<T>`] with a custom deleter.
    pub fn new(data: T, deleter: impl FnOnce(&mut T) + Send + Sync + 'static) -> Self {
        Del {
            data,
            deleter: Some(Box::new(deleter)),
        }
    }
}

impl<T> Deref for Del<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> Drop for Del<T> {
    fn drop(&mut self) {
        (self.deleter.take().unwrap())(&mut self.data);
    }
}

impl<T> AsRef<T> for Del<T> {
    fn as_ref(&self) -> &T {
        self.deref()
    }
}

/// A thread-safe reference-counting pointer, but with a custom deleter.
pub struct ArcDel<T>(pub Arc<Del<T>>);

impl<T> ArcDel<T> {
    /// Constructs a new [`ArcDel<T>`] with a custom deleter.
    pub fn new(data: T, deleter: impl FnOnce(&mut T) + Send + Sync + 'static) -> Self {
        ArcDel(Arc::new(Del::new(data, deleter)))
    }
}

impl<T> Borrow<T> for ArcDel<T> {
    fn borrow(&self) -> &T {
        &self.0.data
    }
}

impl<T: Default> Default for ArcDel<T> {
    fn default() -> Self {
        Self(Arc::new(Del::new(Default::default(), |_| {})))
    }
}

impl<T> From<Del<T>> for ArcDel<T> {
    fn from(value: Del<T>) -> Self {
        Self(Arc::new(value))
    }
}

impl<T> Deref for ArcDel<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

impl<T> AsRef<T> for ArcDel<T> {
    fn as_ref(&self) -> &T {
        self.0.as_ref()
    }
}

impl<T> Clone for ArcDel<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

mock! {
    pub Drop {}
    impl Drop for Drop {
        fn drop(&mut self);
    }
}

/// A set of interfaces that the integration tests are interested to mock.
pub trait TestLayerMock: 'static {
    /// The associated type that implements the [`Layer`] trait. Should be a specialized
    /// [`TestLayer`].
    type TestLayerType: Layer;

    /// Used to mock [`Layer::global_instance`].
    fn instance() -> &'static Global<Self::TestLayerType>;

    /// Used to obtain the singleton of this object.
    ///
    /// This interface is used to obtain the mock object in the static methods like
    /// [`DeviceInfo::hooked_commands`]. The mock object can't be part of [`Global`], because we
    /// want to allow the mocked interfaces to be called during or before
    /// the initialization of [`Global`].
    fn mock() -> &'static Self;

    /// Used to mock [`Layer::manifest`].
    fn manifest(&self) -> LayerManifest;

    /// Used to mock [`GlobalHooksInfo::hooked_commands`].
    fn hooked_global_commands(&self) -> &[LayerVulkanCommand];

    /// Used to mock [`InstanceInfo::hooked_commands`].
    fn hooked_instance_commands(&self) -> &[LayerVulkanCommand];

    /// Used to mock [`DeviceInfo::hooked_commands`].
    fn hooked_device_commands(&self) -> &[LayerVulkanCommand];
}

/// A mock struct that implements the [`GlobalHooksInfo`] trait.
///
/// The `T` type parameter is a [`TestLayerMock`] type that provides the mock of
/// [`GlobalHooksInfo::hooked_commands`].
#[derive(Default)]
pub struct MockGlobalHooksInfo<T: TestLayerMock> {
    /// The mock of the [`GlobalHooks`][crate::GlobalHooks] trait.
    pub mock_hooks: Mutex<MockGlobalHooks>,
    _marker: PhantomData<fn(T)>,
}

impl<T: TestLayerMock> GlobalHooksInfo for MockGlobalHooksInfo<T> {
    type HooksType = MockGlobalHooks;
    type HooksRefType<'a> = MutexGuard<'a, MockGlobalHooks>;
    fn hooked_commands() -> &'static [LayerVulkanCommand] {
        T::mock().hooked_global_commands()
    }

    fn hooks(&self) -> Self::HooksRefType<'_> {
        self.mock_hooks.lock().unwrap()
    }
}

/// A mock struct that implements the [`InstanceInfo`] trait.
///
/// The `T` type parameter is a [`TestLayerMock`] type that provides the mock of
/// [`InstanceInfo::hooked_commands`].
#[derive(Default)]
pub struct MockInstanceInfo<T: TestLayerMock> {
    /// The mock of the [`InstanceHooks`][crate::InstanceHooks].
    pub mock_hooks: Mutex<MockInstanceHooks>,
    mock_drop: Mutex<Option<MockDrop>>,
    _marker: PhantomData<fn(T)>,
}

impl<T: TestLayerMock> MockInstanceInfo<T> {
    /// Mock the drop behavior.
    ///
    /// The expectations can be set through the `f` argument. If this method is never called, the
    /// struct will be dropped as if the drop is not mocked, i.e. won't check how drop is called.
    pub fn with_mock_drop(&self, f: impl FnOnce(&mut MockDrop)) {
        let mut mock_drop = self.mock_drop.lock().unwrap();
        let mock_drop = mock_drop.get_or_insert_with(Default::default);
        f(mock_drop);
    }
}

impl<T: TestLayerMock> InstanceInfo for MockInstanceInfo<T> {
    type HooksType = MockInstanceHooks;
    type HooksRefType<'a> = MutexGuard<'a, MockInstanceHooks>;
    fn hooked_commands() -> &'static [LayerVulkanCommand] {
        T::mock().hooked_instance_commands()
    }

    fn hooks(&self) -> Self::HooksRefType<'_> {
        self.mock_hooks.lock().unwrap()
    }
}

/// A mock struct that implements the [`DeviceInfo`] trait.
///
/// The `T` type parameter is a [`TestLayerMock`] type that provides the mock of
/// [`DeviceInfo::hooked_commands`].
#[derive(Default)]
pub struct MockDeviceInfo<T: TestLayerMock> {
    /// The mock of the [`DeviceHooks`][crate::DeviceHooks].
    pub mock_hooks: Mutex<MockDeviceHooks>,
    mock_drop: Mutex<Option<MockDrop>>,
    _marker: PhantomData<fn(T)>,
}

impl<T: TestLayerMock> MockDeviceInfo<T> {
    /// Mock the drop behavior.
    ///
    /// The expectations can be set through the `f` argument. If this method is never called, the
    /// struct will be dropped as if the drop is not mocked, i.e. won't check how drop is called.
    pub fn with_mock_drop(&self, f: impl FnOnce(&mut MockDrop)) {
        let mut mock_drop = self.mock_drop.lock().unwrap();
        let mock_drop = mock_drop.get_or_insert_with(Default::default);
        f(mock_drop);
    }
}

impl<T: TestLayerMock> DeviceInfo for MockDeviceInfo<T> {
    type HooksType = MockDeviceHooks;
    type HooksRefType<'a> = MutexGuard<'a, MockDeviceHooks>;

    fn hooked_commands() -> &'static [LayerVulkanCommand] {
        T::mock().hooked_device_commands()
    }

    fn hooks(&self) -> Self::HooksRefType<'_> {
        self.mock_hooks.lock().unwrap()
    }
}

/// Test layer tags to distinguish different [`TestLayer`]. Different `I` will result in different
/// types.
pub struct Tag<const I: usize>;

/// A trait used to include all possible [`Tag<I>`].
pub trait TestLayerTag: Sync + Send + 'static {}

impl<const I: usize> TestLayerTag for Tag<I> {}

/// The mock for the [`Layer`] trait.
///
/// Different `T` will result in different types, default to [`Tag<0>`]. [`TestGlobal<T>`] contains
/// the static resources(e.g. [`Global<TestLayer<T>>`]) related to this mock layer.
pub struct TestLayer<T: TestLayerTag = Tag<0>> {
    global_hooks_info: MockGlobalHooksInfo<MockTestLayer<T>>,
    instances: Mutex<HashMap<vk::Instance, Weak<Del<<Self as Layer>::InstanceInfo>>>>,
    devices: Mutex<HashMap<vk::Device, Weak<Del<<Self as Layer>::DeviceInfo>>>>,
    _marker: PhantomData<fn(T)>,
}

impl<T: TestLayerTag> TestLayer<T> {
    /// Get the [`InstanceInfo`] mock from a `VkInstance`.
    pub fn get_instance_info(
        &self,
        instance: vk::Instance,
    ) -> Option<Arc<impl Deref<Target = <Self as Layer>::InstanceInfo>>> {
        self.instances
            .lock()
            .unwrap()
            .get(&instance)
            .and_then(Weak::upgrade)
    }

    /// Get the [`DeviceInfo`] mock from a `VkDevice`.
    pub fn get_device_info(
        &self,
        device: vk::Device,
    ) -> Option<Arc<impl Deref<Target = <Self as Layer>::DeviceInfo>>> {
        self.devices
            .lock()
            .unwrap()
            .get(&device)
            .and_then(Weak::upgrade)
    }
}

impl<T: TestLayerTag> Default for TestLayer<T> {
    fn default() -> Self {
        Self {
            global_hooks_info: Default::default(),
            instances: Default::default(),
            devices: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<T: TestLayerTag> Layer for TestLayer<T> {
    type GlobalHooksInfo = MockGlobalHooksInfo<MockTestLayer<T>>;
    type InstanceInfo = MockInstanceInfo<MockTestLayer<T>>;
    type DeviceInfo = MockDeviceInfo<MockTestLayer<T>>;
    type InstanceInfoContainer = ArcDel<Self::InstanceInfo>;
    type DeviceInfoContainer = ArcDel<Self::DeviceInfo>;

    fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
        MockTestLayer::<T>::instance()
    }

    fn manifest() -> LayerManifest {
        MockTestLayer::<T>::mock().manifest()
    }

    fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
        &self.global_hooks_info
    }

    fn create_device_info(
        &self,
        _physical_device: vk::PhysicalDevice,
        _create_info: &vk::DeviceCreateInfo,
        _allocator: Option<&vk::AllocationCallbacks>,
        device: Arc<ash::Device>,
        _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ) -> ArcDel<Self::DeviceInfo> {
        let device_handle = device.handle();
        let device_info = ArcDel::new(Default::default(), move |_| {
            let layer = &Self::global_instance().layer_info;
            layer.devices.lock().unwrap().remove(&device_handle);
        });
        self.devices
            .lock()
            .unwrap()
            .insert(device.handle(), Arc::downgrade(&device_info.0));
        device_info
    }

    fn create_instance_info(
        &self,
        _create_info: &vk::InstanceCreateInfo,
        _allocator: Option<&vk::AllocationCallbacks>,
        instance: Arc<ash::Instance>,
        _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ) -> ArcDel<Self::InstanceInfo> {
        let instance_handle = instance.handle();
        let instance_info = ArcDel::new(Default::default(), move |_| {
            let layer = &Self::global_instance().layer_info;
            layer.instances.lock().unwrap().remove(&instance_handle);
        });
        self.instances
            .lock()
            .unwrap()
            .insert(instance.handle(), Arc::downgrade(&instance_info.0));
        instance_info
    }
}

mock! {
    pub TestLayer<T: TestLayerTag = Tag<0>> {}

    impl<T: TestLayerTag> TestLayerMock for TestLayer<T> {
        type TestLayerType = TestLayer<T>;

        fn instance() -> &'static Global<TestLayer<T>>;
        fn mock() -> &'static Self;

        fn manifest(&self) -> LayerManifest;
        fn hooked_global_commands(&self) -> &[LayerVulkanCommand];
        fn hooked_instance_commands(&self) -> &[LayerVulkanCommand];
        fn hooked_device_commands(&self) -> &[LayerVulkanCommand];
    }
}

impl<T: TestLayerTag> MockTestLayer<T> {
    /// Set the default behavior of the [`MockTestLayer`]: intercept no commands and a valid
    /// [`LayerManifest`].
    pub fn set_default_expectations(&mut self) {
        self.expect_manifest()
            .return_const(LayerManifest::test_default());
        self.expect_hooked_global_commands().return_const(vec![]);
        self.expect_hooked_instance_commands().return_const(vec![]);
        self.expect_hooked_device_commands().return_const(vec![]);
    }
}

/// The container of the static resources related to [`TestLayer<T>`].
///
/// Use the [`TestGlobal::builder`] to construct one.
pub struct TestGlobal<T: TestLayerTag = Tag<0>> {
    layer_mock: Lazy<MockTestLayer<T>>,
    layer_global: Lazy<Global<TestLayer<T>>>,
}

/// The builder for [`TestGlobal`].
pub struct TestGlobalBuilder<T: TestLayerTag = Tag<0>> {
    layer_mock_builder: fn() -> MockTestLayer<T>,
    layer_global_builder: fn() -> Global<TestLayer<T>>,
}

impl<T: TestLayerTag> TestGlobalBuilder<T> {
    /// Specify how [`MockTestLayer`] should be created.
    ///
    /// Can be used to customize the behavior of pre-initialized methods, like
    /// [`InstanceInfo::hooked_commands`].
    ///
    /// Pre-initialized methods are almost always called, so it is usually needed to call
    /// [`MockTestLayer::set_default_expectations`] right before return to provide a meaningful
    /// default behavior.
    pub const fn set_layer_mock_builder(
        self,
        layer_mock_builder: fn() -> MockTestLayer<T>,
    ) -> Self {
        Self {
            layer_mock_builder,
            ..self
        }
    }

    /// Specify how [`Global<TestLayer<T>>`] should be created.
    ///
    /// Can be used to customize the behavior and expectations of layer behaviors after [`Global`]
    /// is initialized, especially the global commands, e.g.
    /// [`GlobalHooks::create_instance`][crate::GlobalHooks::create_instance].
    pub const fn set_layer_global_builder(
        self,
        layer_global_builder: fn() -> Global<TestLayer<T>>,
    ) -> Self {
        Self {
            layer_global_builder,
            ..self
        }
    }

    /// Construct the [`TestGlobal<T>`].
    ///
    /// This is a const function, and can be directly used to initialize a static variable.
    pub const fn build(&self) -> TestGlobal<T> {
        TestGlobal {
            layer_mock: Lazy::new(self.layer_mock_builder),
            layer_global: Lazy::new(self.layer_global_builder),
        }
    }
}

impl<T: TestLayerTag> TestGlobal<T> {
    /// Create the builder for initialization.
    pub const fn builder() -> TestGlobalBuilder<T> {
        TestGlobalBuilder {
            layer_mock_builder: || {
                let mut mock = MockTestLayer::<T>::default();
                mock.set_default_expectations();
                mock
            },
            layer_global_builder: Default::default,
        }
    }

    /// Create a context for expectations for the static object.
    pub fn create_context(&'static self) -> Box<dyn Any> {
        let layer_mock_ctx = MockTestLayer::mock_context();
        layer_mock_ctx.expect().return_const(&*self.layer_mock);
        let layer_global_ctx = MockTestLayer::instance_context();
        layer_global_ctx.expect().return_const(&*self.layer_global);
        Box::new((layer_mock_ctx, layer_global_ctx))
    }
}

/// A trait to provide an extra meaningful constructor for [`LayerManifest`].
pub trait LayerManifestExt {
    /// Create a [`LayerManifest`] with reasonable fields.
    fn test_default() -> Self;
}

impl LayerManifestExt for LayerManifest {
    fn test_default() -> Self {
        Self {
            name: "VK_LAYER_GOOGLE_test",
            spec_version: vk::API_VERSION_1_1,
            ..Default::default()
        }
    }
}
