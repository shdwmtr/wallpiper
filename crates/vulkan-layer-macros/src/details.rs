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

use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::iter::zip;
use syn::{spanned::Spanned, Error, Ident, ImplItem, ItemImpl, Type};

fn snake_case_to_upper_camel_case(input: &str) -> String {
    let first_char = match input.chars().next() {
        Some(first) => first,
        None => return input.to_owned(),
    };
    let first_char = first_char.to_uppercase();
    let mut res: String = first_char.to_string();
    for (prev, (cur, next)) in zip(
        input.chars(),
        zip(input.chars().skip(1), input.chars().skip(2)),
    ) {
        if cur == '_' && next != '_' {
            continue;
        }
        if prev == '_' {
            res.push_str(&cur.to_uppercase().to_string());
            continue;
        }
        res.push(cur);
    }
    res.push(input.chars().last().unwrap());
    res
}

pub fn autoinfo(item: &ItemImpl, target_trait: &TokenStream2) -> Result<TokenStream2, Error> {
    let type_name = item.self_ty.as_ref();
    let hooked_commands = item.items.iter().filter_map(|item| {
        let function = if let ImplItem::Fn(function) = item {
            function
        } else {
            return None;
        };
        let func_name = function.sig.ident.to_string();
        let enum_variant_name = snake_case_to_upper_camel_case(&func_name);
        let enum_variant_name = Ident::new(&enum_variant_name, function.span());
        Some(quote!(::vulkan_layer::LayerVulkanCommand::#enum_variant_name))
    });
    Ok(quote! {
        impl #target_trait for #type_name {
            type HooksType = Self;
            type HooksRefType<'a> = &'a Self;

            fn hooked_commands() -> &'static [::vulkan_layer::LayerVulkanCommand] {
                &[#(#hooked_commands),*]
            }

            fn hooks(&self) -> &Self {
                self
            }
        }
    })
}

pub fn declare_introspection_queries_impl(global_type: &Type) -> Result<TokenStream2, Error> {
    Ok(quote! {
        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "See valid usage of `vkEnumerateInstanceLayerProperties` at "]
        #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceLayerProperties.html>"]
        #[deny(unsafe_op_in_unsafe_fn)]
        #[no_mangle]
        pub unsafe extern "system" fn vkEnumerateInstanceLayerProperties(
            property_count: *mut u32,
            properties: *mut ::ash::vk::LayerProperties
        ) -> ::ash::vk::Result {
            // Safe because the caller is supposed to follow the exact same safety requirement.
            unsafe { #global_type::enumerate_instance_layer_properties(property_count, properties) }
        }

        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "See valid usage of `vkEnumerateInstanceExtensionProperties` at "]
        #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateInstanceExtensionProperties.html>"]
        #[deny(unsafe_op_in_unsafe_fn)]
        #[no_mangle]
        pub unsafe extern "system" fn vkEnumerateInstanceExtensionProperties(
            layer_name: *const ::std::ffi::c_char,
            property_count: *mut u32,
            properties: *mut ::ash::vk::ExtensionProperties
        ) -> ::ash::vk::Result {
            // Safe because the caller is supposed to follow the exact same safety requirement.
            unsafe {
                #global_type::enumerate_instance_extension_properties(
                    layer_name,
                    property_count,
                    properties,
                )
            }
        }

        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "See valid usage of `vkEnumerateDeviceLayerProperties` at "]
        #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceLayerProperties.html>"]
        #[deny(unsafe_op_in_unsafe_fn)]
        #[no_mangle]
        pub unsafe extern "system" fn vkEnumerateDeviceLayerProperties(
            physical_device: ::ash::vk::PhysicalDevice,
            p_property_count: *mut u32,
            p_properties: *mut ::ash::vk::LayerProperties,
        ) -> ::ash::vk::Result {
            // Safe, because the caller is supposed to follow the exact same safety requirement.
            unsafe {
                #global_type::enumerate_device_layer_properties(
                    physical_device,
                    p_property_count,
                    p_properties,
                )
            }
        }

        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "See valid usage of `vkEnumerateDeviceLayerProperties` at "]
        #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkEnumerateDeviceLayerProperties.html>"]
        #[deny(unsafe_op_in_unsafe_fn)]
        #[no_mangle]
        pub unsafe extern "system" fn vkEnumerateDeviceExtensionProperties(
            physical_device: ::ash::vk::PhysicalDevice,
            p_layer_name: *const ::std::ffi::c_char,
            p_property_count: *mut u32,
            p_properties: *mut ::ash::vk::ExtensionProperties,
        ) -> ::ash::vk::Result {
            // Safe, because the caller is supposed to follow the exact same safety requirement.
            unsafe {
                #global_type::enumerate_device_extension_properties(
                    physical_device,
                    p_layer_name,
                    p_property_count,
                    p_properties,
                )
            }
        }

        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "See valid usage of `vkGetInstanceProcAddr` at "]
        #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetInstanceProcAddr.html>"]
        #[deny(unsafe_op_in_unsafe_fn)]
        #[no_mangle]
        pub unsafe extern "system" fn vkGetInstanceProcAddr(
            instance: ::ash::vk::Instance,
            p_name: *const ::std::ffi::c_char,
        ) -> ::ash::vk::PFN_vkVoidFunction {
            // Safe, because the caller is supposed to follow the exact same safety requirement.
            unsafe {
                #global_type::get_instance_proc_addr(instance, p_name)
            }
        }

        #[doc = "# Safety"]
        #[doc = ""]
        #[doc = "See valid usage of `vkGetDeviceProcAddr` at "]
        #[doc = "<https://registry.khronos.org/vulkan/specs/1.3-extensions/man/html/vkGetDeviceProcAddr.html>"]
        #[deny(unsafe_op_in_unsafe_fn)]
        #[no_mangle]
        pub unsafe extern "system" fn vkGetDeviceProcAddr(
            device: ::ash::vk::Device,
            p_name: *const ::std::ffi::c_char,
        ) -> ::ash::vk::PFN_vkVoidFunction {
            // Safe, because the caller is supposed to follow the exact same safety requirement.
            unsafe {
                #global_type::get_device_proc_addr(device, p_name)
            }
        }
    })
}
