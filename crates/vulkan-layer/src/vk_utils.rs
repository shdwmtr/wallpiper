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

use std::{collections::BTreeSet, ffi::CStr, fmt::Debug, mem::MaybeUninit, ptr::NonNull};

use ash::vk;
use num_traits::Zero;

use crate::global_simple_intercept::{ApiVersion, Extension, Feature};

/// A chain of
/// [`VkBaseOutStructure`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkBaseOutStructure.html).
///
/// This type provides an easy way to visit the `*mut c_void`
/// [`pNext` chain](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#fundamentals-validusage-pNext).
/// One can convert an `Option<&mut vk::BaseOutStructure>` to [`VulkanBaseOutStructChain`].
///
/// # Examples
///
/// ```
/// use ash::vk;
/// use vulkan_layer::VulkanBaseOutStructChain;
///
/// let mut timeline_semaphore_feature = vk::PhysicalDeviceTimelineSemaphoreFeatures::builder();
/// let mut ycbcr_feature = vk::PhysicalDeviceSamplerYcbcrConversionFeatures::builder();
/// let features2 = vk::PhysicalDeviceFeatures2::builder()
///     .push_next(&mut timeline_semaphore_feature)
///     .push_next(&mut ycbcr_feature);
/// let out_chain = features2.p_next.cast::<vk::BaseOutStructure>();
/// let mut out_chain: VulkanBaseOutStructChain = unsafe { out_chain.as_mut() }.into();
/// // `push_next` prepends the element to the pNext chain.
/// assert_eq!(
///     out_chain.next().unwrap().s_type,
///     vk::StructureType::PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES
/// );
/// assert_eq!(
///     out_chain.next().unwrap().s_type,
///     vk::StructureType::PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES
/// );
/// assert!(out_chain.next().is_none());
/// ```
pub struct VulkanBaseOutStructChain<'a> {
    next: Option<&'a mut vk::BaseOutStructure>,
}

impl<'a> From<Option<&'a mut vk::BaseOutStructure>> for VulkanBaseOutStructChain<'a> {
    fn from(next: Option<&'a mut vk::BaseOutStructure>) -> Self {
        Self { next }
    }
}

impl<'a> Iterator for VulkanBaseOutStructChain<'a> {
    type Item = &'a mut vk::BaseOutStructure;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().inspect(|element| {
            self.next = unsafe { element.p_next.as_mut() };
        })
    }
}

/// A chain of
/// [`VkBaseInStructure`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VkBaseInStructure.html).
///
/// This type provides an easy way to visit the `*const c_void`
/// [`pNext` chain](https://registry.khronos.org/vulkan/specs/1.3-extensions/html/vkspec.html#fundamentals-validusage-pNext).
/// One can convert an `Option<&vk::BaseInStructure>` to [`VulkanBaseInStructChain`].
///
/// # Examples
/// ```
/// use ash::vk;
/// use vulkan_layer::VulkanBaseInStructChain;
///
/// let mut external_memory_image_info = vk::ExternalMemoryImageCreateInfo::builder();
/// let mut image_format_list_info = vk::ImageFormatListCreateInfo::builder();
/// let image_create_info = vk::ImageCreateInfo::builder()
///     .push_next(&mut external_memory_image_info)
///     .push_next(&mut image_format_list_info);
/// let in_chain = image_create_info.p_next.cast::<vk::BaseInStructure>();
/// let mut in_chain: VulkanBaseInStructChain = unsafe { in_chain.as_ref() }.into();
/// // `push_next` prepends the element to the pNext chain.
/// assert_eq!(
///     in_chain.next().unwrap().s_type,
///     vk::StructureType::IMAGE_FORMAT_LIST_CREATE_INFO
/// );
/// assert_eq!(
///     in_chain.next().unwrap().s_type,
///     vk::StructureType::EXTERNAL_MEMORY_IMAGE_CREATE_INFO
/// );
/// assert!(in_chain.next().is_none());
/// ```
pub struct VulkanBaseInStructChain<'a> {
    next: Option<&'a vk::BaseInStructure>,
}

