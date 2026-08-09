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

use std::{
    ffi::{c_void, CStr},
    fmt::Debug,
    mem::MaybeUninit,
};

use ash::vk;

use smallvec::SmallVec;
use thiserror::Error;

pub mod generated;
pub use generated::*;

use crate::vk_utils::ptr_as_uninit_mut;

#[derive(Error, Debug)]
pub enum TryFromExtensionError {
    #[error("unknown extension `{0}`")]
    UnknownExtension(String),
}

/// A union type of extensions and core API version.
///
/// In `vk.xml`, Vulkan commands and types are grouped under different API version and extensions.
/// The tag of those group XML elements is `feature` or `extension`. One command will have only one
/// single correspondent feature. This type is mostly used to tell if a command should be returned
/// by `vkGet*ProcAddr` given the supported/enabled Vulkan API version and extensions.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum Feature {
    /// Vulkan core API interface.
    Core(ApiVersion),
    /// Vulkan extension interface.
    Extension(Extension),
}

impl From<ApiVersion> for Feature {
    fn from(value: ApiVersion) -> Self {
        Self::Core(value)
    }
}

impl From<Extension> for Feature {
    fn from(value: Extension) -> Self {
        Self::Extension(value)
    }
}

/// Vulkan API version number.
///
/// Can be used to store the result decoded from
/// [`VK_MAKE_API_VERSION`](https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_MAKE_API_VERSION.html).
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct ApiVersion {
    /// The major version number. At most 7 bits.
    pub major: u8,
    /// The minor version number. At most 10 bits
    pub minor: u16,
}

impl ApiVersion {
    /// Vulkan version 1.0. The initial release of the Vulkan API.
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_VERSION_1_0.html>
    pub const V1_0: Self = Self { major: 1, minor: 0 };
    /// Vulkan version 1.1.
    ///
    /// <https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/VK_VERSION_1_1.html>
    pub const V1_1: Self = Self { major: 1, minor: 1 };
}

impl From<u32> for ApiVersion {
    fn from(value: u32) -> Self {
        Self {
            major: vk::api_version_major(value)
                .try_into()
                .expect("The major version must be no more than 7 bits."),
            minor: vk::api_version_minor(value)
                .try_into()
                .expect("The minor version must be no more than 10 bits."),
        }
    }
}

impl From<ApiVersion> for u32 {
    fn from(value: ApiVersion) -> Self {
        vk::make_api_version(0, value.major.into(), value.minor.into(), 0)
    }
}

pub(crate) struct VulkanCommand {
    pub name: &'static str,
    pub features: SmallVec<[Feature; 2]>,
    pub hooked: bool,
    pub proc: vk::PFN_vkVoidFunction,
}

fn get_instance_proc_addr_loader(
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    instance: &ash::Instance,
) -> impl Fn(&CStr) -> *const c_void + '_ {
    move |name| {
        // Safe because the VkInstance is valid, and a valid C string pointer is passed to the
        // `p_name` parameter.
        let fp = unsafe { get_instance_proc_addr(instance.handle(), name.as_ptr()) };
        if let Some(fp) = fp {
            return fp as *const _;
        }
        std::ptr::null()
    }
}

fn get_device_proc_addr_loader(
    get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    device: &ash::Device,
) -> impl Fn(&CStr) -> *const c_void + '_ {
    move |name| {
        // Safe because the VkDevice is valid, and a valid C string pointer is passed to the
        // `p_name` parameter.
        let fp = unsafe { get_device_proc_addr(device.handle(), name.as_ptr()) };
        match fp {
            Some(fp) => fp as *const _,
            None => std::ptr::null(),
        }
    }
}

