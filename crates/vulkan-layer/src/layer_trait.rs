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

//! This module includes all the traits that require the layer implementation to implement.

use crate::{bindings::vk_layer::VkLayerInstanceLink, global_simple_intercept::Extension, Global};
use ash::vk;
use std::{borrow::Borrow, ffi::CString, ops::Deref, sync::Arc};
use thiserror::Error;

pub mod generated;
pub use generated::*;

/// The return value of an intercepted Vulkan command by the layer implementation.
///
/// The layer framework uses the value of this type to decide whether to call into the next call
/// chain after calling into the layer implementation. For an intercepted Vulkan command, if the
/// layer implementation returns [`LayerResult::Handled`], the layer framework won't call into the
/// next layer call chain. Otherwise, if [`LayerResult::Unhandled`] is returned, the layer framework
/// will call into the next layer call chain with the original parameters.
#[must_use]
#[derive(Clone)]
pub enum LayerResult<T> {
    /// The layer implementation has handled this Vulkan command, so the layer framework shouldn't
    /// try to call into the next call chain.
    ///
    /// This can be useful if the layer implementation wants to modify the parameters passed to the
    /// next call chain, or wants to use some other Vulkan commands to fake/emulate the intercepted
    /// Vulkan command. This is useful when the layer wants to emulate an unsupported extension.
    /// `T` is usually `ash::prelude::VkResult<U>`.
    Handled(T),
    /// The layer implementation doesn't handle this Vulkan command, so the layer framework should
    /// call into the next call chain with the original parameters.
    ///
    /// If the layer implementation doesn't want to bother calling into the next call chain, and
    /// wants the layer framework to do the job, this variant can be used. Note that if the layer
    /// implementation needs to inform the layer framework that there is an error and needs to
    /// return early, [`LayerResult::Handled`] should be used.
    Unhandled,
}

#[derive(Error, Debug)]
pub enum TryFromVulkanCommandError {
    #[error("unknown command `{0}`")]
    UnknownCommand(String),
}

/// A trait for the layer implementation to provide metadata of [`DeviceHooks`] for the layer
/// framework.
///
/// This trait serves as the container of the [`DeviceHooks`] type and provides the list of
/// intercepted
/// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions)
/// for the layer framework. This trait and [`DeviceHooks`] can be implemented by the same type.
///
/// This trait can also be automatically implemented with the help of the
/// [`auto_deviceinfo_impl`][crate::auto_deviceinfo_impl] attribute macro.
///
/// # Examples
///
/// A layer that needs to intercept the `vkCreateImage` function.
/// ```
/// # use ash::{vk, prelude::VkResult};
/// # use vulkan_layer::{DeviceHooks, DeviceInfo, LayerVulkanCommand as VulkanCommand, LayerResult};
/// #
/// struct MyLayerDeviceInfo;
///
/// impl DeviceHooks for MyLayerDeviceInfo {
///     fn create_image(
///         &self,
///         _p_create_info: &vk::ImageCreateInfo,
///         _p_allocator: Option<&vk::AllocationCallbacks>,
///     ) -> LayerResult<VkResult<vk::Image>> {
///         LayerResult::Unhandled
///     }
/// }
///
/// impl DeviceInfo for MyLayerDeviceInfo {
///     type HooksType = Self;
///     type HooksRefType<'a> = &'a Self;
///
///     fn hooked_commands() -> &'static [VulkanCommand] {
///         &[VulkanCommand::CreateImage]
///     }
///
///     fn hooks(&self) -> Self::HooksRefType<'_> {
///         self
///     }
/// }
/// ```
pub trait DeviceInfo: Send + Sync {
    /// The underlying [`DeviceHooks`] type that implements the core logic to intercept
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions).
    ///
    /// If a type implements both this trait and the [`DeviceHooks`] trait simultaneously,
    /// [`DeviceInfo::HooksType`] can be set to `Self`.
    type HooksType: DeviceHooks;

    /// A type that can be dereferencing to [`DeviceInfo::HooksType`]. Usually just
    /// `&DeviceInfo::HooksType`.
    ///
    /// This extra associated type allows the layer implementation to use a smart pointer like
    /// [`Arc`][std::sync::Arc] or [`MutexGuard`][std::sync::MutexGuard] to hold the reference.
    ///
    /// # Examples
    ///
    /// A layer implementation that returns a [`Arc`][std::sync::Arc].
    /// ```
    /// # use std::sync::Arc;
    /// # use vulkan_layer::{DeviceHooks, DeviceInfo, LayerVulkanCommand as VulkanCommand};
    /// #
    /// # struct MyLayerDeviceHooks;
    /// #
    /// # impl DeviceHooks for MyLayerDeviceHooks {}
    /// #
    /// struct MyLayerDeviceInfo(Arc<MyLayerDeviceHooks>);
    ///
    /// impl DeviceInfo for MyLayerDeviceInfo {
    ///     type HooksType = MyLayerDeviceHooks;
    ///     type HooksRefType<'a> = Arc<MyLayerDeviceHooks>;
    /// #
    /// #     fn hooked_commands() -> &'static [VulkanCommand] {
    /// #         &[]
    /// #     }
    ///
    ///     // Other required methods omitted.
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         Arc::clone(&self.0)
    ///     }
    /// }
    /// ```
    type HooksRefType<'a>: Deref<Target = Self::HooksType> + 'a
    where
        Self: 'a;

    /// Returns a slice of
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions)
    /// that the layer implementation needs to intercept.
    ///
    /// This function should normally return the methods implemented by the associated
    /// [`DeviceHooks`] type. If the layer implementation needs to decide the functions to intercept
    /// dynamically at runtime, use the [`Layer::hooked_device_commands`] method to override the
    /// list of intercepted device functions.
    ///
    /// Avoid returning commands other than
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions).
    /// Otherwise, those commands may or may not be intercepted.
    ///
    /// For Vulkan commands that have multiple names, e.g. `vkCreateSamplerYcbcrConversion` and
    /// `vkCreateSamplerYcbcrConversionKHR`, only the shortest name should be
    /// used([`VulkanCommand::CreateSamplerYcbcrConversion`] in this example). The layer framework
    /// automatically wires both entries to the layer implementation.
    ///
    /// # Examples
    ///
    /// A layer that intercepts the `vkCreateImage` only if the ASTC LDR feature is enabled.
    /// ```
    /// use ash::{self, prelude::VkResult, vk};
    /// use once_cell::sync::Lazy;
    /// use std::sync::Arc;
    /// use vulkan_layer::{
    ///     DeviceHooks, DeviceInfo, Global, Layer, LayerManifest, LayerResult,
    ///     LayerVulkanCommand as VulkanCommand, StubGlobalHooks, StubInstanceInfo,
    ///     VulkanBaseInStructChain,
    /// };
    ///
    /// struct MyLayerDeviceInfo {
    ///     is_astc_enabled: bool,
    /// }
    ///
    /// impl DeviceHooks for MyLayerDeviceInfo {
    ///     fn create_image(
    ///         &self,
    ///         create_info: &vk::ImageCreateInfo,
    ///         _p_allocator: Option<&vk::AllocationCallbacks>,
    ///     ) -> LayerResult<VkResult<vk::Image>> {
    ///         if create_info.format == vk::Format::ASTC_4X4_UNORM_BLOCK {
    ///             println!("ASTC 4x4 UNORM image created.");
    ///         }
    ///         LayerResult::Unhandled
    ///     }
    /// }
    ///
    /// impl DeviceInfo for MyLayerDeviceInfo {
    ///     type HooksType = Self;
    ///     type HooksRefType<'a> = &'a Self;
    ///
    ///     fn hooked_commands() -> &'static [VulkanCommand] {
    ///         &[VulkanCommand::CreateImage]
    ///     }
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         self
    ///     }
    /// }
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     // ...
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    /// #     type InstanceInfo = StubInstanceInfo;
    ///     type DeviceInfo = MyLayerDeviceInfo;
    /// #     type InstanceInfoContainer = StubInstanceInfo;
    ///     type DeviceInfoContainer = MyLayerDeviceInfo;
    /// #
    /// #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    /// #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    /// #         &*GLOBAL
    /// #     }
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    /// #
    /// #     fn create_instance_info(
    /// #         &self,
    /// #         _: &vk::InstanceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Instance>,
    /// #         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    /// #     ) -> Self::InstanceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// #
    ///     fn create_device_info(
    ///         &self,
    ///         _: vk::PhysicalDevice,
    ///         create_info: &vk::DeviceCreateInfo,
    ///         _: Option<&vk::AllocationCallbacks>,
    ///         _: Arc<ash::Device>,
    ///         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ///     ) -> Self::DeviceInfoContainer {
    ///         let mut p_next_chain: VulkanBaseInStructChain =
    ///             unsafe { (create_info.p_next as *const vk::BaseInStructure).as_ref() }.into();
    ///         let is_astc_enabled = p_next_chain
    ///             .find_map(|p_next| {
    ///                 let p_next = p_next as *const vk::BaseInStructure;
    ///                 unsafe {
    ///                     ash::match_in_struct!(match p_next {
    ///                         features2 @ vk::PhysicalDeviceFeatures2 => {
    ///                             Some(&features2.features)
    ///                         }
    ///                         _ => {
    ///                             None
    ///                         }
    ///                     })
    ///                 }
    ///             })
    ///             .or_else(|| unsafe { create_info.p_enabled_features.as_ref() })
    ///             .map(|features| features.texture_compression_astc_ldr == vk::TRUE)
    ///             .unwrap_or(false);
    ///         MyLayerDeviceInfo { is_astc_enabled }
    ///     }
    ///
    ///     fn hooked_device_commands(
    ///         &self,
    ///         _instance_info: &Self::InstanceInfo,
    ///         device_info: Option<&Self::DeviceInfo>,
    ///     ) -> Box<dyn Iterator<Item = VulkanCommand>> {
    ///         let should_hook_create_image = device_info
    ///             .map(|device_info| device_info.is_astc_enabled)
    ///             // Always hook the vkCreateImage function in the function pointers returned by
    ///             // vkGetInstanceProcAddr: we don't know if the to-be-created VkDevice will be
    ///             // created with the ASTC feature enabled.
    ///             .unwrap_or(true);
    ///         Box::new(
    ///             Self::DeviceInfo::hooked_commands()
    ///                 .iter()
    ///                 .cloned()
    ///                 .filter(move |command| match command {
    ///                     VulkanCommand::CreateImage => should_hook_create_image,
    ///                     _ => true,
    ///                 }),
    ///         )
    ///     }
    /// }
    /// ```
    fn hooked_commands() -> &'static [VulkanCommand];

    /// Returns the reference of the [`DeviceInfo::HooksType`].
    ///
    /// The layer framework uses the returned value to call into the layer implementation of
    /// intercepted commands.
    fn hooks(&self) -> Self::HooksRefType<'_>;
}