impl<'a> From<Option<&'a vk::BaseInStructure>> for VulkanBaseInStructChain<'a> {
    fn from(next: Option<&'a vk::BaseInStructure>) -> Self {
        Self { next }
    }
}

impl<'a> Iterator for VulkanBaseInStructChain<'a> {
    type Item = &'a vk::BaseInStructure;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().inspect(|element| {
            self.next = unsafe { element.p_next.as_ref() };
        })
    }
}

/// Used to extend an iterator of [`Feature`].
pub trait IsCommandEnabled {
    /// Given a list of [`Feature`]s where the command is defined, the Vulkan API version and a set
    /// of enabled extensions, returns if the command is enabled.
    ///
    /// One command with the same name can appear in different features, e.g.
    /// [`vkCmdBindVertexBuffers2EXT`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkCmdBindVertexBuffers2EXT.html)
    /// is defined in both
    /// [`VK_EXT_extended_dynamic_state`](<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_EXT_extended_dynamic_state.html#:~:text=New%20Commands-,vkCmdBindVertexBuffers2EXT,-vkCmdSetCullModeEXT>)
    /// and
    /// [`VK_EXT_shader_object`](<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_EXT_shader_object.html#:~:text=vkCmdBindShadersEXT-,vkCmdBindVertexBuffers2EXT,-vkCmdSetAlphaToCoverageEnableEXT>).
    /// `vkGetDeviceProcAddr` should return a valid function pointer `vkCmdBindVertexBuffers2EXT` if
    /// either extension is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use ash::vk;
    /// use std::collections::BTreeSet;
    /// use vulkan_layer::{
    ///     unstable_api::{ApiVersion, Feature, IsCommandEnabled},
    ///     Extension,
    /// };
    ///
    /// let features: Vec<Feature> = vec![
    ///     Extension::EXTExtendedDynamicState.into(),
    ///     Extension::EXTShaderObject.into(),
    /// ];
    /// assert!(features.is_command_enabled(
    ///     &ApiVersion::V1_1,
    ///     &BTreeSet::from([Extension::EXTExtendedDynamicState.into()])
    /// ));
    /// assert!(features.is_command_enabled(
    ///     &ApiVersion::V1_1,
    ///     &BTreeSet::from([Extension::EXTShaderObject.into()])
    /// ));
    /// assert!(!features.is_command_enabled(&ApiVersion::V1_1, &BTreeSet::new()));
    /// ```
    #[allow(clippy::wrong_self_convention)]
    fn is_command_enabled(
        self,
        api_version: &ApiVersion,
        enabled_extensions: &BTreeSet<Extension>,
    ) -> bool;
}

impl<'a, T: IntoIterator<Item = &'a Feature>> IsCommandEnabled for T {
    fn is_command_enabled(
        self,
        api_version: &ApiVersion,
        enabled_extensions: &BTreeSet<Extension>,
    ) -> bool {
        self.into_iter().any(|feature| match feature {
            Feature::Extension(extension) => enabled_extensions.contains(extension),
            Feature::Core(version) => version <= api_version,
        })
    }
}

/// Similar to [`std::slice::from_raw_parts`], but allow null pointer for 0 length slice.
///
/// # Safety
/// Follow the safety requirements for [`std::slice::from_raw_parts`] with one major difference: if
/// `len` is `0`, `data` can be a null pointer, unlike [`std::slice::from_raw_parts`].
#[deny(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn slice_from_raw_parts<'a, T>(
    data: *const T,
    len: impl TryInto<usize, Error = impl Debug>,
) -> &'a [T] {
    let len: usize = len
        .try_into()
        .expect("length must be able to cast to usize");
    if len == 0 {
        return &[];
    }
    // Safety: since `len` isn't 0 at this point, the caller guarantees that the safety requirement
    // for `std::slice::from_raw_parts` is met.
    unsafe { std::slice::from_raw_parts(data, len) }
}