/// Converts a raw pointer to a reference of a slice of maybe-uninit. In contrast to
/// `from_raw_parts_mut`, this does not require that the value has to be initialized and the input
/// pointer can be null and may not be aligned if the input size is 0. If either `p_out_array` or
/// `p_size` is null, [`None`] is returned. Otherwise, [`Some`] is returned.
///
/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
///
/// * `p_data` must be valid for writes for `size * mem::size_of::<T>()` many bytes, and it must be
///   properly aligned. This means in particular, the entire memory range of this slice must be
///   contained within a single allocated object! Slices can never span across multiple allocated
///   objects. `p_data` must be be aligned if `size` is not 0.
/// * The memory referenced by the returned slice must not be accessed through any other pointer
///   (not derived from the return value) for the duration of lifetime 'a. Both read and write
///   accesses are forbidden.
/// * The total size `size * mem::size_of::<T>()` of the slice must be no larger than
///   [`std::isize::MAX`], and adding that size to data must not “wrap around” the address space.
///   See the safety documentation of
///   [`pointer::offset`](https://doc.rust-lang.org/std/primitive.pointer.html#method.offset).
/// * `p_size` must be either null or points to a valid data to read. See details at [`pointer::as_ref`](https://doc.rust-lang.org/std/primitive.pointer.html#safety).
///
/// # Panics
/// * Panics if `size` can't be converted to usize.
#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn maybe_uninit_slice_from_raw_parts_mut<'a, T>(
    p_out_array: *mut T,
    p_size: *const (impl TryInto<usize, Error = impl Debug> + Copy),
) -> Option<&'a mut [MaybeUninit<T>]> {
    let size: usize = unsafe { p_size.as_ref() }
        .copied()?
        .try_into()
        .expect("size mut be within the range of usize");
    if p_out_array.is_null() {
        return None;
    }
    Some(unsafe { uninit_slice_from_raw_parts_mut(p_out_array, size) })
}

/// Forms a slice from a pointer and a length.
///
/// In contrast to [`std::slice::from_raw_parts`], this does not require that `data` must be
/// non-null and unaligned for zero-length slice. If `data` is null, [`None`] is returned.
///
/// # Safety
///
/// If `len` is 0, there is no safety requirement.
///
/// If `len` is not 0, the following conditions shouldn't be violated:
/// * `data` must be valid for reads for `len * mem::size_of::<T>()` many bytes, and it must be
///   properly aligned. This means in particular: the entire memory range of this slice must be
///   contained within a single allocated object! Slices can never span across multiple allocated
///   objects.
/// * `data` must point to `len` consecutive properly initialized values of type `T`.
/// * The memory referenced by the returned slice must not be mutated for the duration of lifetime
///   `'a`, except inside an `UnsafeCell`.
/// * The total size `len * mem::size_of::<T>()` of the slice must be no larger than [`isize::MAX`],
///   and adding that size to data must not “wrap around” the address space. See the safety
///   documentation of
///   [`pointer::offset`](https://doc.rust-lang.org/std/primitive.pointer.html#method.offset).
///
/// # Panics
///
/// Panics if `len` can't be converted to `uszie`.
#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn maybe_slice_from_raw_parts<'a, T>(
    data: *const T,
    len: impl TryInto<usize, Error = impl Debug>,
) -> Option<&'a [T]> {
    if data.is_null() {
        return None;
    }
    let len: usize = len
        .try_into()
        .expect("len mut be within the range of usize");
    if len == 0 {
        return Some(&[]);
    }
    Some(unsafe { std::slice::from_raw_parts(data, len) })
}

/// Converts a raw pointer to a reference of a slice of maybe-uninit. In contrast to
/// `from_raw_parts_mut`, this does not require that the value has to be initialized and the input
/// pointer can be null and may not be aligned if the input size is 0.
///
/// # Safety
/// Behavior is undefined if any of the following conditions are violated:
///
/// * `p_data` must be valid for writes for `size * mem::size_of::<T>()` many bytes, and it must be
///   properly aligned. This means in particular, the entire memory range of this slice must be
///   contained within a single allocated object! Slices can never span across multiple allocated
///   objects. `p_data` must be be aligned if `size` is not 0.
/// * The memory referenced by the returned slice must not be accessed through any other pointer
///   (not derived from the return value) for the duration of lifetime 'a. Both read and write
///   accesses are forbidden.
/// * The total size `size * mem::size_of::<T>()` of the slice must be no larger than
///   [`std::isize::MAX`], and adding that size to data must not “wrap around” the address space.
///   See the safety documentation of
///   [`pointer::offset`](https://doc.rust-lang.org/std/primitive.pointer.html#method.offset).
///
/// # Panics
/// * Panics if `size` is not 0 and `p_data` is null.
/// * Panics if `size` can't be converted to usize.
#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn uninit_slice_from_raw_parts_mut<'a, T>(
    p_data: *mut T,
    size: impl TryInto<usize, Error = impl Debug>,
) -> &'a mut [MaybeUninit<T>] {
    let size: usize = size
        .try_into()
        .expect("size mut be within the range of usize");
    if size == 0 {
        return &mut [];
    }
    let first_element =
        unsafe { ptr_as_uninit_mut(p_data) }.expect("the input data pointer should not be null");
    unsafe { std::slice::from_raw_parts_mut(first_element, size) }
}