/// A trait for the layer implementation to provide metadata of [`InstanceHooks`] for the layer
/// framework.
///
/// This trait serves as the container of the [`InstanceHooks`] type and provides the list of
/// intercepted
/// [Vulkan instance functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#instance-functions)
/// ([global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
/// not included) for the layer framework. This trait and [`InstanceHooks`] can be implemented by
/// the same type. Avoid returning either
/// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions)
/// or
/// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.).
///
/// This trait can also be automatically implemented with the help of the
/// [`auto_instanceinfo_impl`][crate::auto_instanceinfo_impl] attribute macro.
///
/// # Examples
///
/// A layer that needs to intercept the `vkCreateWin32SurfaceKHR` function.
/// ```
/// use ash::{prelude::VkResult, vk};
/// use vulkan_layer::{
///     InstanceHooks, InstanceInfo, LayerResult, LayerVulkanCommand as VulkanCommand,
/// };
///
/// struct MyLayerInstanceHooks;
///
/// impl InstanceHooks for MyLayerInstanceHooks {
///     fn create_win32_surface_khr(
///         &self,
///         _p_create_info: &vk::Win32SurfaceCreateInfoKHR,
///         _p_allocator: Option<&vk::AllocationCallbacks>,
///     ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
///         LayerResult::Unhandled
///     }
/// }
///
/// impl InstanceInfo for MyLayerInstanceHooks {
///     type HooksType = Self;
///     type HooksRefType<'a> = &'a Self;
///
///     fn hooked_commands() -> &'static [VulkanCommand] {
///         &[VulkanCommand::CreateWin32SurfaceKhr]
///     }
///
///     fn hooks(&self) -> Self::HooksRefType<'_> {
///         self
///     }
/// }
/// ```
pub trait InstanceInfo: Send + Sync {
    /// The underlying [`InstanceHooks`] type that implements the core logic to intercept
    /// [Vulkan instance functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#instance-functions)
    /// apart from
    /// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.).
    ///
    /// If a type implements both this trait and the [`InstanceHooks`] trait simultaneously,
    /// [`InstanceInfo::HooksType`] can be set to `Self`.
    type HooksType: InstanceHooks;

    /// A type that can be dereferencing to [`InstanceInfo::HooksType`]. Usually
    /// `&InstanceInfo::HooksType`.
    ///
    /// This extra associated type allows the layer implementation to use a smart pointer like
    /// [`Arc`][std::sync::Arc] or [`MutexGuard`][std::sync::MutexGuard] to hold the reference.
    ///
    /// # Examples
    ///
    /// A layer implementation that returns a [`Arc`][std::sync::Arc].
    /// ```
    /// # use std::sync::Arc;
    /// # use vulkan_layer::{InstanceHooks, InstanceInfo, LayerVulkanCommand as VulkanCommand};
    /// #
    /// # struct MyLayerInstanceHooks;
    /// #
    /// # impl InstanceHooks for MyLayerInstanceHooks {}
    /// #
    /// struct MyLayerInstanceInfo(Arc<MyLayerInstanceHooks>);
    ///
    /// impl InstanceInfo for MyLayerInstanceInfo {
    ///     type HooksType = MyLayerInstanceHooks;
    ///     type HooksRefType<'a> = Arc<MyLayerInstanceHooks>;
    /// #
    /// #     fn hooked_commands() -> &'static [VulkanCommand] {
    /// #         &[]
    /// #     }
    ///
    ///     // Other required methods omitted.
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         Arc::clone(&self.0)
    ///     }
    /// }
    /// ```
    type HooksRefType<'a>: Deref<Target = Self::HooksType> + 'a
    where
        Self: 'a;

