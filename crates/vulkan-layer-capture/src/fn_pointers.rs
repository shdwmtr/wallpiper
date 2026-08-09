use ash::vk;

pub(crate) type PfnCreateSwapchainKhr = unsafe extern "C" fn(
    vk::Device,
    *const vk::SwapchainCreateInfoKHR,
    *const vk::AllocationCallbacks,
    *mut vk::SwapchainKHR,
) -> vk::Result;
pub(crate) type PfnDestroySwapchainKhr =
    unsafe extern "C" fn(vk::Device, vk::SwapchainKHR, *const vk::AllocationCallbacks);
pub(crate) type PfnQueuePresentKhr =
    unsafe extern "C" fn(vk::Queue, *const vk::PresentInfoKHR) -> vk::Result;
pub(crate) type PfnGetMemoryFdKhr =
    unsafe extern "C" fn(vk::Device, *const vk::MemoryGetFdInfoKHR, *mut i32) -> vk::Result;
pub(crate) type PfnGetSwapchainImagesKhr =
    unsafe extern "C" fn(vk::Device, vk::SwapchainKHR, *mut u32, *mut vk::Image) -> vk::Result;
pub(crate) type PfnGetImageDrmFormatModifierPropertiesExt = unsafe extern "C" fn(
    vk::Device,
    vk::Image,
    *mut vk::ImageDrmFormatModifierPropertiesEXT,
) -> vk::Result;
pub(crate) type PfnGetDeviceQueue = unsafe extern "C" fn(vk::Device, u32, u32, *mut vk::Queue);
pub(crate) type PfnGetDeviceQueue2 =
    unsafe extern "C" fn(vk::Device, *const vk::DeviceQueueInfo2, *mut vk::Queue);

pub(crate) struct DeviceProcTable {
    pub create_swapchain_khr: PfnCreateSwapchainKhr,
    pub destroy_swapchain_khr: PfnDestroySwapchainKhr,
    pub queue_present_khr: PfnQueuePresentKhr,
    pub get_memory_fd_khr: PfnGetMemoryFdKhr,
    pub get_swapchain_images_khr: PfnGetSwapchainImagesKhr,
    pub get_image_drm_format_modifier_properties_ext: PfnGetImageDrmFormatModifierPropertiesExt,
    pub get_device_queue: PfnGetDeviceQueue,
    pub get_device_queue2: PfnGetDeviceQueue2,
}

impl DeviceProcTable {
    pub(crate) unsafe fn load(
        get_proc_addr: vk::PFN_vkGetDeviceProcAddr,
        device: vk::Device,
    ) -> Self {
        Self {
            create_swapchain_khr: load_fn(get_proc_addr, device, c"vkCreateSwapchainKHR"),
            destroy_swapchain_khr: load_fn(get_proc_addr, device, c"vkDestroySwapchainKHR"),
            queue_present_khr: load_fn(get_proc_addr, device, c"vkQueuePresentKHR"),
            get_memory_fd_khr: load_fn(get_proc_addr, device, c"vkGetMemoryFdKHR"),
            get_swapchain_images_khr: load_fn(get_proc_addr, device, c"vkGetSwapchainImagesKHR"),
            get_image_drm_format_modifier_properties_ext: load_fn(
                get_proc_addr,
                device,
                c"vkGetImageDrmFormatModifierPropertiesEXT",
            ),
            get_device_queue: load_fn(get_proc_addr, device, c"vkGetDeviceQueue"),
            get_device_queue2: load_fn(get_proc_addr, device, c"vkGetDeviceQueue2"),
        }
    }
}

unsafe fn load_fn<F>(
    get_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    device: vk::Device,
    name: &std::ffi::CStr,
) -> F {
    let f = get_proc_addr(device, name.as_ptr());
    std::mem::transmute_copy::<_, F>(&f)
}
