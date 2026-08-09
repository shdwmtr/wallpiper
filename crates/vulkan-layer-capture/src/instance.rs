use std::ffi::{CStr, CString};
use std::mem::MaybeUninit;
use std::sync::{Arc, OnceLock};

use ash::{prelude::VkResult, vk};
use vulkan_layer::{auto_instanceinfo_impl, InstanceHooks, LayerResult, VkLayerDeviceLink};

use crate::logging::log;
use crate::process::is_target_process;

static GLOBAL_INSTANCE: OnceLock<Arc<ash::Instance>> = OnceLock::new();

const REQUIRED_DEVICE_EXTENSIONS: [&CStr; 2] = [
    c"VK_EXT_image_drm_format_modifier",
    c"VK_KHR_image_format_list",
];

pub(crate) fn set_global_instance(instance: Arc<ash::Instance>) {
    let _ = GLOBAL_INSTANCE.set(instance);
}

pub(crate) fn global_instance() -> Arc<ash::Instance> {
    GLOBAL_INSTANCE
        .get()
        .expect("instance not yet created")
        .clone()
}

#[derive(Default)]
pub(crate) struct WallpiperInstanceHooks;

#[auto_instanceinfo_impl]
impl InstanceHooks for WallpiperInstanceHooks {
    fn create_device(
        &self,
        physical_device: vk::PhysicalDevice,
        p_create_info: &vk::DeviceCreateInfo,
        layer_device_link: &VkLayerDeviceLink,
        p_allocator: Option<&vk::AllocationCallbacks>,
        p_device: &mut MaybeUninit<vk::Device>,
    ) -> LayerResult<VkResult<()>> {
        if !is_target_process() {
            return LayerResult::Unhandled;
        }

        let mut extensions: Vec<CString> = unsafe {
            std::slice::from_raw_parts(
                p_create_info.pp_enabled_extension_names,
                p_create_info.enabled_extension_count as usize,
            )
        }
        .iter()
        .map(|&p| unsafe { CStr::from_ptr(p) }.to_owned())
        .collect();

        for required in REQUIRED_DEVICE_EXTENSIONS {
            if !extensions.iter().any(|e| e.as_c_str() == required) {
                extensions.push(required.to_owned());
            }
        }
        let extension_ptrs: Vec<*const std::os::raw::c_char> =
            extensions.iter().map(|e| e.as_ptr()).collect();

        let mut new_create_info = *p_create_info;
        new_create_info.enabled_extension_count = extension_ptrs.len() as u32;
        new_create_info.pp_enabled_extension_names = extension_ptrs.as_ptr();

        let instance_handle = global_instance().handle();
        let next_create_device: vk::PFN_vkCreateDevice = unsafe {
            std::mem::transmute((layer_device_link.pfnNextGetInstanceProcAddr)(
                instance_handle,
                c"vkCreateDevice".as_ptr(),
            ))
        };

        let allocator_ptr =
            p_allocator.map_or(std::ptr::null(), |a| a as *const vk::AllocationCallbacks);
        let res = unsafe {
            next_create_device(
                physical_device,
                &new_create_info,
                allocator_ptr,
                p_device.as_mut_ptr(),
            )
        };
        log!("create_device: injected required extensions, next-in-chain returned {res:?}");

        if res == vk::Result::SUCCESS {
            LayerResult::Handled(Ok(()))
        } else {
            LayerResult::Handled(Err(res))
        }
    }
}