    /// Returns a slice of
    /// [Vulkan instance functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#instance-functions)
    /// ([global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
    /// not included) that the layer implementation needs to intercept.
    ///
    /// This function should normally return the methods implemented by the associated
    /// [`InstanceHooks`] type. If the layer implementation needs to decide the function to
    /// intercept dynamically at runtime, use the [`Layer::hooked_instance_commands`] method to
    /// override the list of intercepted instance functions.
    ///
    /// Avoid returning unexpected commands(i.e.
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions)
    /// or
    /// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)).
    /// Otherwise, those commands may or may not be intercepted.
    ///
    /// For Vulkan commands that have multiple names, e.g. `vkGetPhysicalDeviceProperties2` and
    /// `vkGetPhysicalDeviceProperties2KHR`, only the shortest name should be
    /// used([`VulkanCommand::GetPhysicalDeviceProperties2`] in this example). The layer framework
    /// automatically wires both entries to the layer implementation.
    ///
    /// # Examples
    ///
    /// A layer that intercepts `vkDestroySurfaceKHR` only if the `VK_KHR_win32_surface` extension
    /// is enabled.
    /// ```
    /// use ash::vk;
    /// use once_cell::sync::Lazy;
    /// use std::{ffi::CStr, sync::Arc};
    /// use vulkan_layer::{
    ///     Global, InstanceHooks, InstanceInfo, Layer, LayerManifest, LayerResult,
    ///     LayerVulkanCommand as VulkanCommand, StubDeviceInfo, StubGlobalHooks,
    /// };
    ///
    /// struct MyLayerInstanceInfo {
    ///     is_win32_surface_enabled: bool,
    /// }
    ///
    /// impl InstanceHooks for MyLayerInstanceInfo {
    ///     fn destroy_surface_khr(
    ///         &self,
    ///         _surface: vk::SurfaceKHR,
    ///         _p_allocator: Option<&vk::AllocationCallbacks>,
    ///     ) -> LayerResult<()> {
    ///         LayerResult::Unhandled
    ///     }
    /// }
    ///
    /// impl InstanceInfo for MyLayerInstanceInfo {
    ///     type HooksType = Self;
    ///     type HooksRefType<'a> = &'a Self;
    ///
    ///     fn hooked_commands() -> &'static [VulkanCommand] {
    ///         &[VulkanCommand::DestroySurfaceKhr]
    ///     }
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         self
    ///     }
    /// }
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     // ...
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    ///     type InstanceInfo = MyLayerInstanceInfo;
    /// #     type DeviceInfo = StubDeviceInfo;
    ///     type InstanceInfoContainer = MyLayerInstanceInfo;
    /// #     type DeviceInfoContainer = StubDeviceInfo;
    /// #
    /// #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    /// #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    /// #         &*GLOBAL
    /// #     }
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    ///
    ///     fn create_instance_info(
    ///         &self,
    ///         create_info: &vk::InstanceCreateInfo,
    ///         _: Option<&vk::AllocationCallbacks>,
    ///         _: Arc<ash::Instance>,
    ///         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ///     ) -> Self::InstanceInfoContainer {
    ///         let enabled_extensions = if create_info.enabled_extension_count > 0 {
    ///             unsafe {
    ///                 std::slice::from_raw_parts(
    ///                     create_info.pp_enabled_extension_names,
    ///                     create_info.enabled_extension_count as usize,
    ///                 )
    ///             }
    ///         } else {
    ///             &[]
    ///         };
    ///         let is_win32_surface_enabled = enabled_extensions
    ///             .iter()
    ///             .find(|extension_name| {
    ///                 (unsafe { CStr::from_ptr(**extension_name) })
    ///                     == ash::extensions::khr::Win32Surface::name()
    ///             })
    ///             .is_some();
    ///         MyLayerInstanceInfo {
    ///             is_win32_surface_enabled,
    ///         }
    ///     }
    ///
    ///     fn hooked_instance_commands(
    ///         &self,
    ///         instance_info: &Self::InstanceInfo,
    ///     ) -> Box<dyn Iterator<Item = VulkanCommand>> {
    ///         let should_hook_destroy_surface = instance_info.is_win32_surface_enabled;
    ///         Box::new(
    ///             Self::InstanceInfo::hooked_commands()
    ///                 .iter()
    ///                 .cloned()
    ///                 .filter(move |command| match command {
    ///                     VulkanCommand::DestroySurfaceKhr => should_hook_destroy_surface,
    ///                     _ => true,
    ///                 }),
    ///         )
    ///     }
    /// #
    /// #     fn create_device_info(
    /// #         &self,
    /// #         _: vk::PhysicalDevice,
    /// #         _: &vk::DeviceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Device>,
    /// #         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    /// #     ) -> Self::DeviceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// }
    /// ```
    fn hooked_commands() -> &'static [VulkanCommand];

    /// Returns the reference of the [`InstanceInfo::HooksType`].
    ///
    /// The layer framework uses the returned value to call into the layer implementation of
    /// intercepted commands.
    fn hooks(&self) -> Self::HooksRefType<'_>;
}

/// A trait for the layer implementation to provide the implementation of the intercepted
/// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.).
///
/// The global commands are copied here for convenience:
/// * `vkEnumerateInstanceVersion`
/// * `vkEnumerateInstanceExtensionProperties`
/// * `vkEnumerateInstanceLayerProperties`
/// * `vkCreateInstance`
///
/// Currently, it is only supported to intercept the `vkCreateInstance` function.
pub trait GlobalHooks: Send + Sync {
    /// The logic to intercept the `vkCreateInstance` function.
    ///
    /// The implementations should return [`LayerResult::Unhandled`] if the layer implementation
    /// wants the layer framework to handle the task to call into the next layer, otherwise should
    /// return [`LayerResult::Handled`].
    ///
    /// This function is usually intercepted if the layer needs to modify the `VkInstanceCreateInfo`
    /// passed to the next layer. Otherwise, [`Layer::create_instance_info`] is almost always
    /// better. This function is called before [`Layer::create_instance_info`] to create the
    /// `VkInstance`.
    ///
    /// # Arguments
    ///
    /// * `_p_create_info` is a pointer to a VkInstanceCreateInfo structure controlling creation of
    ///   the instance. The `VkLayerInstanceLink` linked list in the `pNext` chain is already
    ///   advanced by the layer framework.
    /// * `_layer_instance_link` is the `VkLayerInstanceLink` linked list node pointing to the next
    ///   layer.
    /// * `_p_allocator` controls host memory allocation as described in the Memory Allocation
    ///   chapter of the Vulkan spec.
    /// * `_p_instance` points a VkInstance handle in which the resulting instance is returned.
    ///   If the layer implementation wants to handle the call to the next layer, according to
    ///   [the `LLP_LAYER_21` requirement](https://github.com/KhronosGroup/Vulkan-Loader/blob/main/docs/LoaderLayerInterface.md#example-code-for-createinstance:~:text=LLP_LAYER_21,N/A),
    ///   implementations must guarantee that the original pointer should be passed to the lower
    ///   layers.
    ///
    /// # Examples
    /// A layer that always enables the `VK_KHR_surface` extension.
    /// ```
    /// use ash::vk;
    /// use std::{ffi::CStr, ptr::null};
    /// use vulkan_layer::{GlobalHooks, LayerResult, VkLayerInstanceLink};
    ///
    /// struct MyGlobalHooks;
    ///
    /// impl GlobalHooks for MyGlobalHooks {
    ///     fn create_instance(
    ///         &self,
    ///         p_create_info: &vk::InstanceCreateInfo,
    ///         layer_instance_link: &VkLayerInstanceLink,
    ///         p_allocator: Option<&vk::AllocationCallbacks>,
    ///         p_instance: *mut vk::Instance,
    ///     ) -> LayerResult<ash::prelude::VkResult<()>> {
    ///         // Clone the input enabled extension names as a Vec.
    ///         let mut enabled_extensions = if p_create_info.enabled_extension_count == 0 {
    ///             vec![]
    ///         } else {
    ///             unsafe {
    ///                 std::slice::from_raw_parts(
    ///                     p_create_info.pp_enabled_extension_names,
    ///                     p_create_info.enabled_extension_count as usize,
    ///                 )
    ///             }
    ///             .to_vec()
    ///         };
    ///         // Push the VK_KHR_surface extension name only if it doesn't exist.
    ///         if enabled_extensions
    ///             .iter()
    ///             .find(|extension_name| {
    ///                 (unsafe { CStr::from_ptr(**extension_name) })
    ///                     == ash::extensions::khr::Win32Surface::name()
    ///             })
    ///             .is_none()
    ///         {
    ///             enabled_extensions.push(ash::extensions::khr::Win32Surface::name().as_ptr());
    ///         }
    ///         // Create a new VkInstanceCreateInfo with the new extensions. The passed in
    ///         // VkInstanceCreateInfo is const, so we have to create another VkInstanceCreateInfo
    ///         // to mutate the field we need to avoid possible undefined behavior in Rust.
    ///         let mut create_info = p_create_info.clone();
    ///         create_info.enabled_extension_count = enabled_extensions.len() as u32;
    ///         create_info.pp_enabled_extension_names = enabled_extensions.as_ptr();
    ///         // Convert p_allocator to a raw pointer that can be directly passed to
    ///         // vkCreateInstance.
    ///         let p_allocator = p_allocator
    ///             .map(|allocator| allocator as *const _)
    ///             .unwrap_or_else(null);
    ///         // Find the vkCreateInstance entry of the next layer.
    ///         let create_instance = unsafe {
    ///             (layer_instance_link.pfnNextGetInstanceProcAddr)(
    ///                 vk::Instance::null(),
    ///                 c"vkCreateInstance".as_ptr(),
    ///             )
    ///         };
    ///         let create_instance: vk::PFN_vkCreateInstance = match create_instance {
    ///             Some(fp) => unsafe { std::mem::transmute(fp) },
    ///             None => return LayerResult::Handled(Err(vk::Result::ERROR_INITIALIZATION_FAILED)),
    ///         };
    ///         // Call into the vkCreateInstance of the next layer with the original p_instance
    ///         // pointer.
    ///         LayerResult::Handled(
    ///             unsafe { create_instance(&create_info, p_allocator, p_instance) }.result(),
    ///         )
    ///     }
    /// }
    /// ```
    fn create_instance(
        &self,
        _p_create_info: &vk::InstanceCreateInfo,
        _layer_instance_link: &VkLayerInstanceLink,
        _p_allocator: Option<&vk::AllocationCallbacks>,
        _p_instance: *mut vk::Instance,
    ) -> LayerResult<ash::prelude::VkResult<()>> {
        LayerResult::Unhandled
    }
}