#[deny(unsafe_op_in_unsafe_fn)]
unsafe fn bool_iterator_from_raw_parts(
    ptr: *const vk::Bool32,
    size: impl TryInto<usize, Error = impl Debug>,
) -> impl Iterator<Item = bool> {
    let size: usize = size
        .try_into()
        .expect("size mut be within the range of usize");
    let slice = if size == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(ptr, size) }
    };
    slice.iter().map(|v| *v == vk::TRUE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bool_iterator_from_raw_parts_result_should_match() {
        let expected_value = vec![true, true, false, true, false];
        let input = expected_value
            .iter()
            .map(|v| if *v { vk::TRUE } else { vk::FALSE })
            .collect::<Vec<_>>();
        let result = unsafe { bool_iterator_from_raw_parts(input.as_ptr(), expected_value.len()) }
            .collect::<Vec<_>>();
        assert_eq!(result, expected_value);
    }

    #[test]
    fn bool_iterator_from_raw_parts_empty_with_null_ptr() {
        let result =
            unsafe { bool_iterator_from_raw_parts(std::ptr::null(), 0) }.collect::<Vec<_>>();
        assert!(result.is_empty());
    }

    #[test]
    fn maybe_uninit_slice_from_raw_parts_mut_result_should_match() {
        const LEN: usize = 10;
        let expected_value: [i32; LEN] = [81, 95, 43, 65, 34, 47, 65, 62, 47, 82];
        let mut input = [MaybeUninit::<i32>::uninit(); LEN];
        let input_ptr = input.as_mut_ptr() as *mut i32;
        let output = unsafe { maybe_uninit_slice_from_raw_parts_mut(input_ptr, &LEN) }
            .expect("for valid input should return in Some");
        assert_eq!(output.len(), LEN);
        for (i, output_element) in output.iter_mut().enumerate() {
            output_element.write(expected_value[i]);
        }

        for (i, input_element) in input.iter().enumerate() {
            assert_eq!(
                *unsafe { input_element.assume_init_ref() },
                expected_value[i]
            );
        }
    }

    #[test]
    fn maybe_uninit_slice_from_raw_parts_mut_null_data_ptr() {
        let input_ptr: *mut i32 = std::ptr::null_mut();
        let output = unsafe { maybe_uninit_slice_from_raw_parts_mut(input_ptr, &10) };
        assert!(output.is_none());
    }

    #[test]
    fn maybe_uninit_slice_from_raw_parts_zero_length_unaligned_data_ptr() {
        // Some address in the u8_array must be unaligned with u32.
        let mut u8_array = [0u8; 2];
        let input_ptrs = [
            &mut u8_array[0] as *mut _ as *mut i32,
            &mut u8_array[1] as *mut _ as *mut i32,
        ];
        for input_ptr in input_ptrs {
            let output = unsafe { maybe_uninit_slice_from_raw_parts_mut(input_ptr, &0) }
                .expect("for valid input pointer, Some should be returned");
            assert!(output.is_empty());
        }
    }

    #[test]
    fn maybe_uninit_slice_from_raw_parts_mut_null_size_ptr() {
        let mut data = MaybeUninit::<u32>::uninit();
        let output = unsafe {
            maybe_uninit_slice_from_raw_parts_mut(data.as_mut_ptr(), std::ptr::null::<usize>())
        };
        assert!(output.is_none());
    }

    #[test]
    #[should_panic]
    fn maybe_uninit_slice_from_raw_parts_mut_invalid_size_value() {
        let mut data = MaybeUninit::<u32>::uninit();
        unsafe { maybe_uninit_slice_from_raw_parts_mut(data.as_mut_ptr(), &-1) };
    }

    #[test]
    fn uninit_slice_from_raw_parts_mut_zero_size_and_invalid_data_address() {
        // Some address in the u8_array must be unaligned with u32.
        let mut u8_array = [0u8; 2];
        let input_ptrs = [
            &mut u8_array[0] as *mut _ as *mut i32,
            &mut u8_array[1] as *mut _ as *mut i32,
        ];
        for input_ptr in input_ptrs {
            let output = unsafe { uninit_slice_from_raw_parts_mut(input_ptr, 0) };
            assert!(output.is_empty());
        }

        let output = unsafe { uninit_slice_from_raw_parts_mut(std::ptr::null_mut::<u8>(), 0) };
        assert!(output.is_empty());
    }

    #[test]
    #[should_panic]
    fn uninit_slice_from_raw_parts_mut_valid_size_and_null_data_address() {
        unsafe { uninit_slice_from_raw_parts_mut(std::ptr::null_mut::<u8>(), 10) };
    }

    #[test]
    #[should_panic]
    fn uninit_slice_from_raw_parts_mut_invalid_size_value() {
        let mut data = MaybeUninit::<u32>::uninit();
        unsafe { uninit_slice_from_raw_parts_mut(data.as_mut_ptr(), -1) };
    }

    #[test]
    fn uninit_slice_from_raw_parts_mut_result_should_match() {
        const LEN: usize = 10;
        let expected_value: [i32; LEN] = [14, 45, 60, 97, 35, 21, 13, 42, 11, 12];
        let mut input = [MaybeUninit::<i32>::uninit(); LEN];
        let input_ptr = input.as_mut_ptr() as *mut i32;
        let output = unsafe { uninit_slice_from_raw_parts_mut(input_ptr, LEN) };
        assert_eq!(output.len(), LEN);
        for (i, output_element) in output.iter_mut().enumerate() {
            output_element.write(expected_value[i]);
        }

        for (i, input_element) in input.iter().enumerate() {
            assert_eq!(
                *unsafe { input_element.assume_init_ref() },
                expected_value[i]
            );
        }
    }

    #[test]
    fn maybe_slice_from_raw_parts_null_data() {
        let res = unsafe { maybe_slice_from_raw_parts(std::ptr::null::<u8>(), 0) };
        assert!(res.is_none());
        let res = unsafe { maybe_slice_from_raw_parts(std::ptr::null::<u8>(), 10) };
        assert!(res.is_none());
    }

    #[test]
    fn maybe_slice_from_raw_parts_zero_length_invalid_data_ptr() {
        let u8_array = [0u8; 2];
        // Some address must not be aligned.
        let input_ptrs = u8_array
            .iter()
            .map(|element| element as *const _ as *const u32)
            .collect::<Vec<_>>();
        for input_ptr in input_ptrs {
            let res = unsafe { maybe_slice_from_raw_parts(input_ptr, 0) }
                .expect("should always return Some for non-null pointers");
            assert!(res.is_empty());
        }
    }

    #[test]
    fn maybe_slice_from_raw_parts_reflects_original_array() {
        let input: [u32; 10] = [47, 63, 14, 13, 8, 45, 52, 97, 21, 10];
        let result = unsafe { maybe_slice_from_raw_parts(input.as_ptr(), input.len()) }
            .expect("should always return Some for non-null pointers");
        assert_eq!(result, &input);
    }

    #[test]
    #[should_panic]
    fn maybe_slice_from_raw_parts_bad_len() {
        unsafe { maybe_slice_from_raw_parts(std::ptr::NonNull::<u8>::dangling().as_ptr(), -1) };
    }

    #[test]
    fn extension_try_from_should_return_error_on_unknown_extension() {
        let unknown_extension = "VK_UNKNOWN_unknown";
        let err = Extension::try_from(unknown_extension).unwrap_err();
        assert!(err.to_string().contains(unknown_extension));
    }
}
