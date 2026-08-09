use std::collections::HashMap;
use std::mem::MaybeUninit;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use ash::{
    prelude::VkResult,
    vk::{self, Handle},
};
use vulkan_layer::{DeviceHooks, DeviceInfo, LayerResult, LayerVulkanCommand as VulkanCommand};

use crate::capture::SwapchainState;
use crate::fn_pointers::DeviceProcTable;
use crate::instance::global_instance;
use crate::ipc::CaptureLink;
use crate::logging::{log, should_sample};
use crate::process::is_target_process;

static PRESENT_COUNT: AtomicU64 = AtomicU64::new(0);

pub(crate) struct WallpiperDeviceInfo {
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) device: Arc<ash::Device>,
    pub(crate) procs: DeviceProcTable,
    pub(crate) swapchains: Mutex<HashMap<u64, SwapchainState>>,
    queue_family_map: Mutex<HashMap<u64, u32>>,
    pub(crate) capture_link: CaptureLink,
}

impl WallpiperDeviceInfo {
    pub(crate) fn new(
        physical_device: vk::PhysicalDevice,
        device: Arc<ash::Device>,
        next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ) -> Self {
        if is_target_process() {
            log!("create_device_info: resolving device function pointers");
        }
        let procs = unsafe { DeviceProcTable::load(next_get_device_proc_addr, device.handle()) };
        if is_target_process() {
            log!("create_device_info: device function pointers resolved");
        }
        Self {
            physical_device,
            device,
            procs,
            swapchains: Mutex::new(HashMap::new()),
            queue_family_map: Mutex::new(HashMap::new()),
            capture_link: CaptureLink::default(),
        }
    }

    pub(crate) fn find_memory_type(
        &self,
        type_bits: u32,
        flags: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        let props = unsafe {
            global_instance().get_physical_device_memory_properties(self.physical_device)
        };
        (0..props.memory_type_count).find(|&i| {
            let bit_set = (type_bits & (1 << i)) != 0;
            let has_flags = props.memory_types[i as usize]
                .property_flags
                .contains(flags);
            bit_set && has_flags
        })
    }

    pub(crate) fn queue_family_for(&self, queue: vk::Queue) -> Option<u32> {
        self.queue_family_map
            .lock()
            .unwrap()
            .get(&queue.as_raw())
            .copied()
    }
}

fn present_result(res: vk::Result) -> VkResult<()> {
    if res == vk::Result::SUCCESS || res == vk::Result::SUBOPTIMAL_KHR {
        Ok(())
    } else {
        Err(res)
    }
}

impl DeviceHooks for WallpiperDeviceInfo {
    fn get_device_queue(
        &self,
        queue_family_index: u32,
        queue_index: u32,
    ) -> LayerResult<vk::Queue> {
        if !is_target_process() {
            return LayerResult::Unhandled;
        }
        let mut queue = MaybeUninit::uninit();
        unsafe {
            (self.procs.get_device_queue)(
                self.device.handle(),
                queue_family_index,
                queue_index,
                queue.as_mut_ptr(),
            );
        };
        let queue = unsafe { queue.assume_init() };
        self.queue_family_map
            .lock()
            .unwrap()
            .insert(queue.as_raw(), queue_family_index);
        LayerResult::Handled(queue)
    }

    fn get_device_queue2(&self, p_queue_info: &vk::DeviceQueueInfo2) -> LayerResult<vk::Queue> {
        if !is_target_process() {
            return LayerResult::Unhandled;
        }
        let mut queue = MaybeUninit::uninit();
        unsafe {
            (self.procs.get_device_queue2)(self.device.handle(), p_queue_info, queue.as_mut_ptr());
        };
        let queue = unsafe { queue.assume_init() };
        self.queue_family_map
            .lock()
            .unwrap()
            .insert(queue.as_raw(), p_queue_info.queue_family_index);
        LayerResult::Handled(queue)
    }

    fn create_swapchain_khr(
        &self,
        p_create_info: &vk::SwapchainCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SwapchainKHR>> {
        log!("create_swapchain_khr: entered");
        let mut swapchain = vk::SwapchainKHR::null();
        let res = unsafe {
            (self.procs.create_swapchain_khr)(
                self.device.handle(),
                p_create_info,
                std::ptr::null(),
                &mut swapchain,
            )
        };
        log!("create_swapchain_khr: next-in-chain returned {res:?}");
        let result: VkResult<vk::SwapchainKHR> = if res == vk::Result::SUCCESS {
            Ok(swapchain)
        } else {
            Err(res)
        };
        if let (true, Ok(swapchain)) = (is_target_process(), result) {
            self.register_swapchain(swapchain, p_create_info);
        }
        LayerResult::Handled(result)
    }

    fn destroy_swapchain_khr(
        &self,
        swapchain: vk::SwapchainKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        if is_target_process() {
            self.teardown_swapchain(swapchain);
        }
        unsafe {
            (self.procs.destroy_swapchain_khr)(self.device.handle(), swapchain, std::ptr::null())
        };
        LayerResult::Handled(())
    }

    fn queue_present_khr(
        &self,
        queue: vk::Queue,
        p_present_info: &vk::PresentInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        if is_target_process() {
            let n = PRESENT_COUNT.fetch_add(1, Ordering::Relaxed) + 1;
            if p_present_info.swapchain_count > 0 {
                let swapchain = unsafe { *p_present_info.p_swapchains };
                let image_index = unsafe { *p_present_info.p_image_indices } as usize;
                self.capture_and_notify(queue, swapchain, image_index);
            }
            if should_sample(n) {
                log!("queue_present_khr call #{n}");
            }
        }

        let res = unsafe { (self.procs.queue_present_khr)(queue, p_present_info) };
        LayerResult::Handled(present_result(res))
    }
}

impl DeviceInfo for WallpiperDeviceInfo {
    type HooksType = Self;
    type HooksRefType<'a> = &'a Self;

    fn hooked_commands() -> &'static [VulkanCommand] {
        &[
            VulkanCommand::QueuePresentKhr,
            VulkanCommand::CreateSwapchainKhr,
            VulkanCommand::DestroySwapchainKhr,
            VulkanCommand::GetDeviceQueue,
            VulkanCommand::GetDeviceQueue2,
        ]
    }

    fn hooks(&self) -> Self::HooksRefType<'_> {
        self
    }
}