/// A trait for the layer implementation to provide metadata of [`GlobalHooks`] for the layer
/// framework.
///
/// This trait serves as the container of the [`GlobalHooks`] type and provides the list of
/// intercepted
/// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
/// for the layer framework. This trait and [`GlobalHooks`] can be implemented by the same type.
///
/// This trait can also be automatically implemented with the help of the
/// [`auto_globalhooksinfo_impl`][`crate::auto_globalhooksinfo_impl`] attribute macro.
///
/// # Examples
///
/// A layer that needs to intercept the `vkCreateInstance` function.
///
/// ```
/// # use ash::vk;
/// # use vulkan_layer::{
/// #     GlobalHooks, GlobalHooksInfo, LayerResult, LayerVulkanCommand as VulkanCommand,
/// #     VkLayerInstanceLink,
/// # };
/// #
/// struct MyLayerGlobalHooks;
///
/// impl GlobalHooks for MyLayerGlobalHooks {
///     fn create_instance(
///         &self,
///         _p_create_info: &vk::InstanceCreateInfo,
///         _layer_instance_link: &VkLayerInstanceLink,
///         _p_allocator: Option<&vk::AllocationCallbacks>,
///         _p_instance: *mut vk::Instance,
///     ) -> LayerResult<ash::prelude::VkResult<()>> {
///         LayerResult::Unhandled
///     }
/// }
///
/// impl GlobalHooksInfo for MyLayerGlobalHooks {
///     type HooksType = Self;
///     type HooksRefType<'a> = &'a Self;
///
///     fn hooked_commands() -> &'static [VulkanCommand] {
///         &[VulkanCommand::CreateInstance]
///     }
///
///     fn hooks(&self) -> Self::HooksRefType<'_> {
///         self
///     }
/// }
/// ```
pub trait GlobalHooksInfo: Send + Sync {
    /// The underlying [`GlobalHooks`] type that implements the core logic to intercept
    /// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.).
    ///
    /// If a type implements both this trait and the [`GlobalHooks`] trait simultaneously,
    /// [`GlobalHooksInfo::HooksType`] can be set to `Self`.
    type HooksType: GlobalHooks;

    /// A type that can be dereferencing to [`GlobalHooksInfo::HooksType`]. Usually just
    /// `&GlobalHooksInfo::HooksType`.
    ///
    /// This extra associated type allows the layer implementation to use a smart pointer like
    /// [`Arc`][std::sync::Arc] or [`MutexGuard`][std::sync::MutexGuard] to hold the reference.
    ///
    /// # Examples
    ///
    /// A layer implementation that returns a [`Arc`][std::sync::Arc].
    /// ```
    /// # use std::sync::Arc;
    /// # use vulkan_layer::{GlobalHooks, GlobalHooksInfo, LayerVulkanCommand as VulkanCommand};
    /// #
    /// # struct MyLayerGlobalHooks;
    /// #
    /// # impl GlobalHooks for MyLayerGlobalHooks {}
    /// #
    /// struct MyLayerGlobalHooksInfo(Arc<MyLayerGlobalHooks>);
    ///
    /// impl GlobalHooksInfo for MyLayerGlobalHooksInfo {
    ///     type HooksType = MyLayerGlobalHooks;
    ///     type HooksRefType<'a> = Arc<MyLayerGlobalHooks>;
    /// #
    /// #     fn hooked_commands() -> &'static [VulkanCommand] {
    /// #         &[]
    /// #     }
    ///
    ///     // Other required methods omitted.
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         Arc::clone(&self.0)
    ///     }
    /// }
    /// ```
    type HooksRefType<'a>: Deref<Target = Self::HooksType> + 'a
    where
        Self: 'a;

    /// Returns a slice of
    /// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
    /// that the layer implementation needs to intercept.
    ///
    /// Avoid returning commands other than
    /// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.).
    /// Otherwise, those commands may or may not be intercepted.
    ///
    /// Because this function is called in [`Global::default`] to initialize the singleton
    /// [`Global`] instance, implementations must not call into [`Global::default`] directly or
    /// indirectly including:
    /// * [`Layer::global_instance`]: this function will call [`Global::default`] under certain
    ///   circumstances.
    /// * Any [`Global`] static methods: These functions may call into [`Layer::global_instance`] to
    ///   obtain the global singleton.
    /// * Any Vulkan APIs exposed by the Vulkan loader: These functions will almost certainly
    ///   trigger the layer initialization logic.
    fn hooked_commands() -> &'static [VulkanCommand];

    /// Returns the reference of the [`GlobalHooksInfo::HooksType`].
    ///
    /// The layer framework uses the returned value to call into the layer implementation of
    /// intercepted commands.
    fn hooks(&self) -> Self::HooksRefType<'_>;
}

/// A wrapper for
/// [`VkExtensionProperties`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkExtensionProperties.html).
/// Structure specifying an extension properties.
///
/// Can convert to the `ash::vk::ExtensionProperties` C binding with [`From`] or [`Into`].
///
/// # Examples
///
/// ```
/// use ash::vk;
/// use std::ffi::CStr;
/// use vulkan_layer::{Extension, ExtensionProperties};
///
/// let extension_properties = ExtensionProperties {
///     name: Extension::KHRSurface,
///     spec_version: 25,
/// };
/// let extension_properties_ffi: vk::ExtensionProperties = extension_properties.clone().into();
/// assert_eq!(
///     unsafe { CStr::from_ptr(extension_properties_ffi.extension_name.as_ptr()) },
///     ash::vk::KhrSurfaceFn::name()
/// );
/// assert_eq!(extension_properties_ffi.spec_version, 25);
/// ```
#[derive(Clone)]
pub struct ExtensionProperties {
    /// The name of the extension.
    pub name: Extension,

    /// `spec_version` is the version of this extension. It is an integer, incremented with
    /// backward compatible changes.
    pub spec_version: u32,
}

impl From<ExtensionProperties> for vk::ExtensionProperties {
    fn from(ExtensionProperties { name, spec_version }: ExtensionProperties) -> Self {
        let name: &str = name.into();
        let name = CString::new(name).unwrap();
        let byte_len = name.as_bytes().len();
        let name = unsafe { std::slice::from_raw_parts(name.as_ptr(), byte_len) };
        let mut res = Self::builder()
            .extension_name([0; vk::MAX_EXTENSION_NAME_SIZE])
            .spec_version(spec_version)
            .build();
        res.extension_name[0..byte_len].copy_from_slice(name);
        res
    }
}

/// A Rust bindings of the
/// [layer manifest file](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#layer-manifest-file-format).
///
/// The return type of [`Layer::manifest`]. This type is marked as `non_exhaustive`, because it will
/// evolve with the [layer manifest schema](https://github.com/LunarG/VulkanTools/blob/v1.3.261/vkconfig_core/layers/layers_schema.json).
/// The user should initialize the value of this type in a way that will still compile if new fields
/// are added. [`LayerManifest::default`] will initialize all fields to empty.
/// ```
/// # use ash::vk;
/// # use vulkan_layer::LayerManifest;
/// let mut manifest = LayerManifest::default();
/// manifest.name = "VK_LAYER_VENDOR_rust_example";
/// manifest.spec_version = vk::API_VERSION_1_1;
/// ```
// TODO: add the capability to serialize to json.
#[non_exhaustive]
#[derive(Default, Clone)]
pub struct LayerManifest {
    /// The string used to uniquely identify this layer to applications.
    ///
    /// The layer name should follow the
    /// [layer name convention](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#layer-conventions-and-rules)
    /// according to
    /// [`LLP_LAYER_3`](<https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#requirements-of-well-behaved-layers:~:text=Conventions%20and%20Rules-,LLP_LAYER_3,-Any%20new%20layer>).
    pub name: &'static str,