/// Convert from a slice of i8 pointers to an iterator of strings.
///
/// Usually used to parse the C style C string array like `VkInstanceInfo::ppEnabledExtensionNames`.
///
/// # Safety
/// Every entry of `cstr_ptrs` must meet the safety requirement of [`std::ffi::CStr::from_ptr`].
///
/// # Panic
/// When iterating the returned iterator, it will panic if the pointer doesn't point to a valid
/// UTF-8 string.
#[deny(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn slice_to_owned_strings(
    cstr_ptrs: &[*const i8],
) -> impl Iterator<Item = String> + '_ {
    cstr_ptrs.iter().map(|cstr_ptr| {
        let cstr = unsafe { CStr::from_ptr(*cstr_ptr) };
        cstr.to_str()
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to decode the string {}: {}",
                    cstr.to_string_lossy(),
                    e
                )
            })
            .to_owned()
    })
}

/// Clone the slice to a C style out array with proper `VkResult` return value.
///
/// The caller can use this function to handle the output array in Vulkan with slice.
///
/// If `p_out` is `NULL`, write the length to `p_out`. If `p_out` is not `NULL`, `p_count` describes
/// the length of the output array. This function will clone the slice to the output array, and
/// rewrite the `p_count` to indicate how many elements are written to the out array. If the length
/// of the slice is larger than the out array, `VK_INCOMPLETE` will be returned. For all other
/// cases, `VK_SUCCESS` is returned.
///
/// # Safety
/// `p_count` must be a valid pointer to `U`. If `p_count` doesn't reference 0, and `p_out` is not
/// null, `p_out` must be a valid pointer to `*p_count` number of `T`'s.
///
/// # Examples
///
/// Obtain the length with a NULL `p_out`.
///
/// ```
/// use ash::vk;
/// use std::ptr::NonNull;
/// use vulkan_layer::fill_vk_out_array;
///
/// let mut count = 0;
/// let out_array = &[0, 1, 2, 3];
/// assert_eq!(
///     unsafe {
///         fill_vk_out_array(
///             out_array,
///             NonNull::new(&mut count as *mut _).unwrap(),
///             std::ptr::null_mut(),
///         )
///     },
///     vk::Result::SUCCESS
/// );
/// assert_eq!(count as usize, out_array.len());
/// ```
///
/// Short output array results in a `VK_INCOMPLETE`.
///
/// ```
/// use ash::vk;
/// use std::ptr::NonNull;
/// use vulkan_layer::fill_vk_out_array;
///
/// let mut out_array = [0; 2];
/// let mut count = out_array.len() as u32;
/// let out_slice = &[3, 1, 44];
///
/// assert_eq!(
///     unsafe {
///         fill_vk_out_array(
///             out_slice,
///             NonNull::new(&mut count as *mut _).unwrap(),
///             out_array.as_mut_ptr(),
///         )
///     },
///     vk::Result::INCOMPLETE
/// );
/// assert_eq!(count as usize, out_array.len());
/// assert_eq!(out_array, out_slice[0..out_array.len()]);
/// ```
///
/// Longer output array will only be partly overwritten.
///
/// ```
/// use ash::vk;
/// use std::ptr::NonNull;
/// use vulkan_layer::fill_vk_out_array;
///
/// let mut out_array = [42; 3];
/// let mut count = out_array.len() as u32;
/// let out_slice = &[3, 1];
///
/// assert_eq!(
///     unsafe {
///         fill_vk_out_array(
///             out_slice,
///             NonNull::new(&mut count as *mut _).unwrap(),
///             out_array.as_mut_ptr(),
///         )
///     },
///     vk::Result::SUCCESS
/// );
/// assert_eq!(count as usize, out_slice.len());
/// assert_eq!(out_array, [3, 1, 42]);
/// ```
#[deny(unsafe_op_in_unsafe_fn)]
pub unsafe fn fill_vk_out_array<T, U>(
    out: &[T],
    mut p_count: NonNull<U>,
    p_out: *mut T,
) -> vk::Result
where
    T: Clone,
    U: TryFrom<usize> + TryInto<usize> + Zero + Copy,
    <U as TryFrom<usize>>::Error: Debug,
    <U as TryInto<usize>>::Error: Debug,
{
    let count = unsafe { p_count.as_mut() };
    let p_out = match NonNull::new(p_out) {
        Some(p_out) => p_out,
        None => {
            *count = out.len().try_into().unwrap();
            return vk::Result::SUCCESS;
        }
    };
    if count.is_zero() {
        if out.is_empty() {
            return vk::Result::SUCCESS;
        } else {
            return vk::Result::INCOMPLETE;
        }
    }
    let out_slice =
        unsafe { std::slice::from_raw_parts_mut(p_out.as_ptr(), (*count).try_into().unwrap()) };
    if out_slice.len() < out.len() {
        *count = out_slice.len().try_into().unwrap();
        out_slice.clone_from_slice(&out[..out_slice.len()]);
        vk::Result::INCOMPLETE
    } else {
        *count = out.len().try_into().unwrap();
        out_slice[..out.len()].clone_from_slice(out);
        vk::Result::SUCCESS
    }
}

