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

#![warn(missing_docs)]
#![cfg_attr(all(doc, RUSTC_NIGHTLY), feature(doc_auto_cfg))]

//! This crate provides a convenient framework to develop
//! [Vulkan layers](https://github.com/KhronosGroup/Vulkan-Loader/blob/main/docs/LoaderLayerInterface.md)
//! in Rust on top of the [ash](https://crates.io/crates/ash) crate. If you are not familiar how to
//! write a Vulkan layer, [this C++ tutorial](https://renderdoc.org/vulkan-layer-guide.html) by
//! Baldur Karlsson is a good reference to start with.
//!
//! Key features provided by this crate includes:
//!
//! * Support the look-up map fashion of implementing a Vulkan layer.
//!
//!   The look-up maps for `VkDevice` and `VkInstance` are handled by this crate.
//!
//! * Implement `vkGet*ProcAddr` automatically.
//!
//!   This is a non-trivial work to comply with the spec, because of extensions and the
//!   required/supported Vulkan API level. See the
//!   [spec](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html)
//!   of `vkGetInstanceProc` for details.
//!
//! * Handle dispatch tables, `vkCreateInstance` and `vkCreateDevice`.
//!
//!   This mainly includes using the `vkGet*ProcAddr` function pointer correctly, and advancing the
//!   `VkLayer*CreateInfo::u::pLayerInfo` link list. One common mistake in layer implementation
//!   in `vkCreateDevice` is to call `getInstanceProcAddr(VK_NULL_HANDLE, "vkCreateDevice")` to
//!   obtain the function pointer to `vkCreateDevice`. According to
//!   [the spec](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#_description),
//!   `getInstanceProcAddr` should return `NULL`. This framework helps you avoid bugs like this.
//!
//! Note that object wrapping is not supported by this crate, and we don't plan such support,
//! because object wrapping requires us to intercept ALL Vulkan commands related to one handle type,
//! so it can't handle unknown commands. In addition, object wrapping is more complicated because it
//! is required to call loader callback on every dispatchable handle creation and destruction.
//!
//! # Overview
//!
//! The primary types in this crate are the [`Layer`] trait and the [`Global`] struct. The user
//! implements the [`Layer`] trait for a type `T`, and the [`Global<T>`] will include all necessary
//! functions needed to export from this dynamic link library. With the help of the
//! [`declare_introspection_queries`] macro, the user can export those functions in oneline.
//!
//! ```
//! use ash::{self, vk};
//! use once_cell::sync::Lazy;
//! use std::sync::Arc;
//! use vulkan_layer::{
//!     declare_introspection_queries, Global, Layer, LayerManifest, StubDeviceInfo,
//!     StubGlobalHooks, StubInstanceInfo,
//! };
//!
//! // Define the layer type.
//! #[derive(Default)]
//! struct MyLayer(StubGlobalHooks);
//!
//! // Implement the Layer trait.
//! impl Layer for MyLayer {
//!     type GlobalHooksInfo = StubGlobalHooks;
//!     type InstanceInfo = StubInstanceInfo;
//!     type DeviceInfo = StubDeviceInfo;
//!     type InstanceInfoContainer = StubInstanceInfo;
//!     type DeviceInfoContainer = StubDeviceInfo;
//!
//!     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
//!         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
//!         &*GLOBAL
//!     }
//!
//!     fn manifest() -> LayerManifest {
//!         Default::default()
//!     }
//!
//!     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
//!         &self.0
//!     }
//!
//!     fn create_instance_info(
//!         &self,
//!         _: &vk::InstanceCreateInfo,
//!         _: Option<&vk::AllocationCallbacks>,
//!         _: Arc<ash::Instance>,
//!         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
//!     ) -> Self::InstanceInfoContainer {
//!         Default::default()
//!     }
//!
//!     fn create_device_info(
//!         &self,
//!         _: vk::PhysicalDevice,
//!         _: &vk::DeviceCreateInfo,
//!         _: Option<&vk::AllocationCallbacks>,
//!         _: Arc<ash::Device>,
//!         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
//!     ) -> Self::DeviceInfoContainer {
//!         Default::default()
//!     }
//! }
//!
//! // Define the global type from the layer type.
//! type MyGlobal = Global<MyLayer>;
//! // Export C functions.
//! declare_introspection_queries!(MyGlobal);
//! ```
//! The user can provide their own types that implement different traits to intercept different
//! commands and custom the layer behavior:
//! * [`GlobalHooks`], [`InstanceHooks`], [`DeviceHooks`]: Implements the interception to different
//!   Vulkan commands based on the dispatch types.
//! * [`GlobalHooksInfo`], [`InstanceInfo`], [`DeviceInfo`]: Container of [`GlobalHooks`],
//!   [`InstanceHooks`], [`DeviceHooks`]. Provide the list of commands to intercept for the
//!   framework. Can be automatically implemented through the [`auto_deviceinfo_impl`],
//!   [`auto_globalhooksinfo_impl`], [`auto_instanceinfo_impl`] macros from the [`GlobalHooks`],
//!   [`InstanceHooks`], [`DeviceHooks`] implementation respectively.
//! * [`Layer`]: Provide the metadata of a layer through [`LayerManifest`], e.g. the name, version,
//!   exported extensions of the layer. Provide the storage for the [`Global`] object. Container of
//!   [`GlobalHooksInfo`] type. The factory type of the [`InstanceInfo`], [`DeviceInfo`] types.
//!
//! # Usage
//!
//! You can check a live example in `examples/hello-world`.
//!
//! First, create a Rust lib crate.
//! ```bash
//! $ mkdir vulkan-layer-rust-example
//! $ cd vulkan-layer-rust-example
//! $ cargo init --lib
//!      Created library package
//! ```
//!
//! Second, modify the crate type to `cdylib` and set the panic behavior to abort in `Cargo.toml`.
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//!
//! [profile.dev]
//! panic = "abort"
//!
//! [profile.release]
//! panic = "abort"
//! ```
//! We need to set panic to abort to avoid unwinding from Rust to the caller(most likely C/C++),
//! because unwinding into other language from Rust is
//! [undefined behavior](https://doc.rust-lang.org/beta/nomicon/unwinding.html).
//!
//! If you want to try the layer on Android, also modify the crate name to
//! `VkLayer_vendor_rust_example`, because Android requires that the layer shared object library
//! follow a specific name convention.
//! ```toml
//! [lib]
//! name = "VkLayer_vendor_rust_example"
//! ```
//!
//! Third, set up the dependency in `Cargo.toml`. In my case, I checkout the project repository in
//! the same directory where the `vulkan-layer-rust-example` folder lives.
//! ```toml
//! [dependencies]
//! vulkan-layer = { path = "../vk-layer-for-rust/vulkan-layer" }
//! ```
//! Other dependencies.
//! ```bash
//! cargo add ash once_cell
//! ```
//!
//! Fourth, implement the layer trait in `lib.rs`.
//! ```
//! use ash::{self, vk};
//! use once_cell::sync::Lazy;
//! use std::sync::Arc;
//! use vulkan_layer::{
//!     declare_introspection_queries, Global, Layer, LayerManifest, StubDeviceInfo,
//!     StubGlobalHooks, StubInstanceInfo,
//! };
//!
//! #[derive(Default)]
//! struct MyLayer(StubGlobalHooks);
//!
//! impl Layer for MyLayer {
//!     type GlobalHooksInfo = StubGlobalHooks;
//!     type InstanceInfo = StubInstanceInfo;
//!     type DeviceInfo = StubDeviceInfo;
//!     type InstanceInfoContainer = StubInstanceInfo;
//!     type DeviceInfoContainer = StubDeviceInfo;
//!
//!     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
//!         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
//!         &*GLOBAL
//!     }
//!
//!     fn manifest() -> LayerManifest {
//!         let mut manifest = LayerManifest::default();
//!         manifest.name = "VK_LAYER_VENDOR_rust_example";
//!         manifest.spec_version = vk::API_VERSION_1_1;
//!         manifest.description = "Rust test layer";
//!         manifest
//!     }
//!
//!     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
//!         &self.0
//!     }
//!
//!     fn create_instance_info(
//!         &self,
//!         _: &vk::InstanceCreateInfo,
//!         _: Option<&vk::AllocationCallbacks>,
//!         _: Arc<ash::Instance>,
//!         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
//!     ) -> Self::InstanceInfoContainer {
//!         Default::default()
//!     }
//!
//!     fn create_device_info(
//!         &self,
//!         _: vk::PhysicalDevice,
//!         _: &vk::DeviceCreateInfo,
//!         _: Option<&vk::AllocationCallbacks>,
//!         _: Arc<ash::Device>,
//!         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
//!     ) -> Self::DeviceInfoContainer {
//!         println!("Hello from the Rust Vulkan layer!");
//!         Default::default()
//!     }
//! }
//! ```
//!
//! Fifth, export functions through the [`declare_introspection_queries`] macro
//! ```
//! # use ash::{self, vk};
//! # use once_cell::sync::Lazy;
//! # use std::sync::Arc;
//! # use vulkan_layer::{
//! #     declare_introspection_queries, Global, Layer, LayerManifest, StubDeviceInfo,
//! #     StubGlobalHooks, StubInstanceInfo,
//! # };
//! #
//! # #[derive(Default)]
//! # struct MyLayer(StubGlobalHooks);
//! #
//! # impl Layer for MyLayer {
//! #     type GlobalHooksInfo = StubGlobalHooks;
//! #     type InstanceInfo = StubInstanceInfo;
//! #     type DeviceInfo = StubDeviceInfo;
//! #     type InstanceInfoContainer = StubInstanceInfo;
//! #     type DeviceInfoContainer = StubDeviceInfo;
//! #
//! #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
//! #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
//! #         &*GLOBAL
//! #     }
//! #
//! #     fn manifest() -> LayerManifest {
//! #         let mut manifest = LayerManifest::default();
//! #         manifest.name = "VK_LAYER_VENDOR_rust_example";
//! #         manifest.spec_version = vk::API_VERSION_1_1;
//! #         manifest.description = "Rust test layer";
//! #         manifest
//! #     }
//! #
//! #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
//! #         &self.0
//! #     }
//! #
//! #     fn create_instance_info(
//! #         &self,
//! #         _: &vk::InstanceCreateInfo,
//! #         _: Option<&vk::AllocationCallbacks>,
//! #         _: Arc<ash::Instance>,
//! #         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
//! #     ) -> Self::InstanceInfoContainer {
//! #         Default::default()
//! #     }
//! #
//! #     fn create_device_info(
//! #         &self,
//! #         _: vk::PhysicalDevice,
//! #         _: &vk::DeviceCreateInfo,
//! #         _: Option<&vk::AllocationCallbacks>,
//! #         _: Arc<ash::Device>,
//! #         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
//! #     ) -> Self::DeviceInfoContainer {
//! #         println!("Hello from the Rust Vulkan layer!");
//! #         Default::default()
//! #     }
//! # }
//! declare_introspection_queries!(Global::<MyLayer>);
//! ```
//!
//! Sixth, build and check the exported symbol of the build artifacts, and we should see Vulkan
//! introspection APIs are exported.
//!
//! On Windows, use
//! [`dumpbin`](https://learn.microsoft.com/en-us/cpp/build/reference/dumpbin-reference?view=msvc-170)
//! ```cmd
//! $ dumpbin /exports .\target\debug\VkLayer_vendor_rust_example.dll
//! Microsoft (R) COFF/PE Dumper Version 14.36.32537.0
//! Copyright (C) Microsoft Corporation.  All rights reserved.
//!
//!
//! Dump of file .\target\debug\VkLayer_vendor_rust_example.dll
//!
//! File Type: DLL
//!
//!   Section contains the following exports for VkLayer_vendor_rust_example.dll
//!
//!     00000000 characteristics
//!     FFFFFFFF time date stamp
//!         0.00 version
//!            1 ordinal base
//!            6 number of functions
//!            6 number of names
//!
//!     ordinal hint RVA      name
//!
//!           1    0 000F1840 vkEnumerateDeviceExtensionProperties = vkEnumerateDeviceExtensionProperties
//!           2    1 000F1820 vkEnumerateDeviceLayerProperties = vkEnumerateDeviceLayerProperties
//!           3    2 000F1800 vkEnumerateInstanceExtensionProperties = vkEnumerateInstanceExtensionProperties
//!           4    3 000F17E0 vkEnumerateInstanceLayerProperties = vkEnumerateInstanceLayerProperties
//!           5    4 000F1890 vkGetDeviceProcAddr = vkGetDeviceProcAddr
//!           6    5 000F1870 vkGetInstanceProcAddr = vkGetInstanceProcAddr
//!
//!   Summary
//!
//!         1000 .data
//!        11000 .pdata
//!        37000 .rdata
//!         2000 .reloc
//!       171000 .text
//! ```
//! On Linux, use [`objdump`](https://linux.die.net/man/1/objdump).
//! ```bat
//! $ objdump -TC target/debug/libVkLayer_vendor_rust_example.so
//!
//! target/debug/libVkLayer_vendor_rust_example.so:     file format elf64-x86-64
//!
//! DYNAMIC SYMBOL TABLE:
//! (omit some irrelevant symbols...)
//! 00000000000fad20 g    DF .text  0000000000000022  Base        vkEnumerateDeviceExtensionProperties
//! 00000000000face0 g    DF .text  000000000000001c  Base        vkEnumerateInstanceExtensionProperties
//! 00000000000fad50 g    DF .text  0000000000000018  Base        vkGetInstanceProcAddr
//! 00000000000facc0 g    DF .text  0000000000000018  Base        vkEnumerateInstanceLayerProperties
//! 00000000000fad00 g    DF .text  000000000000001c  Base        vkEnumerateDeviceLayerProperties
//! 00000000000fad70 g    DF .text  0000000000000018  Base        vkGetDeviceProcAddr
//! ```
//!
//! Seventh, create the layer manifest file named `rust_example_layer.json` right beside the built
//! artifact. If targeting Android, the json manifest file is not needed.
//!
//! For Windows,
//! ```json
//! {
//!     "file_format_version" : "1.2.1",
//!     "layer": {
//!         "name": "VK_LAYER_VENDOR_rust_example",
//!         "type": "INSTANCE",
//!         "library_path": ".\\VkLayer_vendor_rust_example.dll",
//!         "library_arch" : "64",
//!         "api_version" : "1.1.0",
//!         "implementation_version" : "0",
//!         "description" : "Rust test layer"
//!     }
//! }
//! ```
//!
//! For Linux,
//! ```json
//! {
//!     "file_format_version" : "1.2.1",
//!     "layer": {
//!         "name": "VK_LAYER_VENDOR_rust_example",
//!         "type": "INSTANCE",
//!         "library_path": "./libVkLayer_vendor_rust_example.so",
//!         "library_arch" : "64",
//!         "api_version" : "1.1.0",
//!         "implementation_version" : "0",
//!         "description" : "Rust test layer"
//!     }
//! }
//! ```
//! This json file will define an explicit layer named `VK_LAYER_VENDOR_rust_example`.
//!
//! Eighth, use [`VkConfig`](https://github.com/LunarG/VulkanTools/blob/main/vkconfig/README.md)
//! (i.e. Vulkan Configurator) to force enable this explicit layer, and launch the vkcube
//! application through `VkConfig`. In the log view, we should see the
//! `"Hello from the Rust Vulkan layer!"` log line. For Android, follow
//! [this instruction](https://developer.android.com/ndk/guides/graphics/validation-layer) to enable
//! the layer. [`println!`] won't write to logcat, so one needs to change the layer implementation
//! to write to the logcat.
//!
//! # Global initialization and clean up
//!
//! See [the][Layer#initialization] [document][Layer#global-clean-up] of [`Layer`] for details.
//!
//! # Extensions
//!
//! TODO
//!
//! # Synchronization
//!
//! TODO
//!
//! # Safety
//!
//! TODO
//!
//! # API stability
//!
//! TODO