    /// The `VkLayerProperties::specVersion` field. The `"api_version"` JSON node.
    ///
    /// The major.minor.patch version number of the Vulkan API that the layer supports encoded as
    /// described in
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#extendingvulkan-coreversions-versionnumbers>.
    /// It does not require the application to make use of that API version. It simply is an
    /// indication that the layer can support Vulkan API instance and device functions up to and
    /// including that API version.
    pub spec_version: u32,

    /// The version of the layer implemented.
    ///
    /// It is an integer, increasing with backward compatible changes. If the layer itself has any
    /// major changes, this number should change so the loader and/or application can identify it
    /// properly.
    pub implementation_version: u32,

    /// A high-level description of the layer and its intended use which provides additional
    /// details that can be used by the application to identify the layer.
    ///
    /// The description shouldn't be longer than `VK_MAX_DESCRIPTION_SIZE` bytes after encoded as a
    /// null-terminated UTF-8 string. Otherwise, the layer framework will panic at runtime.
    pub description: &'static str,

    /// Contains the list of device extension names supported by this layer.
    ///
    /// An array of one or more elements is required if any device extensions are supported by a
    /// layer; otherwise the array should be empty. In `vkCreateDevice`, the layer framework
    /// removes the extensions mentioned here from the
    /// `VkDeviceCreateInfo::ppEnabledExtensionNames` list.
    pub device_extensions: &'static [ExtensionProperties],
}