/// Returns `None` if the pointer is null, or else returns a unique reference to the value wrapped
/// in `Some`. In contrast to `as_mut`, this does not require that the value has to be initialized.
///
/// Polyfill of
/// [`as_uninit_mut`](https://doc.rust-lang.org/std/primitive.pointer.html#method.as_uninit_mut) for
/// stable Rust.
///
/// # Safety
///
/// Refers to
/// [`as_uninit_mut`](https://doc.rust-lang.org/std/primitive.pointer.html#method.as_uninit_mut).
#[deny(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn ptr_as_uninit_mut<'a, T>(p_data: *mut T) -> Option<&'a mut MaybeUninit<T>> {
    // SAFETY: same as https://doc.rust-lang.org/std/primitive.pointer.html#method.as_uninit_mut.
    if p_data.is_null() {
        None
    } else {
        Some(unsafe { &mut *(p_data as *mut MaybeUninit<T>) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multiple_in_struct_nodes() {
        let mut external_memory_image_info = vk::ExternalMemoryImageCreateInfo::builder();
        let mut image_format_list_info = vk::ImageFormatListCreateInfo::builder();
        let image_create_info = vk::ImageCreateInfo::builder()
            .push_next(&mut external_memory_image_info)
            .push_next(&mut image_format_list_info);
        let in_chain = image_create_info.p_next as *const vk::BaseInStructure;
        let mut in_chain: VulkanBaseInStructChain = unsafe { in_chain.as_ref() }.into();
        // `push_next` prepends the element to the pNext chain.
        assert_eq!(
            in_chain.next().unwrap().s_type,
            vk::StructureType::IMAGE_FORMAT_LIST_CREATE_INFO
        );
        assert_eq!(
            in_chain.next().unwrap().s_type,
            vk::StructureType::EXTERNAL_MEMORY_IMAGE_CREATE_INFO
        );
        assert!(in_chain.next().is_none());
    }

    #[test]
    fn multiple_out_struct_nodes() {
        let mut timeline_semaphore_feature = vk::PhysicalDeviceTimelineSemaphoreFeatures::builder();
        let mut ycbcr_feature = vk::PhysicalDeviceSamplerYcbcrConversionFeatures::builder();
        let features2 = vk::PhysicalDeviceFeatures2::builder()
            .push_next(&mut timeline_semaphore_feature)
            .push_next(&mut ycbcr_feature);
        let out_chain = features2.p_next.cast::<vk::BaseOutStructure>();
        let mut out_chain: VulkanBaseOutStructChain = unsafe { out_chain.as_mut() }.into();
        // `push_next` prepends the element to the pNext chain.
        assert_eq!(
            out_chain.next().unwrap().s_type,
            vk::StructureType::PHYSICAL_DEVICE_SAMPLER_YCBCR_CONVERSION_FEATURES
        );
        assert_eq!(
            out_chain.next().unwrap().s_type,
            vk::StructureType::PHYSICAL_DEVICE_TIMELINE_SEMAPHORE_FEATURES
        );
        assert!(out_chain.next().is_none());
    }

    #[test]
    fn test_is_command_enabled() {
        let features: Vec<Feature> = vec![
            Extension::EXTExtendedDynamicState.into(),
            Extension::EXTShaderObject.into(),
        ];
        assert!(features.is_command_enabled(
            &ApiVersion::V1_1,
            &BTreeSet::from([Extension::EXTExtendedDynamicState])
        ));
        assert!(features.is_command_enabled(
            &ApiVersion::V1_1,
            &BTreeSet::from([Extension::EXTShaderObject])
        ));
        assert!(!features.is_command_enabled(&ApiVersion::V1_1, &BTreeSet::new()));
    }

    #[test]
    fn test_fill_vk_out_array_null_out_ptr() {
        let data = &[1, 2, 3, 4];
        let mut count = 0;
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    data,
                    NonNull::new(&mut count as *mut _).unwrap(),
                    std::ptr::null_mut(),
                )
            },
            vk::Result::SUCCESS
        );
        assert_eq!(count as usize, data.len());
    }

    #[test]
    fn test_fill_vk_out_array_empty_out_ptr() {
        let mut count = 0;
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    &[1, 2, 3, 4],
                    NonNull::new(&mut count as *mut _).unwrap(),
                    NonNull::dangling().as_ptr(),
                )
            },
            vk::Result::INCOMPLETE
        );
        assert_eq!(count, 0);

        let data: &[i32] = &[];
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    data,
                    NonNull::new(&mut count as *mut _).unwrap(),
                    NonNull::dangling().as_ptr(),
                )
            },
            vk::Result::SUCCESS
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn test_fill_vk_out_array_shorter_out_ptr() {
        let data = &[1, 2, 3, 4];
        let mut out_array = [0; 2];
        let mut count = out_array.len() as u32;
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    data,
                    NonNull::new(&mut count as *mut _).unwrap(),
                    out_array.as_mut_ptr(),
                )
            },
            vk::Result::INCOMPLETE
        );
        assert_eq!(count as usize, out_array.len());
        assert_eq!(out_array, data[0..out_array.len()]);
    }

    #[test]
    fn test_fill_vk_out_array_longer_out_ptr() {
        let data = &[1, 2];
        let mut out_array = [10; 4];
        let mut count = out_array.len() as u32;
        assert_eq!(
            unsafe {
                fill_vk_out_array(
                    data,
                    NonNull::new(&mut count as *mut _).unwrap(),
                    out_array.as_mut_ptr(),
                )
            },
            vk::Result::SUCCESS
        );
        assert_eq!(count as usize, data.len());
        assert_eq!(out_array, [1, 2, 10, 10]);
    }

    #[test]
    fn test_ptr_as_uninit_mut_null_ptr() {
        let ptr: *mut u8 = std::ptr::null_mut();
        assert!(unsafe { ptr_as_uninit_mut(ptr) }.is_none());
    }

    #[test]
    fn test_ptr_as_uninit_mut_uninit_value() {
        let expected_value: u8 = 28;
        let mut uninit_data = MaybeUninit::<u8>::uninit();
        // Miri will test if we read into uninitialized data.
        let result_uninit = unsafe { ptr_as_uninit_mut(uninit_data.as_mut_ptr()) }
            .expect("not null pointer should result in Some");
        assert_eq!(result_uninit.as_mut_ptr(), uninit_data.as_mut_ptr());
        result_uninit.write(expected_value);
        assert_eq!(unsafe { uninit_data.assume_init() }, expected_value);
    }

    // The following 2 tests guarantee that when invalid UTF-8 strings are passed in,
    // slice_to_owned_strings only panics when the returned iterator is consumed.
    #[test]
    fn test_slice_to_owned_strings_invalid_utf8_str_shouldnt_panic_before_iter_consumed() {
        let invalid_input = b"\xc3\x28\x00";
        let invalid_input: &[i8] = bytemuck::cast_slice(invalid_input);
        let invalid_inputs = [invalid_input.as_ptr()];
        let _ = unsafe { slice_to_owned_strings(&invalid_inputs) };
    }

    #[test]
    #[should_panic]
    fn test_slice_to_owned_strings_invalid_utf8_str() {
        let invalid_input = b"\xc3\x28\x00";
        let invalid_input: &[i8] = bytemuck::cast_slice(invalid_input);
        let invalid_inputs = [invalid_input.as_ptr()];
        let output = unsafe { slice_to_owned_strings(&invalid_inputs) };
        output.for_each(|_| {});
    }
}