use ash::vk::{self, Handle};
use bytemuck::cast_slice;
use log::{error, warn};
use std::{
    borrow::Borrow,
    collections::{BTreeMap, BTreeSet},
    ffi::{c_char, c_void, CStr, CString},
    ptr::{null, null_mut, NonNull},
    sync::{Arc, Mutex},
};
extern crate self as vulkan_layer;

mod bindings;
mod global_simple_intercept;
mod layer_trait;
mod lazy_collection;
#[cfg(any(feature = "_test", test))]
pub mod test_utils;

#[cfg(feature = "unstable")]
pub mod unstable_api;
#[cfg(not(feature = "unstable"))]
mod unstable_api;
mod vk_utils;

use bindings::vk_layer::{VkLayerDeviceCreateInfo, VkLayerFunction, VkLayerInstanceCreateInfo};
pub use bindings::vk_layer::{VkLayerDeviceLink, VkLayerInstanceLink};
pub use global_simple_intercept::Extension;
use global_simple_intercept::{DeviceDispatchTable, InstanceDispatchTable, VulkanCommand};
pub use layer_trait::{
    DeviceHooks, DeviceInfo, ExtensionProperties, GlobalHooks, GlobalHooksInfo, InstanceHooks,
    InstanceInfo, Layer, LayerManifest, LayerResult, VulkanCommand as LayerVulkanCommand,
};
use unstable_api::{ApiVersion, IsCommandEnabled, LazyCollection};
pub use vk_utils::{fill_vk_out_array, VulkanBaseInStructChain, VulkanBaseOutStructChain};
use vk_utils::{ptr_as_uninit_mut, slice_from_raw_parts, slice_to_owned_strings};
pub use vulkan_layer_macros::{
    auto_deviceinfo_impl, auto_globalhooksinfo_impl, auto_instanceinfo_impl,
    declare_introspection_queries,
};