/// The [`Layer`] trait provides all layer implementation information for the layer framework.
///
/// The type that implements [`Layer`] provides the following functionalities:
/// * Global initialization for the layer.
/// * The factory of [`LayerManifest`], [`InstanceInfo`], and [`DeviceInfo`].
/// * The container of [`GlobalHooksInfo`].
/// * Providing the storage of [`Global`].
/// * Overriding the intercepted Vulkan commands at runtime.
///
/// # Initialization
///
/// The layer implementation can rely on `Layer::default` to perform the global initialization. e.g.
/// initialize the logger.
///
/// The layer framework guarantees that `Layer::default` is only called once when
/// [`Global::default`] is called, and won't be called anywhere else. The layer framework doesn't
/// call [`Global::default`] to initialize. Instead, [`Layer::global_instance`] is called instead on
/// every Vulkan function entry when the global singleton is needed, and the layer implementation is
/// expected to lazily create the static [`Global`] singleton with [`Global::default`] in
/// [`Layer::global_instance`]. Therefore, if the layer implementation can guarantee that
/// [`Global::default`] is called once during the lifetime of the whole process, it is also
/// guaranteed that `Layer::default` is called once during the lifetime of the whole process. Note
/// that it may be difficult to realize such guarantee, because the Vulkan loader my load and unload
/// the layer library multiple times, and the application my load and unload the Vulkan loader
/// multiple times.
///
/// `Layer::default` must not call into any [`Global`] methods and any Vulkan commands exported by
/// the Vulkan loader to avoid an endless recursion.
///
/// # Global clean up
///
/// It is recommended that the implementor of the [`Layer`] trait should have a no-op drop if all
/// `VkInstance` intercepted by the layer is destroyed. If so, [`Global`] has a no-op drop when all
/// `VkInstance` is destroyed. If the [`Layer`] implementor is always
/// trivially-destructible[^trivial_dtor], it's great.
///
/// It is guaranteed that when `Global::drop` is called, `Layer::drop` is called exactly once.
/// However, as a dynamic link library that may be loaded and unloaded to/from the process multiple
/// times, there is no easy way to guarantee that `Global::drop` is called on every unload. In
/// additoon, in Rust, static items do not call drop at the end of the program[^static_drop]. We
/// can't expect drop to be called on dynamic link library unload as well. Therefore, we should
/// avoid using `Layer::drop` to do clean-up to avoid resource leak.
///
/// If it is really necessary to store some not trivially-destructible types(note that all Rust
/// [`std::collections`] are not trivially-destructible) in side the type that implements [`Layer`],
/// the following techniques may be helpful.
///
/// One way to test if a type `T` leaks heap memory is to run Miri tests with [`std::mem::forget`].
///
/// ```
/// #[derive(Default)]
/// struct TriviallyDestructible;
///
/// #[derive(Default)]
/// struct NotTriviallyDestructible(Box<u32>);
///
/// // This passes the Miri test, so TriviallyDestructible won't leak heap memory if drop is not
/// // called.
/// {
///     let x: TriviallyDestructible = Default::default();
///     std::mem::forget(x);
/// }
///
/// // This fails the Miri test, so NotTriviallyDestructible will leak heap memory if drop is not
/// // called.
/// {
///     let x: NotTriviallyDestructible = Default::default();
///     std::mem::forget(x);
/// }
/// ```
///
/// The drop of Rust [`std::collections`] is not no-op even if the collection is empty, so avoid
/// using them directly in the type that implements [`Layer`].
///
/// The drop of [`std::sync::Weak`] is not no-op even if all strong references to the underlying
/// object don't exist, so we should avoid using it to construct a type that needs a no-op drop.
///
/// The drop of [`Option::None`] is no-op even if the underlying `T` is not trivially destructible,
/// and the drop of [`Mutex`][std::sync::Mutex] is no-op if the drop of the wrapped `T` is no-op.
/// Therefore `Mutex<Option<T>>` is a good basic block to build a type that needs a no-op drop.
///
/// # Examples
///
/// A simple layer that prints "Hello from the Rust Vulkan layer!" to `stdout` on every
/// `vkCreateDevice`.
/// ```
/// use ash::{self, vk};
/// use once_cell::sync::Lazy;
/// use std::sync::Arc;
/// use vulkan_layer::{
///     Global, Layer, LayerManifest, StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
/// };
///
/// #[derive(Default)]
/// struct MyLayer(StubGlobalHooks);
///
/// impl Layer for MyLayer {
///     type GlobalHooksInfo = StubGlobalHooks;
///     type InstanceInfo = StubInstanceInfo;
///     type DeviceInfo = StubDeviceInfo;
///     type InstanceInfoContainer = StubInstanceInfo;
///     type DeviceInfoContainer = StubDeviceInfo;
///
///     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
///         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
///         &*GLOBAL
///     }
///
///     fn manifest() -> LayerManifest {
///         let mut manifest = LayerManifest::default();
///         manifest.name = "VK_LAYER_VENDOR_rust_example";
///         manifest.spec_version = vk::API_VERSION_1_1;
///         manifest
///     }
///
///     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
///         &self.0
///     }
///
///     fn create_instance_info(
///         &self,
///         _: &vk::InstanceCreateInfo,
///         _: Option<&vk::AllocationCallbacks>,
///         _: Arc<ash::Instance>,
///         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
///     ) -> Self::InstanceInfoContainer {
///         Default::default()
///     }
///
///     fn create_device_info(
///         &self,
///         _: vk::PhysicalDevice,
///         _: &vk::DeviceCreateInfo,
///         _: Option<&vk::AllocationCallbacks>,
///         _: Arc<ash::Device>,
///         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
///     ) -> Self::DeviceInfoContainer {
///         println!("Hello from the Rust Vulkan layer!");
///         Default::default()
///     }
/// }
/// ```
///
/// A layer that initializes the logging infrastructure with `env_logger`.
/// ```
/// use ash::{self, vk};
/// use log::info;
/// use once_cell::sync::Lazy;
/// use std::sync::Arc;
/// use vulkan_layer::{
///     Global, Layer, LayerManifest, StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
/// };
///
/// struct MyLayer(StubGlobalHooks);
///
/// impl Default for MyLayer {
///     fn default() -> Self {
///         env_logger::init();
///         Self(Default::default())
///     }
/// }
///
/// impl Layer for MyLayer {
///     type GlobalHooksInfo = StubGlobalHooks;
///     type InstanceInfo = StubInstanceInfo;
///     type DeviceInfo = StubDeviceInfo;
///     type InstanceInfoContainer = StubInstanceInfo;
///     type DeviceInfoContainer = StubDeviceInfo;
///
///     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
///         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
///         &*GLOBAL
///     }
///
///     fn manifest() -> LayerManifest {
///         let mut manifest = LayerManifest::default();
///         manifest.name = "VK_LAYER_VENDOR_rust_example";
///         manifest.spec_version = vk::API_VERSION_1_1;
///         manifest
///     }
///
///     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
///         &self.0
///     }
///
///     fn create_instance_info(
///         &self,
///         _: &vk::InstanceCreateInfo,
///         _: Option<&vk::AllocationCallbacks>,
///         _: Arc<ash::Instance>,
///         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
///     ) -> Self::InstanceInfoContainer {
///         Default::default()
///     }
///
///     fn create_device_info(
///         &self,
///         _: vk::PhysicalDevice,
///         _: &vk::DeviceCreateInfo,
///         _: Option<&vk::AllocationCallbacks>,
///         _: Arc<ash::Device>,
///         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
///     ) -> Self::DeviceInfoContainer {
///         info!("Hello from the Rust Vulkan layer!");
///         Default::default()
///     }
/// }
/// ```
///
/// [^trivial_dtor]: This term is borrowed from
/// [C++](<https://en.cppreference.com/w/cpp/language/destructor#:~:text=since%20C%2B%2B20)-,Trivial%20destructor,-The%20destructor%20for>):
/// if the type itself doesn't have a destructor and the types of all fields are trivially
/// destructible, this type is trivially destructible. Rust has [`std::mem::needs_drop`], but this
/// function doesn't have a strong guarantee.
///
/// [^static_drop]: <https://doc.rust-lang.org/reference/items/static-items.html#:~:text=Static%20items%20do%20not%20call%20drop%20at%20the%20end%20of%20the%20program.>
pub trait Layer: Sync + Default + 'static {
    /// The type that provides information about interception of
    /// [global commands](<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance>).
    ///
    /// If the layer implementation is not interested in intercepting any
    /// [global commands](<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance>),
    /// [`StubGlobalHooks`][crate::StubGlobalHooks] can be used.
    type GlobalHooksInfo: GlobalHooksInfo;

    /// The type that provides information about interception of
    /// [Vulkan instance functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#instance-functions),
    /// [global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
    /// not included.
    ///
    /// If the layer implementation is not interested in intercepting any
    /// [Vulkan instance functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#instance-functions)
    /// ([global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
    /// not included), [`StubInstanceInfo`][crate::StubInstanceInfo] can be used.
    type InstanceInfo: InstanceInfo;

    /// The type that provides information about interception of
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions).
    ///
    /// If the layer implementation is not interested in intercepting any
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions),
    /// [`StubDeviceInfo`][crate::StubDeviceInfo] can be used.
    type DeviceInfo: DeviceInfo;

    /// The type that holds a [`Layer::InstanceInfo`] type. Usually just `Self::InstanceInfo`.
    ///
    /// This extra associated type allows the layer to use a smart pointer like [`Arc`] or [`Box`]
    /// to hold the [`Layer::InstanceInfo`].
    ///
    /// # Examples
    ///
    /// Use [`Arc`] to hold the [`Layer::InstanceInfo`].
    /// ```
    /// use ash::vk;
    /// use once_cell::sync::Lazy;
    /// use std::sync::Arc;
    /// use vulkan_layer::{
    ///     Global, Layer, LayerManifest, StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
    /// };
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     type InstanceInfo = StubInstanceInfo;
    ///     type InstanceInfoContainer = Arc<StubInstanceInfo>;
    ///
    ///     fn create_instance_info(
    ///         &self,
    ///         _: &vk::InstanceCreateInfo,
    ///         _: Option<&vk::AllocationCallbacks>,
    ///         _: Arc<ash::Instance>,
    ///         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ///     ) -> Self::InstanceInfoContainer {
    ///         Arc::default()
    ///     }
    ///
    ///     // Unrelated required items are omitted.
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    /// #     type DeviceInfo = StubDeviceInfo;
    /// #     type DeviceInfoContainer = StubDeviceInfo;
    /// #
    /// #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    /// #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    /// #         &*GLOBAL
    /// #     }
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    /// #
    /// #     fn create_device_info(
    /// #         &self,
    /// #         _: vk::PhysicalDevice,
    /// #         _: &vk::DeviceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Device>,
    /// #         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    /// #     ) -> Self::DeviceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// }
    /// ```
    type InstanceInfoContainer: Borrow<Self::InstanceInfo> + Sync + Send;

    /// The type that holds a [`Layer::DeviceInfo`] type. Usually just `Self::DeviceInfo`.
    ///
    /// This extra associated type allows the layer to use a smart pointer like [`Arc`] to hold the
    /// [`Layer::DeviceInfo`].
    ///
    /// # Examples
    ///
    /// Use [`Arc`] to hold the [`Layer::DeviceInfo`].
    /// ```
    /// use ash::vk;
    /// use once_cell::sync::Lazy;
    /// use std::sync::Arc;
    /// use vulkan_layer::{
    ///     Global, Layer, LayerManifest, StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
    /// };
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     type DeviceInfo = StubDeviceInfo;
    ///     type DeviceInfoContainer = Arc<StubDeviceInfo>;
    ///
    ///     fn create_device_info(
    ///         &self,
    ///         _: vk::PhysicalDevice,
    ///         _: &vk::DeviceCreateInfo,
    ///         _: Option<&vk::AllocationCallbacks>,
    ///         _: Arc<ash::Device>,
    ///         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ///     ) -> Self::DeviceInfoContainer {
    ///         Arc::default()
    ///     }
    ///
    ///     // Unrelated required items are omitted.
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    /// #     type InstanceInfo = StubInstanceInfo;
    /// #     type InstanceInfoContainer = StubInstanceInfo;
    /// #
    /// #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    /// #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    /// #         &*GLOBAL
    /// #     }
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    /// #
    /// #     fn create_instance_info(
    /// #         &self,
    /// #         _: &vk::InstanceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Instance>,
    /// #         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    /// #     ) -> Self::InstanceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// }
    /// ```
    type DeviceInfoContainer: Borrow<Self::DeviceInfo> + Sync + Send;

    /// Returns the
    /// [layer manifest](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#layer-manifest-file-format)
    /// to the layer framework.
    ///
    /// Implementors should always return the same value during the lifetime of the process, and
    /// must match the manifest json file with the layer. Implementors should avoid calling into any
    /// [`Global`] methods because this function can be called to implement Vulkan introspection
    /// queries before any `VkInstance` is created.
    ///
    /// The layer framework will use this value to:
    /// * Implement [Vulkan introspection queries](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderLayerInterface.md#requirements-of-well-behaved-layers:~:text=Parent-,Introspection%20Query,-%22api_version%22),
    ///   including `vkEnumerateInstanceLayerProperties`, `vkEnumerateDeviceLayerProperties`,
    ///   `vkEnumerateInstanceExtensionProperties` and `vkEnumerateDeviceExtensionProperties`.
    /// * Remove the layer device extensions from `VkDeviceCreateInfo::ppEnabledExtensionNames` in
    ///   `vkCreateDevice` if present.
    fn manifest() -> LayerManifest;

    /// Provide the reference to the global singleton of [`Global`].
    ///
    /// Implementors must always return the reference pointing to the same [`Global`] object.
    /// Implementors can use [`LazyLock`][std::sync::LazyLock], [`OnceLock`][std::sync::OnceLock],
    /// or `Lazy` from the `once_cell` crate to implement. Implementors should use
    /// [`Global::default`] to create the [`Global`] object. More information on initialization can
    /// be found [here][Layer#initialization].
    ///
    /// # Examples
    ///
    /// Use `Lazy` provided by the `once_cell` crate to implement.
    /// ```
    /// use ash::{self, vk};
    /// use once_cell::sync::Lazy;
    /// use std::sync::Arc;
    /// use vulkan_layer::{
    ///     Global, Layer, LayerManifest, StubDeviceInfo, StubGlobalHooks, StubInstanceInfo,
    /// };
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    ///         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    ///         &*GLOBAL
    ///     }
    ///
    ///     // Unrelated required items are omitted.
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    /// #     type InstanceInfo = StubInstanceInfo;
    /// #     type DeviceInfo = StubDeviceInfo;
    /// #     type InstanceInfoContainer = StubInstanceInfo;
    /// #     type DeviceInfoContainer = StubDeviceInfo;
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         let mut manifest = LayerManifest::default();
    /// #         manifest.name = "VK_LAYER_VENDOR_rust_example";
    /// #         manifest.spec_version = vk::API_VERSION_1_1;
    /// #         manifest
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    /// #
    /// #     fn create_instance_info(
    /// #         &self,
    /// #         _: &vk::InstanceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Instance>,
    /// #         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    /// #     ) -> Self::InstanceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn create_device_info(
    /// #         &self,
    /// #         _: vk::PhysicalDevice,
    /// #         _: &vk::DeviceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Device>,
    /// #         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    /// #     ) -> Self::DeviceInfoContainer {
    /// #         println!("Hello from the Rust Vulkan layer!");
    /// #         Default::default()
    /// #     }
    /// }
    /// ```
    fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static;

    /// Returns a reference of the underlying [`GlobalHooksInfo`] object.
    fn global_hooks_info(&self) -> &Self::GlobalHooksInfo;

    /// Returns a reference of the underlying [`GlobalHooks`] object.
    ///
    /// The layer framework relies on this function to obtain [`GlobalHooks`], and implementors
    /// should avoid overriding this method.
    fn global_hooks(&self) -> <Self::GlobalHooksInfo as GlobalHooksInfo>::HooksRefType<'_> {
        self.global_hooks_info().hooks()
    }

    /// The factory method for the [`InstanceInfo`] type.
    ///
    /// This function is called by the layer framework in `vkCreateInstance`, after the
    /// `vkCreateInstance` of the next layer returns with success.
    ///
    /// # Arguments
    ///
    /// * `create_info` is a pointer to a VkInstanceCreateInfo structure controlling creation of the
    ///   instance. The [`VkLayerInstanceLink`] linked list in the `pNext` chain is already advanced
    ///   by the layer framework. This is the original `VkInstanceCreateInfo` passed in by the
    ///   application or the previous layer(except the advanced [`VkLayerInstanceLink`] linked
    ///   list), even if the layer implementation passes a different `VkInstanceCreateInfo` to the
    ///   next layer in [`GlobalHooks::create_instance`].
    /// * `allocator` controls host memory allocation as described in the [Memory Allocation](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#memory-allocation)
    ///   chapter of the Vulkan spec.
    /// * `instance` contains the loaded instance dispatch table of the next layer. The layer
    ///   implementation can just clone this [`Arc`] if it is needed to keep the instance dispatch
    ///   table.
    /// * `next_get_instance_proc_addr` is the `vkGetInstanceProcAddr` function pointer of the next
    ///   layer obtained from the [`VkLayerInstanceLink`] linked list.
    fn create_instance_info(
        &self,
        create_info: &vk::InstanceCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
        instance: Arc<ash::Instance>,
        next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ) -> Self::InstanceInfoContainer;

    /// The factory method for the [`DeviceInfo`] type.
    ///
    /// This function is called by the layer framework in `vkCreateDevice`, after the
    /// `vkCreateDevice` of the next layer returns with success.
    ///
    /// # Arguments
    /// * `physical_device` is one of the device handles returned from a call to
    ///   `vkEnumeratePhysicalDevices` that is passed to `vkCreateDevice` by the application or the
    ///   previous layer.
    /// * `create_info` is a pointer to a `VkDeviceCreateInfo` structure containing information
    ///   about how to create the device. The [`VkLayerDeviceLink`][crate::VkLayerDeviceLink] linked
    ///   list in the `pNext` chain is already advanced by the layer framework. This is the original
    ///   `VkDeviceCreateInfo` passed in by the application or the previous layer(except the
    ///   advanced [`VkLayerDeviceLink`][crate::VkLayerDeviceLink] linked list), even if the layer
    ///   implementation passes a different `VkDeviceCreateInfo` to the next layer in
    ///   [`InstanceHooks::create_device`].
    /// * `allocator` controls host memory allocation as described in the [Memory Allocation](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#memory-allocation)
    ///   chapter of the Vulkan spec.
    /// * `device` contains the loaded device dispatch table of the next layer. The layer
    ///   implementation can just clone this [`Arc`] if it is needed to keep the device dispatch
    ///   table.
    /// * `next_get_device_proc_addr` is the `vkGetDeviceProcAddr` function pointer of the next
    ///   layer obtained from the [`VkLayerDeviceLink`][crate::VkLayerDeviceLink] linked list.
    fn create_device_info(
        &self,
        physical_device: vk::PhysicalDevice,
        create_info: &vk::DeviceCreateInfo,
        allocator: Option<&vk::AllocationCallbacks>,
        device: Arc<ash::Device>,
        next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ) -> Self::DeviceInfoContainer;

    /// Returns an iterator of
    /// [Vulkan instance functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#instance-functions)
    /// ([global commands](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html#:~:text=The%20global%20commands%20are%3A%20vkEnumerateInstanceVersion%2C%20vkEnumerateInstanceExtensionProperties%2C%20vkEnumerateInstanceLayerProperties%2C%20and%20vkCreateInstance.)
    /// not included) that the layer implementation needs to intercept.
    ///
    /// This function allows the layer implementation to decide the commands to intercept
    /// dynamically after the `VkInstance` is created. By default it just returns all commands
    /// returned by [`InstanceInfo::hooked_commands`].
    ///
    /// # Arguments
    /// * `_instance_info` is the relevant instance.
    ///
    /// # Examples
    ///
    /// A layer that intercepts `vkDestroySurfaceKHR` only if the `VK_KHR_win32_surface` extension
    /// is enabled.
    /// ```
    /// use ash::vk;
    /// use once_cell::sync::Lazy;
    /// use std::{ffi::CStr, sync::Arc};
    /// use vulkan_layer::{
    ///     Global, InstanceHooks, InstanceInfo, Layer, LayerManifest, LayerResult,
    ///     LayerVulkanCommand as VulkanCommand, StubDeviceInfo, StubGlobalHooks,
    /// };
    ///
    /// struct MyLayerInstanceInfo {
    ///     is_win32_surface_enabled: bool,
    /// }
    ///
    /// impl InstanceHooks for MyLayerInstanceInfo {
    ///     fn destroy_surface_khr(
    ///         &self,
    ///         _surface: vk::SurfaceKHR,
    ///         _p_allocator: Option<&vk::AllocationCallbacks>,
    ///     ) -> LayerResult<()> {
    ///         LayerResult::Unhandled
    ///     }
    /// }
    ///
    /// impl InstanceInfo for MyLayerInstanceInfo {
    ///     type HooksType = Self;
    ///     type HooksRefType<'a> = &'a Self;
    ///
    ///     fn hooked_commands() -> &'static [VulkanCommand] {
    ///         &[VulkanCommand::DestroySurfaceKhr]
    ///     }
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         self
    ///     }
    /// }
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     // ...
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    ///     type InstanceInfo = MyLayerInstanceInfo;
    /// #     type DeviceInfo = StubDeviceInfo;
    ///     type InstanceInfoContainer = MyLayerInstanceInfo;
    /// #     type DeviceInfoContainer = StubDeviceInfo;
    /// #
    /// #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    /// #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    /// #         &*GLOBAL
    /// #     }
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    ///
    ///     fn create_instance_info(
    ///         &self,
    ///         create_info: &vk::InstanceCreateInfo,
    ///         _: Option<&vk::AllocationCallbacks>,
    ///         _: Arc<ash::Instance>,
    ///         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ///     ) -> Self::InstanceInfoContainer {
    ///         let enabled_extensions = if create_info.enabled_extension_count > 0 {
    ///             unsafe {
    ///                 std::slice::from_raw_parts(
    ///                     create_info.pp_enabled_extension_names,
    ///                     create_info.enabled_extension_count as usize,
    ///                 )
    ///             }
    ///         } else {
    ///             &[]
    ///         };
    ///         let is_win32_surface_enabled = enabled_extensions
    ///             .iter()
    ///             .find(|extension_name| {
    ///                 (unsafe { CStr::from_ptr(**extension_name) })
    ///                     == ash::extensions::khr::Win32Surface::name()
    ///             })
    ///             .is_some();
    ///         MyLayerInstanceInfo {
    ///             is_win32_surface_enabled,
    ///         }
    ///     }
    ///
    ///     fn hooked_instance_commands(
    ///         &self,
    ///         instance_info: &Self::InstanceInfo,
    ///     ) -> Box<dyn Iterator<Item = VulkanCommand>> {
    ///         let should_hook_destroy_surface = instance_info.is_win32_surface_enabled;
    ///         Box::new(
    ///             Self::InstanceInfo::hooked_commands()
    ///                 .iter()
    ///                 .cloned()
    ///                 .filter(move |command| match command {
    ///                     VulkanCommand::DestroySurfaceKhr => should_hook_destroy_surface,
    ///                     _ => true,
    ///                 }),
    ///         )
    ///     }
    /// #
    /// #     fn create_device_info(
    /// #         &self,
    /// #         _: vk::PhysicalDevice,
    /// #         _: &vk::DeviceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Device>,
    /// #         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    /// #     ) -> Self::DeviceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// }
    /// ```
    fn hooked_instance_commands(
        &self,
        _instance_info: &Self::InstanceInfo,
    ) -> Box<dyn Iterator<Item = VulkanCommand>> {
        Box::new(Self::InstanceInfo::hooked_commands().iter().cloned())
    }

    /// Returns an iterator of
    /// [Vulkan device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions)
    /// that the layer implementation needs to intercept.
    ///
    /// This function allows the layer implementation to decide the commands to intercept
    /// dynamically after the `VkInstance` or `VkDevice` is created. By default it just returns all
    /// commands returned by [`DeviceInfo::hooked_commands`].
    ///
    /// Arguments
    /// * `_instance_info` is the relevant instance.
    /// * `_device_info` is an optional relevant device. If `_device_info` is [`None`], the layer
    ///   framework is querying all possible intercepted
    ///   [device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions)
    ///   to implement `vkGetInstanceProcAddr` with the `pName` parameter referring to a
    ///   [device functions](https://github.com/KhronosGroup/Vulkan-Loader/blob/v1.3.261/docs/LoaderInterfaceArchitecture.md#device-functions),
    ///   i.e. implementations must guarantee that the returned commands with a [`None`]
    ///   `_device_info` is the superset of all possible `_device_info`. Otherwise, the application
    ///   that just uses the function pointers returned by `vkGetInstanceProcAddr` may skip the
    ///   current layer.
    ///
    /// # Examples
    ///
    /// A layer that intercepts the `vkCreateImage` only if the ASTC LDR feature is enabled.
    /// ```
    /// use ash::{self, prelude::VkResult, vk};
    /// use once_cell::sync::Lazy;
    /// use std::sync::Arc;
    /// use vulkan_layer::{
    ///     DeviceHooks, DeviceInfo, Global, Layer, LayerManifest, LayerResult,
    ///     LayerVulkanCommand as VulkanCommand, StubGlobalHooks, StubInstanceInfo,
    ///     VulkanBaseInStructChain,
    /// };
    ///
    /// struct MyLayerDeviceInfo {
    ///     is_astc_enabled: bool,
    /// }
    ///
    /// impl DeviceHooks for MyLayerDeviceInfo {
    ///     fn create_image(
    ///         &self,
    ///         create_info: &vk::ImageCreateInfo,
    ///         _p_allocator: Option<&vk::AllocationCallbacks>,
    ///     ) -> LayerResult<VkResult<vk::Image>> {
    ///         if create_info.format == vk::Format::ASTC_4X4_UNORM_BLOCK {
    ///             println!("ASTC 4x4 UNORM image created.");
    ///         }
    ///         LayerResult::Unhandled
    ///     }
    /// }
    ///
    /// impl DeviceInfo for MyLayerDeviceInfo {
    ///     type HooksType = Self;
    ///     type HooksRefType<'a> = &'a Self;
    ///
    ///     fn hooked_commands() -> &'static [VulkanCommand] {
    ///         &[VulkanCommand::CreateImage]
    ///     }
    ///
    ///     fn hooks(&self) -> Self::HooksRefType<'_> {
    ///         self
    ///     }
    /// }
    ///
    /// #[derive(Default)]
    /// struct MyLayer(StubGlobalHooks);
    ///
    /// impl Layer for MyLayer {
    ///     // ...
    /// #     type GlobalHooksInfo = StubGlobalHooks;
    /// #     type InstanceInfo = StubInstanceInfo;
    ///     type DeviceInfo = MyLayerDeviceInfo;
    /// #     type InstanceInfoContainer = StubInstanceInfo;
    ///     type DeviceInfoContainer = MyLayerDeviceInfo;
    /// #
    /// #     fn global_instance() -> impl std::ops::Deref<Target = Global<Self>> + 'static {
    /// #         static GLOBAL: Lazy<Global<MyLayer>> = Lazy::new(Default::default);
    /// #         &*GLOBAL
    /// #     }
    /// #
    /// #     fn manifest() -> LayerManifest {
    /// #         Default::default()
    /// #     }
    /// #
    /// #     fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
    /// #         &self.0
    /// #     }
    /// #
    /// #     fn create_instance_info(
    /// #         &self,
    /// #         _: &vk::InstanceCreateInfo,
    /// #         _: Option<&vk::AllocationCallbacks>,
    /// #         _: Arc<ash::Instance>,
    /// #         _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    /// #     ) -> Self::InstanceInfoContainer {
    /// #         Default::default()
    /// #     }
    /// #
    ///     fn create_device_info(
    ///         &self,
    ///         _: vk::PhysicalDevice,
    ///         create_info: &vk::DeviceCreateInfo,
    ///         _: Option<&vk::AllocationCallbacks>,
    ///         _: Arc<ash::Device>,
    ///         _next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ///     ) -> Self::DeviceInfoContainer {
    ///         let mut p_next_chain: VulkanBaseInStructChain =
    ///             unsafe { (create_info.p_next as *const vk::BaseInStructure).as_ref() }.into();
    ///         let is_astc_enabled = p_next_chain
    ///             .find_map(|p_next| {
    ///                 let p_next = p_next as *const vk::BaseInStructure;
    ///                 unsafe {
    ///                     ash::match_in_struct!(match p_next {
    ///                         features2 @ vk::PhysicalDeviceFeatures2 => {
    ///                             Some(&features2.features)
    ///                         }
    ///                         _ => {
    ///                             None
    ///                         }
    ///                     })
    ///                 }
    ///             })
    ///             .or_else(|| unsafe { create_info.p_enabled_features.as_ref() })
    ///             .map(|features| features.texture_compression_astc_ldr == vk::TRUE)
    ///             .unwrap_or(false);
    ///         MyLayerDeviceInfo { is_astc_enabled }
    ///     }
    ///
    ///     fn hooked_device_commands(
    ///         &self,
    ///         _instance_info: &Self::InstanceInfo,
    ///         device_info: Option<&Self::DeviceInfo>,
    ///     ) -> Box<dyn Iterator<Item = VulkanCommand>> {
    ///         let should_hook_create_image = device_info
    ///             .map(|device_info| device_info.is_astc_enabled)
    ///             // Always hook the vkCreateImage function in the function pointers returned by
    ///             // vkGetInstanceProcAddr: we don't know if the to-be-created VkDevice will be
    ///             // created with the ASTC feature enabled.
    ///             .unwrap_or(true);
    ///         Box::new(
    ///             Self::DeviceInfo::hooked_commands()
    ///                 .iter()
    ///                 .cloned()
    ///                 .filter(move |command| match command {
    ///                     VulkanCommand::CreateImage => should_hook_create_image,
    ///                     _ => true,
    ///                 }),
    ///         )
    ///     }
    /// }
    /// ```
    fn hooked_device_commands(
        &self,
        _instance_info: &Self::InstanceInfo,
        _device_info: Option<&Self::DeviceInfo>,
    ) -> Box<dyn Iterator<Item = VulkanCommand>> {
        Box::new(Self::DeviceInfo::hooked_commands().iter().cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vulkan_command_try_from_should_end_up_error_with_unknown_commands() {
        let unknown_command = "vkTestCommandTEST";
        let err = VulkanCommand::try_from(unknown_command).unwrap_err();
        assert!(err.to_string().contains(unknown_command));
    }
}