trait DispatchableObject: vk::Handle + Copy {
    type DispatchKey: From<usize>;

    fn get_dispatch_key(&self) -> Self::DispatchKey {
        assert_eq!(
            std::mem::size_of::<Self>(),
            std::mem::size_of::<*const *const c_void>()
        );
        // Safe, because all dispatchable objects can be cast to void **. See details at
        // https://github.com/KhronosGroup/Vulkan-Loader/blob/35b005a5792f6e4c2931d62a37324923f1a71c79/docs/LoaderDriverInterface.md#driver-dispatchable-object-creation.
        let key = unsafe {
            // We use transmute instead of Handle::as_raw here to avoid integer to pointer cast, and
            // allow the miri tests with tree borrows to work with this test. See
            // https://github.com/ash-rs/ash/issues/996 for details.
            let dispatch_table_ptr = std::mem::transmute_copy::<Self, *const *const c_void>(self);
            std::ptr::read(dispatch_table_ptr)
        };
        (key as usize).into()
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct InstanceDispatchKey(usize);

impl From<usize> for InstanceDispatchKey {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl DispatchableObject for vk::Instance {
    type DispatchKey = InstanceDispatchKey;
}

impl DispatchableObject for vk::PhysicalDevice {
    type DispatchKey = InstanceDispatchKey;
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct DeviceDispatchKey(usize);

impl From<usize> for DeviceDispatchKey {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl DispatchableObject for vk::Device {
    type DispatchKey = DeviceDispatchKey;
}

impl DispatchableObject for vk::CommandBuffer {
    type DispatchKey = DeviceDispatchKey;
}

impl DispatchableObject for vk::Queue {
    type DispatchKey = DeviceDispatchKey;
}

struct InstanceInfoWrapper<T: Layer> {
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    dispatch_table: InstanceDispatchTable,
    api_version: ApiVersion,
    enabled_extensions: BTreeSet<Extension>,
    // instance_commands and device_commands are recalculated on every vkCreateInstance, so that
    // the layer can decide which commands to intercept dynamically.
    instance_commands: Box<[VulkanCommand]>,
    device_commands: Box<[VulkanCommand]>,
    is_create_device_hooked: bool,
    customized_info: T::InstanceInfoContainer,
}

struct PhysicalDeviceInfoWrapper {
    owner_instance: vk::Instance,
    properties: vk::PhysicalDeviceProperties,
}

struct DeviceInfoWrapper<T: Layer> {
    dispatch_table: DeviceDispatchTable,
    get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    api_version: ApiVersion,
    enabled_extensions: BTreeSet<Extension>,
    // device_commands are recalculated on every vkCreateDevice, so that the layer can decide
    // which commands to intercept dynamically.
    device_commands: Box<[VulkanCommand]>,
    customized_info: T::DeviceInfoContainer,
}

/// A struct that implements all necessarily functions for a layer given a type that implements
/// [`Layer`].
///
/// The layer implementation should use [`Global::default`] to construct a [`Global`] object.
/// When all `VkInstance`s and `VkDevice`s are destroyed, the drop of [`Global`] is no-op if the
/// drop of `T` is no-op under such circumstances. See [the][Layer#initialization]
/// [document][Layer#global-clean-up] of [`Layer`] for details on global initialization and
/// clean-up.
///
/// This is supposed to be a global singleton for a layer to store all necessary data to implement a
/// Vulkan layer including dispatch tables, maps between the Vulkan objects and their wrappers, etc.
pub struct Global<T: Layer> {
    instance_map: Mutex<LazyCollection<BTreeMap<InstanceDispatchKey, Arc<InstanceInfoWrapper<T>>>>>,
    physical_device_map:
        Mutex<LazyCollection<BTreeMap<vk::PhysicalDevice, Arc<PhysicalDeviceInfoWrapper>>>>,
    device_map: Mutex<LazyCollection<BTreeMap<DeviceDispatchKey, Arc<DeviceInfoWrapper<T>>>>>,
    /// Access to the underlying `T`.
    // layer_info can't be lazily constructed when the first VkInstance is created, because we want
    // to guarantee that T::default is only called once during the lifetime of Global, so that the
    // client can put global state in T, and can perform global initialization(e.g. the global
    // logger) in T::default. If the client needs Global to be trivially destructible, the client
    // must guarantee that T is also trivially destructible.
    pub layer_info: T,
    get_instance_addr_proc_hooked: bool,
}

impl<T: Layer> Global<T> {
    fn instance() -> impl std::ops::Deref<Target = Self> + 'static {
        T::global_instance()
    }

    fn get_instance_info(
        &self,
        instance: impl DispatchableObject<DispatchKey = InstanceDispatchKey>,
    ) -> Option<Arc<InstanceInfoWrapper<T>>> {
        self.instance_map
            .lock()
            .unwrap()
            .get()
            .get(&instance.get_dispatch_key())
            .map(Arc::clone)
    }

    fn create_physical_device_infos<U: Borrow<vk::PhysicalDevice>>(
        &self,
        instance: vk::Instance,
        physical_devices: impl IntoIterator<Item = U>,
    ) {
        for physical_device in physical_devices {
            let mut physical_device_map = self.physical_device_map.lock().unwrap();
            let mut physical_device_map = physical_device_map.get_mut_or_default();
            if let Some(physical_device_info) = physical_device_map.get(physical_device.borrow()) {
                assert_eq!(physical_device_info.owner_instance, instance);
                continue;
            }
            let instance_info = self.get_instance_info(instance).unwrap_or_else(|| {
                panic!("Unknown VkInstance handle: {:#018x}", instance.as_raw())
            });
            let properties = unsafe {
                instance_info
                    .dispatch_table
                    .core
                    .get_physical_device_properties(*physical_device.borrow())
            };
            physical_device_map.insert(
                *physical_device.borrow(),
                Arc::new(PhysicalDeviceInfoWrapper {
                    owner_instance: instance,
                    properties,
                }),
            );
        }
    }

    fn get_device_info(
        &self,
        device: impl DispatchableObject<DispatchKey = DeviceDispatchKey>,
    ) -> Option<Arc<DeviceInfoWrapper<T>>> {
        self.device_map
            .lock()
            .unwrap()
            .get()
            .get(&device.get_dispatch_key())
            .map(Arc::clone)
    }

    fn get_physical_info(
        &self,
        physical_device: vk::PhysicalDevice,
    ) -> Option<Arc<PhysicalDeviceInfoWrapper>> {
        self.physical_device_map
            .lock()
            .unwrap()
            .get()
            .get(&physical_device)
            .map(Arc::clone)
    }

    extern "system" fn create_instance(
        create_info: *const vk::InstanceCreateInfo,
        allocator: *const vk::AllocationCallbacks,
        p_instance: *mut vk::Instance,
    ) -> vk::Result {
        let p_next_chain = unsafe { create_info.as_ref() }
            .map(|create_info| create_info.p_next as *mut vk::BaseOutStructure)
            .unwrap_or(null_mut());
        let mut p_next_chain: VulkanBaseOutStructChain = unsafe { p_next_chain.as_mut() }.into();
        let layer_create_info = p_next_chain.find_map(|out_struct| {
            let out_struct = out_struct as *mut vk::BaseOutStructure;
            let layer_create_info = unsafe {
                ash::match_out_struct!(match out_struct {
                    out_struct @ VkLayerInstanceCreateInfo => {
                        out_struct
                    }
                    _ => {
                        return None;
                    }
                })
            };
            if layer_create_info.function == VkLayerFunction::VK_LAYER_LINK_INFO {
                Some(layer_create_info)
            } else {
                None
            }
        });
        let layer_create_info = match layer_create_info {
            Some(layer_create_info) => layer_create_info,
            None => {
                error!("failed to find the VkLayerInstanceCreateInfo struct in the chain.");
                return vk::Result::ERROR_INITIALIZATION_FAILED;
            }
        };
        let layer_info = unsafe { layer_create_info.u.pLayerInfo.as_ref() }.unwrap();
        layer_create_info.u.pLayerInfo = layer_info.pNext;

        let get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr =
            layer_info.pfnNextGetInstanceProcAddr;

        let global = Self::instance();
        let hooked =
            T::GlobalHooksInfo::hooked_commands().contains(&LayerVulkanCommand::CreateInstance);
        let layer_result = if hooked {
            global.layer_info.global_hooks().create_instance(
                unsafe { create_info.as_ref() }.unwrap(),
                layer_info,
                unsafe { allocator.as_ref() },
                p_instance,
            )
        } else {
            LayerResult::Unhandled
        };

        match layer_result {
            LayerResult::Handled(Ok(())) => {}
            LayerResult::Handled(Err(e)) => return e,
            LayerResult::Unhandled => {
                let get_proc_addr = |name: &CStr| -> *const c_void {
                    let fp = unsafe { get_instance_proc_addr(vk::Instance::null(), name.as_ptr()) };
                    if let Some(fp) = fp {
                        return fp as *const _;
                    }
                    std::ptr::null()
                };
                let entry = vk::EntryFnV1_0::load(get_proc_addr);

                let ret: vk::Result =
                    unsafe { (entry.create_instance)(create_info, allocator, p_instance) };
                if !matches!(ret, vk::Result::SUCCESS) {
                    return ret;
                }
            }
        };

        let instance = *unsafe { p_instance.as_ref() }.unwrap();
        let ash_instance = unsafe {
            ash::Instance::load(
                &ash::vk::StaticFn {
                    get_instance_proc_addr,
                },
                instance,
            )
        };

        let create_info = unsafe { create_info.as_ref() }.unwrap();
        let enabled_extensions = unsafe {
            slice_from_raw_parts(
                create_info.pp_enabled_extension_names,
                create_info.enabled_extension_count,
            )
        };
        let enabled_extensions = unsafe { slice_to_owned_strings(enabled_extensions) }
            .filter_map(|extension_name| match extension_name.as_str().try_into() {
                Ok(extension) => Some(extension),
                Err(e) => {
                    warn!(
                        "Failed to recognize the extension {}: {}",
                        extension_name, e
                    );
                    None
                }
            })
            .collect();
        let api_version = unsafe { create_info.p_application_info.as_ref() }
            .map(|app_info| app_info.api_version)
            .unwrap_or(0);
        let api_version = if api_version == 0 {
            ApiVersion { major: 1, minor: 0 }
        } else {
            api_version.into()
        };
        let ash_instance = Arc::new(ash_instance);
        let customized_info = global.layer_info.create_instance_info(
            create_info,
            unsafe { allocator.as_ref() },
            Arc::clone(&ash_instance),
            get_instance_proc_addr,
        );
        let is_create_device_hooked = global
            .layer_info
            .hooked_instance_commands(customized_info.borrow())
            .any(|command| command == LayerVulkanCommand::CreateDevice);
        let instance_commands = global.create_instance_commands(customized_info.borrow());
        let device_commands = global.create_device_commands(customized_info.borrow(), None);
        {
            let key = instance.get_dispatch_key();
            let mut instance_map = global.instance_map.lock().unwrap();
            let mut instance_map = instance_map.get_mut_or_default();
            if instance_map.contains_key(&key) {
                error!(
                    "duplicate instances: instance {:?} already exists",
                    instance
                );
                return vk::Result::ERROR_INITIALIZATION_FAILED;
            }
            instance_map.insert(
                key,
                Arc::new(InstanceInfoWrapper {
                    get_instance_proc_addr,
                    dispatch_table: InstanceDispatchTable::load(
                        get_instance_proc_addr,
                        ash_instance,
                    ),
                    api_version,
                    enabled_extensions,
                    instance_commands,
                    device_commands,
                    is_create_device_hooked,
                    customized_info,
                }),
            );
        }
        vk::Result::SUCCESS
    }

    extern "system" fn destroy_instance(
        instance: vk::Instance,
        allocator: *const vk::AllocationCallbacks,
    ) {
        if instance == vk::Instance::null() {
            return;
        }
        let dispatch_key = instance.get_dispatch_key();
        let global = Self::instance();
        let instance_info = global
            .instance_map
            .lock()
            .unwrap()
            .get_mut_or_default()
            .remove(&dispatch_key);
        let instance_info = instance_info.unwrap();
        let instance_info = match Arc::try_unwrap(instance_info) {
            Ok(instance_info) => instance_info,
            Err(_) => panic!(
                "This should be the sole instance dispatch table reference for instance {:?}.",
                instance
            ),
        };
        global
            .physical_device_map
            .lock()
            .unwrap()
            .get_mut_or_default()
            .retain(|_, physical_device_info| physical_device_info.owner_instance != instance);
        unsafe {
            (instance_info.dispatch_table.core.fp_v1_0().destroy_instance)(instance, allocator)
        };
    }

    extern "system" fn enumerate_physical_devices(
        instance: vk::Instance,
        p_physical_device_count: *mut u32,
        p_physical_devices: *mut vk::PhysicalDevice,
    ) -> vk::Result {
        let global = Self::instance();
        let ash_instance = &global
            .get_instance_info(instance)
            .expect("Must be a valid VkInstance")
            .dispatch_table
            .core;
        let res = unsafe {
            (ash_instance.fp_v1_0().enumerate_physical_devices)(
                instance,
                p_physical_device_count,
                p_physical_devices,
            )
        };
        if (res == vk::Result::SUCCESS || res == vk::Result::INCOMPLETE)
            && !p_physical_devices.is_null()
        {
            let physical_device_count = unsafe { p_physical_device_count.as_ref() }.unwrap();
            let physical_devices =
                unsafe { slice_from_raw_parts(p_physical_devices, *physical_device_count) };
            global.create_physical_device_infos(instance, physical_devices);
        }
        res
    }

    extern "system" fn enumerate_physical_device_groups(
        instance: vk::Instance,
        p_physical_device_group_count: *mut u32,
        p_physical_device_group_properties: *mut vk::PhysicalDeviceGroupProperties,
    ) -> vk::Result {
        let global = Self::instance();
        let ash_instance = &global
            .get_instance_info(instance)
            .expect("Must be a valid VkInstance")
            .dispatch_table
            .core;
        let res = unsafe {
            (ash_instance.fp_v1_1().enumerate_physical_device_groups)(
                instance,
                p_physical_device_group_count,
                p_physical_device_group_properties,
            )
        };
        if (res == vk::Result::SUCCESS || res == vk::Result::INCOMPLETE)
            && !p_physical_device_group_count.is_null()
        {
            let physical_device_group_count =
                *unsafe { p_physical_device_group_count.as_ref() }.unwrap();
            let physical_device_groups = unsafe {
                slice_from_raw_parts(
                    p_physical_device_group_properties,
                    physical_device_group_count,
                )
            };
            let physical_devices =
                physical_device_groups
                    .iter()
                    .flat_map(|physical_device_group| {
                        &physical_device_group.physical_devices[..physical_device_group
                            .physical_device_count
                            .try_into()
                            .unwrap()]
                    });
            global.create_physical_device_infos(instance, physical_devices);
        }
        res
    }

    extern "system" fn create_device(
        physical_device: vk::PhysicalDevice,
        create_info: *const vk::DeviceCreateInfo,
        p_allocator: *const vk::AllocationCallbacks,
        p_device: *mut vk::Device,
    ) -> vk::Result {
        let create_info = unsafe { create_info.as_ref() }.unwrap();
        let mut p_next_chain: VulkanBaseOutStructChain =
            unsafe { (create_info.p_next as *mut vk::BaseOutStructure).as_mut() }.into();
        let layer_create_info = p_next_chain.find_map(|out_struct| {
            let out_struct = out_struct as *mut vk::BaseOutStructure;
            let layer_create_info = unsafe {
                ash::match_out_struct!(match out_struct {
                    out_struct @ VkLayerDeviceCreateInfo => {
                        out_struct
                    }
                    _ => {
                        return None;
                    }
                })
            };
            if layer_create_info.function == VkLayerFunction::VK_LAYER_LINK_INFO {
                Some(layer_create_info)
            } else {
                None
            }
        });
        let layer_create_info = match layer_create_info {
            Some(layer_create_info) => layer_create_info,
            None => {
                error!("failed to find the VkLayerDeviceCreateInfo struct in the chain.");
                return vk::Result::ERROR_INITIALIZATION_FAILED;
            }
        };
        let layer_link = unsafe { layer_create_info.u.pLayerInfo.as_ref() }.unwrap();
        layer_create_info.u.pLayerInfo = layer_link.pNext;

        let get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr =
            layer_link.pfnNextGetInstanceProcAddr;
        let get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr = layer_link.pfnNextGetDeviceProcAddr;

        let global = Self::instance();
        let physical_device_info = global
            .get_physical_info(physical_device)
            .expect("physical device must be a valid VkPhysicalDevice.");
        let instance_info = global
            .get_instance_info(physical_device_info.owner_instance)
            .expect("The owner instance of this physical device must be registered.");

        let create_device_name = CString::new("vkCreateDevice").unwrap();
        let next_create_device = unsafe {
            get_instance_proc_addr(
                instance_info.dispatch_table.core.handle(),
                create_device_name.as_ptr(),
            )
        };
        let next_create_device: vk::PFN_vkCreateDevice =
            unsafe { std::mem::transmute(next_create_device) };
        let mut create_info = *create_info;
        let requested_extensions = unsafe {
            slice_from_raw_parts(
                create_info.pp_enabled_extension_names,
                create_info.enabled_extension_count,
            )
        };
        let requested_extensions =
            unsafe { slice_to_owned_strings(requested_extensions) }.collect::<Vec<_>>();
        let create_device_res = if instance_info.is_create_device_hooked {
            instance_info
                .customized_info
                .borrow()
                .hooks()
                .create_device(
                    physical_device,
                    &create_info,
                    layer_link,
                    unsafe { p_allocator.as_ref() },
                    unsafe { ptr_as_uninit_mut(p_device) }.unwrap(),
                )
        } else {
            LayerResult::Unhandled
        };
        let layer_manifest = T::manifest();
        match create_device_res {
            LayerResult::Handled(Ok(())) => {}
            LayerResult::Handled(Err(e)) => return e,
            LayerResult::Unhandled => {
                let enabled_extensions = requested_extensions
                    .iter()
                    .filter_map(|extension_name| {
                        let extension_name_cstring = CString::new(extension_name.clone())
                            .unwrap_or_else(|e| {
                                panic!("Failed to create CString from {}: {}", extension_name, e)
                            });
                        let extension: Extension = match extension_name.as_str().try_into() {
                            Ok(extension) => extension,
                            Err(_) => return Some(extension_name_cstring),
                        };
                        if layer_manifest
                            .device_extensions
                            .iter()
                            .any(|layer_extension| layer_extension.name == extension)
                        {
                            None
                        } else {
                            Some(extension_name_cstring)
                        }
                    })
                    .collect::<Vec<_>>();
                let enabled_extensions = enabled_extensions
                    .iter()
                    .map(|extension_name| extension_name.as_ptr())
                    .collect::<Vec<_>>();
                create_info.enabled_extension_count = enabled_extensions.len().try_into().unwrap();
                create_info.pp_enabled_extension_names = if enabled_extensions.is_empty() {
                    null()
                } else {
                    enabled_extensions.as_ptr()
                };
                let res = unsafe {
                    next_create_device(physical_device, &create_info, p_allocator, p_device)
                };
                if res != vk::Result::SUCCESS {
                    return res;
                }
            }
        }

        let device = *unsafe { p_device.as_ref() }.unwrap();
        let ash_instance = Arc::clone(&instance_info.dispatch_table.core);
        let ash_device = unsafe {
            // ash will also try to load instance-level dispatchable commands with
            // vkGetDeviceProcAddr. Although that won't end up with undefined behavior, and we
            // should always expect NULL returned according to the spec, we may see Android vulkan
            // loader complaining about internal vkGetDeviceProcAddr called for <function name>,
            // which should be benign.
            ash::Device::load(
                &vk::InstanceFnV1_0 {
                    get_device_proc_addr,
                    ..ash_instance.fp_v1_0().clone()
                },
                device,
            )
        };
        let ash_device = Arc::new(ash_device);
        let enabled_extensions = requested_extensions
            .iter()
            .filter_map(|name| {
                let extension_name: Option<Extension> = name.as_str().try_into().ok();
                extension_name
            })
            .collect::<BTreeSet<_>>();
        let api_version = instance_info
            .api_version
            .min(physical_device_info.properties.api_version.into());
        let customized_info = global.layer_info.create_device_info(
            physical_device,
            &create_info,
            unsafe { p_allocator.as_ref() },
            Arc::clone(&ash_device),
            get_device_proc_addr,
        );
        let device_commands = global.create_device_commands(
            instance_info.customized_info.borrow(),
            Some(customized_info.borrow()),
        );
        {
            let mut device_map = global.device_map.lock().unwrap();
            let mut device_map = device_map.get_mut_or_default();
            assert!(
                !device_map.contains_key(&device.get_dispatch_key()),
                "duplicate VkDevice: {:?}",
                device
            );
            device_map.insert(
                device.get_dispatch_key(),
                Arc::new(DeviceInfoWrapper {
                    dispatch_table: DeviceDispatchTable::load(get_device_proc_addr, ash_device),
                    get_device_proc_addr,
                    api_version,
                    enabled_extensions,
                    device_commands,
                    customized_info,
                }),
            );
        }
        vk::Result::SUCCESS
    }

    extern "system" fn destroy_device(
        device: vk::Device,
        p_allocator: *const vk::AllocationCallbacks,
    ) {
        if device == vk::Device::null() {
            return;
        }
        let global = Self::instance();
        let device_info = match Arc::try_unwrap(
            global
                .device_map
                .lock()
                .unwrap()
                .get_mut_or_default()
                .remove(&device.get_dispatch_key())
                .expect("device must be registered"),
        ) {
            Ok(device_info) => device_info,
            Err(_) => panic!("This should be the sole owner of the device {:?}", device),
        };
        let allocation_callback = unsafe { p_allocator.as_ref() };
        unsafe {
            device_info
                .dispatch_table
                .core
                .destroy_device(allocation_callback)
        };
    }

    fn layer_properties() -> Vec<vk::LayerProperties> {
        let layer_manifest = T::manifest();
        // layer_name and description will be set later
        let mut layer_property: vk::LayerProperties = vk::LayerProperties::builder()
            .spec_version(layer_manifest.spec_version)
            .implementation_version(layer_manifest.implementation_version)
            .build();
        assert!(layer_manifest.name.len() < vk::MAX_EXTENSION_NAME_SIZE);
        let layer_name = CString::new(layer_manifest.name).unwrap();
        let layer_name = cast_slice(layer_name.as_bytes());
        layer_property.layer_name[..layer_name.len()].copy_from_slice(layer_name);

        let layer_description = CString::new(layer_manifest.description).unwrap();
        let layer_description = cast_slice(layer_description.as_bytes());
        assert!(layer_description.len() < vk::MAX_DESCRIPTION_SIZE);
        layer_property.description[..layer_description.len()].copy_from_slice(layer_description);
        vec![layer_property]
    }

    // Introspection queries required by Android, so we need to expose them as public functions so
    // that the user can further expose them as functions exported by the dynamic link library.

    /// The `vkEnumerateInstanceLayerProperties` entry point provided by the layer framework.
    ///
    /// The return value is decided by [`Layer::manifest`]. Only enumerate the layer itself
    /// according to the
    /// [layer rule](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=vkEnumerateInstanceLayerProperties%20must%20enumerate%20and%20only%20enumerate%20the%20layer%20itself.>).
    ///
    /// # Safety
    /// See valid usage of `vkEnumerateInstanceLayerProperties` at
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceLayerProperties.html>.
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe extern "system" fn enumerate_instance_layer_properties(
        property_count: *mut u32,
        properties: *mut vk::LayerProperties,
    ) -> vk::Result {
        let ret_properties = Self::layer_properties();
        // Safe, because the caller guarantees that `property_count` is a valid pointer to u32, and
        // if the value referenced by property_count is not 0, and properties is not NULL,
        // properties must be a valid pointer to an array of property_count vk::LayerProperties
        // structures. See details in
        // VUID-vkEnumerateInstanceLayerProperties-pPropertyCount-parameter and
        // VUID-vkEnumerateInstanceLayerProperties-pProperties-parameter.
        unsafe {
            fill_vk_out_array(
                &ret_properties,
                NonNull::new(property_count).unwrap(),
                properties,
            )
        }
    }

    /// The `vkEnumerateInstanceExtensionProperties` entry point provided by the layer framework.
    ///
    /// Returns an empty list with `VK_SUCCESS` if the layer name matches; returns
    /// `VK_ERROR_LAYER_NOT_PRESENT` with all the out pointers untouched, according to the
    /// [`LLP_LAYER_15`](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=LLP_LAYER_15,Conventions%20and%20Rules>)
    /// rule.
    ///
    /// # Safety
    /// See valid usage of `vkEnumerateInstanceExtensionProperties` at
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceExtensionProperties.html>.
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe extern "system" fn enumerate_instance_extension_properties(
        layer_name: *const c_char,
        property_count: *mut u32,
        _: *mut vk::ExtensionProperties,
    ) -> vk::Result {
        if !layer_name.is_null() {
            let layer_name = unsafe { CStr::from_ptr(layer_name) }
                .to_str()
                .expect(concat!(
                    "According to \
                     VUID-vkEnumerateInstanceExtensionProperties-pLayerName-parameter, ",
                    "if p_layer_name is not NULL, p_layer_name must be a null-terminated UTF-8 \
                     string."
                ));
            if layer_name == T::manifest().name {
                // Safe, because the caller guarantees that if the passed in `property_count` is not
                // null, it's a valid pointer to u32.
                if let Some(property_count) = unsafe { property_count.as_mut() } {
                    *property_count = 0;
                }
                return vk::Result::SUCCESS;
            }
        }
        vk::Result::ERROR_LAYER_NOT_PRESENT
    }

    /// The `vkEnumerateDeviceLayerProperties` entry point provided by the layer framework.
    ///
    /// The return value is decided by [`Layer::manifest`]. Only enumerate the layer itself
    /// according to the
    /// [layer interface version 0 requirement](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=vkEnumerateDeviceLayerProperties%20enumerates%20a,device%20function%20interception.>).
    /// Note that although this function
    /// [is deprecated](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=vkEnumerateDeviceLayerProperties%20is%20deprecated,in%20undefined%20behavior.>)
    /// in the Khronos Vulkan loader, it is still used in the Android Vulkan loader.
    ///
    /// # Safety
    /// See valid usage of `vkEnumerateDeviceLayerProperties` at
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceLayerProperties.html>.
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe extern "system" fn enumerate_device_layer_properties(
        _: vk::PhysicalDevice,
        p_property_count: *mut u32,
        p_properties: *mut vk::LayerProperties,
    ) -> vk::Result {
        // TODO: call into the customized instance data with the vk::PhysicalDevice to provide more
        // flexibility.
        let ret_properties = Self::layer_properties();
        // Safe, because the caller guarantees that `p_property_count` is a valid pointer to u32,
        // and if the value referenced by `p_property_count` is not 0, and `p_properties` is not
        // NULL, `p_properties` must be a valid pointer to an array of `p_property_count`
        // vk::LayerProperties structures. See details in
        // VUID-vkEnumerateDeviceLayerProperties-pPropertyCount-parameter
        // and VUID-vkEnumerateDeviceLayerProperties-pProperties-parameter.
        unsafe {
            fill_vk_out_array(
                &ret_properties,
                NonNull::new(p_property_count).expect(concat!(
                    "p_property_count must be a valid pointer to u32 according to ",
                    "VUID-vkEnumerateDeviceLayerProperties-pPropertyCount-parameter"
                )),
                p_properties,
            )
        }
    }

    /// The `vkEnumerateDeviceExtensionProperties` entry point provided by the layer framework.
    ///
    /// The return value is decided by [`Layer::manifest`] where `p_layer_name` is itself. If the
    /// `p_layer_name` doesn't match, the behavior depends on whether the `physical_device` argument
    /// is `VK_NULL_HANDLE` or not: if `physical_device` argument is `VK_NULL_HANDLE`,
    /// `VK_ERROR_LAYER_NOT_PRESENT` is returned; if `physical_device` is not `VK_NULL_HANDLE`, the
    /// layer framework calls into the next layer of the call chain. This behavior is defined by the [layer interface version 0](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=vkEnumerateDeviceExtensionProperties%20enumerates%20device,function%20never%20fails.>)
    /// and
    /// [`LLP_LAYER_16`](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=LLP_LAYER_16,Conventions%20and%20Rules>).
    ///
    /// # Safety
    /// See valid usage of `vkEnumerateDeviceExtensionProperties` at
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceExtensionProperties.html>.
    /// A NULL `VkPhysicalDevice` can also be used to call this interface, according to the
    /// [layer interface version 0](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#:~:text=vkEnumerateDeviceExtensionProperties%20enumerates%20device,function%20never%20fails.>).
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe extern "system" fn enumerate_device_extension_properties(
        physical_device: vk::PhysicalDevice,
        p_layer_name: *const c_char,
        p_property_count: *mut u32,
        p_properties: *mut vk::ExtensionProperties,
    ) -> vk::Result {
        let layer_manifest = T::manifest();
        let is_this_layer = if p_layer_name.is_null() {
            false
        } else {
            unsafe { CStr::from_ptr(p_layer_name) }
                .to_str()
                .map(|layer_name| layer_name == layer_manifest.name)
                .unwrap_or(false)
        };
        if !is_this_layer {
            if physical_device == vk::PhysicalDevice::null() {
                return vk::Result::ERROR_LAYER_NOT_PRESENT;
            }
            let global = Self::instance();
            let physical_device_info = global
                .get_physical_info(physical_device)
                .expect("physical device must be a valid VkPhysicalDevice.");
            let instance_info = global
                .get_instance_info(physical_device_info.owner_instance)
                .expect("The owner instance of this physical device must be registered.");
            return unsafe {
                (instance_info
                    .dispatch_table
                    .core
                    .fp_v1_0()
                    .enumerate_device_extension_properties)(
                    physical_device,
                    p_layer_name,
                    p_property_count,
                    p_properties,
                )
            };
        }
        let property_count = NonNull::new(p_property_count).expect(concat!(
            "`p_property_count` must be a valid pointer to u32 according to ",
            "VUID-vkEnumerateDeviceExtensionProperties-pPropertyCount-parameter."
        ));
        // Safe because the caller guarantees `p_property_count` is a valid pointer to u32 according
        // to VUID-vkEnumerateDeviceExtensionProperties-pPropertyCount-parameter, and if
        // `p_property_count` doesn't point to 0, p_properties is either NULL or a valid pointer to
        // an array of `p_property_count` `vk::ExtensionProperties` structures according to
        // VUID-vkEnumerateDeviceExtensionProperties-pProperties-parameter.
        let device_extensions = layer_manifest
            .device_extensions
            .iter()
            .cloned()
            .map(Into::<vk::ExtensionProperties>::into)
            .collect::<Vec<_>>();
        unsafe { fill_vk_out_array(&device_extensions, property_count, p_properties) }
    }

    /// The `vkGetInstanceProcAddr` entry point provided by the layer framework.
    ///
    /// The layer framework will make use of [`Layer::hooked_instance_commands`] and
    /// [`Layer::hooked_device_commands`] (with the `device_info` argument being [`None`]) to decide
    /// whether a local function pointer should be returned or should just passthrough to the next
    /// layer in the call chain. Note that the layer implementation doesn't have to take care of
    /// whether the command is enabled/available, and only needs to use the interfaces to express
    /// the intent. The layer framework should handle most of the logic(e.g. the requirement of
    /// [`LLP_LAYER_18`](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#pre-instance-functions:~:text=LLP_LAYER_18,Conventions%20and%20Rules>))
    /// here unless the layer implementation decides to intercept `vkGetInstanceProcAddr`.
    ///
    /// The actual behavior is defined as follow:
    ///
    /// 1. If the `instance` parameter is `VK_NULL_HANDLE`, returns the local function pointers of
    ///    the current layer for all
    ///    [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
    ///    and `vkGetInstanceProcAddr`; `NULL` is returned for other Vulkan commands. One culprit
    ///    for now is that currently the layer framework always returns `NULL` for
    ///    `vkEnumerateInstanceVersion`, but that should be Ok, because the layer framework
    ///    currently doesn't allow to intercept pre-instance functions anyway, so this function
    ///    should be completely handled by the Vulkan loader.
    ///
    /// 1. If instance is not null, the situation is more complicated:
    ///
    ///    If the layer implementation intercepts the `vkGetInstanceProcAddr` function(i.e.
    ///    [`Layer::hooked_instance_commands`] returns `vkGetInstanceProcAddr`), the layer framework
    ///    just calls [`InstanceHooks::get_instance_proc_addr`] and return the return value if
    ///    [`LayerResult::Handled`] is returned.
    ///
    ///    Otherwise, the layer framework handles most of the logic according to
    ///    [`Layer::hooked_instance_commands`] and [`Layer::hooked_device_commands`]:
    ///
    ///     1. For dispatchable commands that are introspection queries:
    ///        `vkEnumerateDeviceExtensionProperties`, `vkEnumerateDeviceLayerProperties`, always
    ///        returns a local function pointer of the current layer.
    ///
    ///     1. For `vkDestroyInstance`, `vkCreateDevice`, `vkDestroyDevice`, always returns a local
    ///        function pointer. The layer framework needs to intercept those functions to build the
    ///        dispatch table and perform instance/device specific initialization. Note that
    ///        `vkCreateInstance` is a
    ///        [global command](<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.>),
    ///        so `NULL` will be returned if the `instance` argument is not `VK_NULL_HANDLE`.
    ///
    ///     1. For `vkEnumeratePhysicalDevices`, `vkEnumeratePhysicalDeviceGroups`, always returns a
    ///        local function pointer. The layer framework needs to intercept those functions to
    ///        build a map between `VkPhysicalDevice` and `VkInstance` so that the layer framework
    ///        can find the correspondent `VkInstance` in `vkCreateDevice`.
    ///
    ///     1. For other core dispatchable commands and enabled instance extension dispatchable
    ///        commands, if the layer implementation decides to intercept(according to
    ///        [`Layer::hooked_instance_commands`] and [`Layer::hooked_device_commands`]), returns a
    ///        local function pointer, otherwise returns the function pointer from the next layer.
    ///
    ///     1. For disabled instance extension dispatchable commands(either because the API version
    ///        is not high enough or the extension is not enabled), directly call to the next layer
    ///        regardless of whether the layer implementation wants to intercept the command or not,
    ///        and rely on the next layer to return `NULL`.
    ///
    ///     1. For device extension dispatchable commands directly supported by the layer
    ///        implementation according to [`LayerManifest::device_extensions`], always returns a
    ///        local function pointer if the layer implementation intercepts the command. Otherwise,
    ///        returns the function pointer of the next layer.
    ///
    ///     1. For device extension dispatchable commands not directly supported by the layer(i.e.
    ///        the extension doesn't appear in the [`LayerManifest::device_extensions`] of the
    ///        current layer):
    ///
    ///        * If the next layer returns `NULL`, `NULL` is always returned regardless of whether
    ///          the layer implementation wants to intercept the command. This indicates that this
    ///          device extension dispathcable command is not available for the instance.
    ///
    ///        * if the next layer returns non-NULL, a local function pointer is returned if the
    ///          layer implementation wants to intercept the command, otherwise returns the function
    ///          pointer of the next layer This indicates that this device extension dispathcable
    ///          command is available for the instance.
    ///
    ///     1. Passthrough to the next layer if the command is not recognized.
    ///
    /// # Safety
    /// See valid usage of `vkGetInstanceProcAddr` at
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html>.
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe extern "system" fn get_instance_proc_addr(
        instance: vk::Instance,
        p_name: *const c_char,
    ) -> vk::PFN_vkVoidFunction {
        // Make sure Global is initialized.
        let global = Self::instance();
        // Safe because the caller is expected to pass in a C string in the name parameter. See
        // VUID-vkGetInstanceProcAddr-pName-parameter.
        let name = unsafe { CStr::from_ptr(p_name) };
        let name = name.to_str().expect(concat!(
            "According to VUID-vkGetInstanceProcAddr-pName-parameter, p_name must be a null-",
            "terminated UTF-8 string."
        ));
        // Returns a valid function pointer for global commands all the time, although this behavior
        // doesn't match the spec. See https://github.com/KhronosGroup/Vulkan-Loader/issues/1537 for
        // details.
        match name {
            "vkGetInstanceProcAddr" => {
                return unsafe {
                    std::mem::transmute::<vk::PFN_vkGetInstanceProcAddr, vk::PFN_vkVoidFunction>(
                        Self::get_instance_proc_addr,
                    )
                }
            }
            "vkEnumerateInstanceExtensionProperties" => {
                return unsafe {
                    std::mem::transmute::<
                        vk::PFN_vkEnumerateInstanceExtensionProperties,
                        vk::PFN_vkVoidFunction,
                    >(Self::enumerate_instance_extension_properties)
                }
            }
            "vkEnumerateInstanceLayerProperties" => {
                return unsafe {
                    std::mem::transmute::<
                        vk::PFN_vkEnumerateInstanceLayerProperties,
                        vk::PFN_vkVoidFunction,
                    >(Self::enumerate_instance_layer_properties)
                }
            }
            "vkCreateInstance" => {
                return unsafe {
                    std::mem::transmute::<vk::PFN_vkCreateInstance, vk::PFN_vkVoidFunction>(
                        Self::create_instance,
                    )
                }
            }
            _ => {}
        }
        if instance == vk::Instance::null() {
            // TODO: allow inteception of vkEnumerateInstanceVersion and handle
            // vkEnumerateInstanceVersion properly. For now, we still return NULL for
            // vkEnumerateInstanceVersion. The Vulkan loader should cover us for this case.
            // Per spec, if instance is NULL, and pName is neither NULL nor a global command, NULL
            // should be returned.
            return None;
        }
        let instance_info = global.get_instance_info(instance)?;
        if global.get_instance_addr_proc_hooked {
            if let LayerResult::Handled(res) = instance_info
                .customized_info
                .borrow()
                .hooks()
                .get_instance_proc_addr(name)
            {
                return res;
            }
        }
        let get_next_proc_addr =
            || unsafe { (instance_info.get_instance_proc_addr)(instance, p_name) };
        let instance_commands = &instance_info.instance_commands;
        let instance_command = match instance_commands
            .binary_search_by_key(&name, |VulkanCommand { name, .. }| name)
        {
            Ok(index) => Some(&instance_commands[index]),
            Err(_) => None,
        };
        if let Some(instance_command) = instance_command {
            if !instance_command.hooked {
                return get_next_proc_addr();
            }
            let enabled = instance_command.features.is_command_enabled(
                &instance_info.api_version,
                &instance_info.enabled_extensions,
            );
            if !enabled {
                return get_next_proc_addr();
            }
            return instance_command.proc;
        }
        let device_commands = &instance_info.device_commands;
        let device_command =
            match device_commands.binary_search_by_key(&name, |VulkanCommand { name, .. }| name) {
                Ok(index) => Some(&device_commands[index]),
                Err(_) => None,
            };
        if let Some(device_command) = device_command {
            let next_proc_addr = get_next_proc_addr();
            if !device_command.hooked {
                return next_proc_addr;
            }
            let layer_device_extensions: BTreeSet<Extension> = T::manifest()
                .device_extensions
                .iter()
                .map(|ExtensionProperties { name, .. }| name.clone())
                .collect();
            // If the layer supports the command or the next proc addr can find it, this is an
            // available device command.
            let command_available = device_command
                .features
                .is_command_enabled(&instance_info.api_version, &layer_device_extensions)
                || next_proc_addr.is_some();
            if command_available {
                return device_command.proc;
            } else {
                return next_proc_addr;
            }
        }
        // We don't recognize this command.
        get_next_proc_addr()
    }

    /// The `vkGetDeviceProcAddr` entry point provided by the layer framework.
    ///
    /// The layer framework will make use of [`Layer::hooked_device_commands`] to decide whether a
    /// local function pointer should be returned or should passthrough to the next layer in the
    /// call chain. Note that the layer implementation doesn't have to take care of whether the
    /// command is enabled(either included in the requested core version or enabled extension), and
    /// only needs to use the [`Layer::hooked_device_commands`] interface to express the intent. The
    /// layer framework should handle most of the logic(e.g. the requirement of
    /// [`LLP_LAYER_18`](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#pre-instance-functions:~:text=LLP_LAYER_18,Conventions%20and%20Rules>))
    /// here unless the layer implementation decides to intercept `vkGetDeviceProcAddr`.
    ///
    /// # Safety
    /// See valid usage of `vkGetDeviceProcAddr` at
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetDeviceProcAddr.html>.
    #[deny(unsafe_op_in_unsafe_fn)]
    pub unsafe extern "system" fn get_device_proc_addr(
        device: vk::Device,
        p_name: *const c_char,
    ) -> vk::PFN_vkVoidFunction {
        let name = unsafe { CStr::from_ptr(p_name) };
        let name = name.to_str().expect("name should be a valid UTF-8 string.");
        let global = Self::instance();
        let device_info = global.get_device_info(device)?;
        if let LayerResult::Handled(res) = device_info
            .customized_info
            .borrow()
            .hooks()
            .get_device_proc_addr(name)
        {
            return res;
        }
        let get_next_device_proc_addr =
            || unsafe { (device_info.get_device_proc_addr)(device, p_name) };
        let device_commands = &device_info.device_commands;
        let command = if let Ok(index) =
            device_commands.binary_search_by_key(&name, |VulkanCommand { name, .. }| name)
        {
            &device_commands[index]
        } else {
            return get_next_device_proc_addr();
        };
        if !command
            .features
            .is_command_enabled(&device_info.api_version, &device_info.enabled_extensions)
        {
            return get_next_device_proc_addr();
        }
        if !command.hooked {
            return get_next_device_proc_addr();
        }
        command.proc
    }
}

impl<T: Layer> Default for Global<T> {
    fn default() -> Self {
        let layer_info: T = Default::default();
        let get_instance_addr_proc_hooked = T::GlobalHooksInfo::hooked_commands()
            .contains(&LayerVulkanCommand::GetInstanceProcAddr);
        Self {
            instance_map: Default::default(),
            physical_device_map: Default::default(),
            device_map: Default::default(),
            layer_info,
            get_instance_addr_proc_hooked,
        }
    }
}

/// A stub struct that intercept no commands, which implements [`GlobalHooks`] and
/// [`GlobalHooksInfo`].
#[derive(Default)]
pub struct StubGlobalHooks;

#[auto_globalhooksinfo_impl]
impl GlobalHooks for StubGlobalHooks {}

/// A stub struct that intercept no commands, which implements [`InstanceHooks`] and
/// [`InstanceInfo`].
#[derive(Default)]
pub struct StubInstanceInfo;

#[auto_instanceinfo_impl]
impl InstanceHooks for StubInstanceInfo {}

/// A stub struct that intercept no commands, which implements [`DeviceHooks`] and [`DeviceInfo`].
#[derive(Default)]
pub struct StubDeviceInfo;

#[auto_deviceinfo_impl]
impl DeviceHooks for StubDeviceInfo {}

#[cfg(test)]
mod tests {
    use once_cell::sync::Lazy;

    use crate::test_utils::LayerManifestExt;
    use std::{cmp::Ordering, mem::MaybeUninit};

    use super::*;

    #[test]
    fn commands_must_be_sorted() {
        #[derive(Default)]
        struct TestLayer(StubGlobalHooks);

        impl Layer for TestLayer {
            type GlobalHooksInfo = StubGlobalHooks;
            type InstanceInfo = StubInstanceInfo;
            type DeviceInfo = StubDeviceInfo;
            type InstanceInfoContainer = StubInstanceInfo;
            type DeviceInfoContainer = StubDeviceInfo;

            fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
                &*GLOBAL
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

        static GLOBAL: Lazy<Global<TestLayer>> = Lazy::new(Default::default);

        #[inline]
        fn check<T: PartialOrd>(last: &mut T) -> impl FnMut(T) -> bool + '_ {
            move |curr| {
                if let Some(Ordering::Greater) | None = (*last).partial_cmp(&curr) {
                    return false;
                }
                *last = curr;
                true
            }
        }

        let instance_info = Default::default();
        let device_commands =
            Global::<TestLayer>::instance().create_device_commands(&instance_info, None);
        let mut name_iter = device_commands
            .iter()
            .map(|VulkanCommand { name, .. }| *name);
        let mut last = name_iter.next().unwrap();

        assert!(name_iter.all(check(&mut last)));

        let instance_commands =
            Global::<TestLayer>::instance().create_instance_commands(&instance_info);
        let mut name_iter = instance_commands
            .iter()
            .map(|VulkanCommand { name, .. }| *name);
        let mut last = name_iter.next().unwrap();

        assert!(name_iter.all(check(&mut last)));
    }

    #[test]
    fn fill_vk_out_array_should_handle_zero_count_correctly() {
        let extensions = [ExtensionProperties {
            name: Extension::KHRSwapchain,
            spec_version: 1,
        }]
        .into_iter()
        .map(Into::<vk::ExtensionProperties>::into)
        .collect::<Vec<_>>();

        let mut count = 0;
        assert_eq!(
            unsafe { fill_vk_out_array(&extensions, (&mut count).into(), null_mut()) },
            vk::Result::SUCCESS
        );
        assert_eq!(count, extensions.len());

        count = 0;
        let mut extension_property = MaybeUninit::<vk::ExtensionProperties>::uninit();
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    &extensions,
                    (&mut count).into(),
                    extension_property.as_mut_ptr(),
                )
            },
            vk::Result::INCOMPLETE
        );
        assert_eq!(count, 0);

        count = 0;
        assert_eq!(
            unsafe { fill_vk_out_array(&[], (&mut count).into(), extension_property.as_mut_ptr()) },
            vk::Result::SUCCESS
        );
    }

    #[test]
    fn fill_vk_out_array_should_return_incomplete_with_short_out_array() {
        let extensions = [
            ExtensionProperties {
                name: Extension::KHRSwapchain,
                spec_version: 1,
            },
            ExtensionProperties {
                name: Extension::KHRSamplerYcbcrConversion,
                spec_version: 1,
            },
        ]
        .into_iter()
        .map(Into::<vk::ExtensionProperties>::into)
        .collect::<Vec<_>>();

        let mut count = 1;
        let mut extension_property = MaybeUninit::<vk::ExtensionProperties>::uninit();
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    &extensions,
                    (&mut count).into(),
                    extension_property.as_mut_ptr(),
                )
            },
            vk::Result::INCOMPLETE
        );
        assert_eq!(count, 1);
    }
}
