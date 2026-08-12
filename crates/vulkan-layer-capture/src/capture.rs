use std::os::unix::io::RawFd;

use ash::vk::{self, Handle};

use crate::config::{CAPTURE_SLOT_COUNT, SLOT_FENCE_TIMEOUT_NS};
use crate::device::WallpiperDeviceInfo;
use crate::instance::global_instance;
use crate::ipc::BufferAnnounce;
use crate::logging::{log, should_sample};

#[derive(Clone, Copy, PartialEq, Eq)]
enum SlotOwnership {
    NeverWritten,
    ReleasedToForeign,
}

struct CaptureSlot {
    image: vk::Image,
    memory: vk::DeviceMemory,
    pool: vk::CommandPool,
    cmd: vk::CommandBuffer,
    fence: vk::Fence,
    fence_pending: bool,
    ready_semaphore: vk::Semaphore,
    ownership: SlotOwnership,
    buf_sent: bool,
    stride: u32,
    modifier: u64,
}

pub(crate) struct SwapchainState {
    format: vk::Format,
    extent: vk::Extent2D,
    images: Vec<vk::Image>,
    candidate_modifiers: Vec<u64>,
    slots: Vec<CaptureSlot>,
    next_slot: usize,
    present_count: u64,
}

impl WallpiperDeviceInfo {
    pub(crate) fn register_swapchain(
        &self,
        swapchain: vk::SwapchainKHR,
        create_info: &vk::SwapchainCreateInfoKHR,
    ) {
        let images = self.get_swapchain_images(swapchain);
        let candidate_modifiers = self.compute_candidate_modifiers(create_info.image_format);
        log!(
            "create_swapchain_khr -> {swapchain:?}, format={:?}, extent={:?}, {} images, {} candidate modifiers",
            create_info.image_format,
            create_info.image_extent,
            images.len(),
            candidate_modifiers.len(),
        );
        self.swapchains.lock().unwrap().insert(
            swapchain.as_raw(),
            SwapchainState {
                format: create_info.image_format,
                extent: create_info.image_extent,
                images,
                candidate_modifiers,
                slots: Vec::with_capacity(CAPTURE_SLOT_COUNT),
                next_slot: 0,
                present_count: 0,
            },
        );
    }

    pub(crate) fn teardown_swapchain(&self, swapchain: vk::SwapchainKHR) {
        let Some(state) = self.swapchains.lock().unwrap().remove(&swapchain.as_raw()) else {
            return;
        };
        log!(
            "destroy_swapchain_khr: tearing down {} capture slot(s) for {swapchain:?}",
            state.slots.len()
        );
        for slot in state.slots {
            self.destroy_capture_slot(slot);
        }
    }

    fn compute_candidate_modifiers(&self, format: vk::Format) -> Vec<u64> {
        let required_features =
            vk::FormatFeatureFlags::TRANSFER_DST | vk::FormatFeatureFlags::SAMPLED_IMAGE;
        let instance = global_instance();

        let mut modifier_list_query = vk::DrmFormatModifierPropertiesListEXT::builder();
        let mut format_props2 =
            vk::FormatProperties2::builder().push_next(&mut modifier_list_query);
        unsafe {
            instance.get_physical_device_format_properties2(
                self.physical_device,
                format,
                &mut format_props2,
            )
        };
        let count = modifier_list_query.drm_format_modifier_count;

        let mut modifier_props =
            vec![vk::DrmFormatModifierPropertiesEXT::default(); count as usize];
        let mut modifier_list_query2 = vk::DrmFormatModifierPropertiesListEXT::builder()
            .drm_format_modifier_properties(&mut modifier_props);
        let mut format_props2b =
            vk::FormatProperties2::builder().push_next(&mut modifier_list_query2);
        unsafe {
            instance.get_physical_device_format_properties2(
                self.physical_device,
                format,
                &mut format_props2b,
            )
        };

        modifier_props
            .iter()
            .filter(|m| {
                m.drm_format_modifier_plane_count == 1
                    && m.drm_format_modifier_tiling_features
                        .contains(required_features)
            })
            .map(|m| m.drm_format_modifier)
            .collect()
    }

    fn create_capture_slot(
        &self,
        format: vk::Format,
        extent: vk::Extent2D,
        candidate_modifiers: &[u64],
        queue_family_index: u32,
    ) -> Option<CaptureSlot> {
        if candidate_modifiers.is_empty() {
            log!("no usable single-plane DRM format modifier found for capture image");
            return None;
        }

        let mut export_image_info = vk::ExternalMemoryImageCreateInfo::builder()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let mut modifier_create_info = vk::ImageDrmFormatModifierListCreateInfoEXT::builder()
            .drm_format_modifiers(candidate_modifiers);

        let image_info = vk::ImageCreateInfo::builder()
            .push_next(&mut export_image_info)
            .push_next(&mut modifier_create_info)
            .image_type(vk::ImageType::TYPE_2D)
            .format(format)
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            })
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED)
            .sharing_mode(vk::SharingMode::EXCLUSIVE)
            .initial_layout(vk::ImageLayout::UNDEFINED);

        let image = match unsafe { self.device.create_image(&image_info, None) } {
            Ok(i) => i,
            Err(e) => {
                log!("create_image (capture target) failed: {e:?}");
                return None;
            }
        };

        let mut chosen_modifier_props = vk::ImageDrmFormatModifierPropertiesEXT::default();
        let res = unsafe {
            (self.procs.get_image_drm_format_modifier_properties_ext)(
                self.device.handle(),
                image,
                &mut chosen_modifier_props,
            )
        };
        if res != vk::Result::SUCCESS {
            log!("vkGetImageDrmFormatModifierPropertiesEXT failed: {res:?}");
            unsafe { self.device.destroy_image(image, None) };
            return None;
        }
        let modifier = chosen_modifier_props.drm_format_modifier;

        let mem_req = unsafe { self.device.get_image_memory_requirements(image) };
        let Some(mem_type_index) = self.find_memory_type(
            mem_req.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        ) else {
            log!("no suitable memory type found for capture image");
            unsafe { self.device.destroy_image(image, None) };
            return None;
        };

        let mut export_alloc_info = vk::ExportMemoryAllocateInfo::builder()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
        let alloc_info = vk::MemoryAllocateInfo::builder()
            .push_next(&mut export_alloc_info)
            .allocation_size(mem_req.size)
            .memory_type_index(mem_type_index);

        let memory = match unsafe { self.device.allocate_memory(&alloc_info, None) } {
            Ok(m) => m,
            Err(e) => {
                log!("allocate_memory (export) failed: {e:?}");
                unsafe { self.device.destroy_image(image, None) };
                return None;
            }
        };

        if let Err(e) = unsafe { self.device.bind_image_memory(image, memory, 0) } {
            log!("bind_image_memory failed: {e:?}");
            unsafe {
                self.device.destroy_image(image, None);
                self.device.free_memory(memory, None);
            }
            return None;
        }

        let subresource = vk::ImageSubresource::builder()
            .aspect_mask(vk::ImageAspectFlags::MEMORY_PLANE_0_EXT)
            .mip_level(0)
            .array_layer(0);
        let layout = unsafe {
            self.device
                .get_image_subresource_layout(image, *subresource)
        };
        let stride = layout.row_pitch as u32;

        let pool_info = vk::CommandPoolCreateInfo::builder()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(queue_family_index);
        let pool = match unsafe { self.device.create_command_pool(&pool_info, None) } {
            Ok(p) => p,
            Err(e) => {
                log!("create_command_pool failed: {e:?}");
                unsafe {
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                return None;
            }
        };

        let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(1);
        let cmd = match unsafe { self.device.allocate_command_buffers(&cmd_alloc_info) } {
            Ok(b) => b[0],
            Err(e) => {
                log!("allocate_command_buffers failed: {e:?}");
                unsafe {
                    self.device.destroy_command_pool(pool, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                return None;
            }
        };

        let fence = match unsafe {
            self.device
                .create_fence(&vk::FenceCreateInfo::builder(), None)
        } {
            Ok(f) => f,
            Err(e) => {
                log!("create_fence failed: {e:?}");
                unsafe {
                    self.device.destroy_command_pool(pool, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                return None;
            }
        };

        let mut export_semaphore_info = vk::ExportSemaphoreCreateInfo::builder()
            .handle_types(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
        let semaphore_info =
            vk::SemaphoreCreateInfo::builder().push_next(&mut export_semaphore_info);
        let ready_semaphore = match unsafe { self.device.create_semaphore(&semaphore_info, None) } {
            Ok(s) => s,
            Err(e) => {
                log!("create_semaphore (export) failed: {e:?}");
                unsafe {
                    self.device.destroy_fence(fence, None);
                    self.device.destroy_command_pool(pool, None);
                    self.device.destroy_image(image, None);
                    self.device.free_memory(memory, None);
                }
                return None;
            }
        };

        log!(
            "capture slot created: image={image:?} memory={memory:?} {}x{} format={format:?} stride={stride} modifier={modifier} queue_family={queue_family_index}",
            extent.width, extent.height
        );

        Some(CaptureSlot {
            image,
            memory,
            pool,
            cmd,
            fence,
            fence_pending: false,
            ready_semaphore,
            ownership: SlotOwnership::NeverWritten,
            buf_sent: false,
            stride,
            modifier,
        })
    }

    fn destroy_capture_slot(&self, slot: CaptureSlot) {
        unsafe {
            if slot.fence_pending {
                let _ = self
                    .device
                    .wait_for_fences(&[slot.fence], true, SLOT_FENCE_TIMEOUT_NS);
            }
            self.device.destroy_semaphore(slot.ready_semaphore, None);
            self.device.destroy_fence(slot.fence, None);
            self.device.destroy_command_pool(slot.pool, None);
            self.device.destroy_image(slot.image, None);
            self.device.free_memory(slot.memory, None);
        }
    }

    fn record_and_submit_copy(
        &self,
        queue: vk::Queue,
        queue_family_index: u32,
        slot: &CaptureSlot,
        src_image: vk::Image,
        extent: vk::Extent2D,
    ) -> bool {
        let cmd = slot.cmd;
        let begin_info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        if let Err(e) = unsafe { self.device.begin_command_buffer(cmd, &begin_info) } {
            log!("begin_command_buffer failed: {e:?}");
            return false;
        }

        let subresource = vk::ImageSubresourceRange::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .base_mip_level(0)
            .level_count(1)
            .base_array_layer(0)
            .layer_count(1);

        let to_transfer_src = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(src_image)
            .subresource_range(*subresource)
            .src_access_mask(vk::AccessFlags::MEMORY_READ)
            .dst_access_mask(vk::AccessFlags::TRANSFER_READ);

        let to_transfer_dst = match slot.ownership {
            SlotOwnership::NeverWritten => vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(slot.image)
                .subresource_range(*subresource)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE),
            SlotOwnership::ReleasedToForeign => vk::ImageMemoryBarrier::builder()
                .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
                .dst_queue_family_index(queue_family_index)
                .image(slot.image)
                .subresource_range(*subresource)
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE),
        };

        unsafe {
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*to_transfer_src, *to_transfer_dst],
            );
        }

        let copy_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);
        let region = vk::ImageCopy::builder()
            .src_subresource(*copy_subresource)
            .src_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .dst_subresource(*copy_subresource)
            .dst_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .extent(vk::Extent3D {
                width: extent.width,
                height: extent.height,
                depth: 1,
            });

        unsafe {
            self.device.cmd_copy_image(
                cmd,
                src_image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                slot.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[*region],
            );
        }

        let back_to_present = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
            .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(src_image)
            .subresource_range(*subresource)
            .src_access_mask(vk::AccessFlags::TRANSFER_READ)
            .dst_access_mask(vk::AccessFlags::MEMORY_READ);

        let release_to_foreign = vk::ImageMemoryBarrier::builder()
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .src_queue_family_index(queue_family_index)
            .dst_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
            .image(slot.image)
            .subresource_range(*subresource)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::empty());

        unsafe {
            self.device.cmd_pipeline_barrier(
                cmd,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::ALL_COMMANDS,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[*back_to_present, *release_to_foreign],
            );
        }

        if let Err(e) = unsafe { self.device.end_command_buffer(cmd) } {
            log!("end_command_buffer failed: {e:?}");
            return false;
        }

        if let Err(e) = unsafe {
            self.device.queue_submit(
                queue,
                &[*vk::SubmitInfo::builder()
                    .command_buffers(&[cmd])
                    .signal_semaphores(&[slot.ready_semaphore])],
                slot.fence,
            )
        } {
            log!("queue_submit failed: {e:?}");
            return false;
        }

        true
    }

    fn get_swapchain_images(&self, swapchain: vk::SwapchainKHR) -> Vec<vk::Image> {
        let mut count = 0u32;
        let mut res = unsafe {
            (self.procs.get_swapchain_images_khr)(
                self.device.handle(),
                swapchain,
                &mut count,
                std::ptr::null_mut(),
            )
        };
        if res != vk::Result::SUCCESS || count == 0 {
            log!("get_swapchain_images (count) failed: {res:?}");
            return Vec::new();
        }
        let mut images = vec![vk::Image::null(); count as usize];
        res = unsafe {
            (self.procs.get_swapchain_images_khr)(
                self.device.handle(),
                swapchain,
                &mut count,
                images.as_mut_ptr(),
            )
        };
        if res != vk::Result::SUCCESS {
            log!("get_swapchain_images (fetch) failed: {res:?}");
            return Vec::new();
        }
        images
    }

    pub(crate) fn capture_and_notify(
        &self,
        queue: vk::Queue,
        swapchain: vk::SwapchainKHR,
        image_index: usize,
    ) {
        let Some(queue_family_index) = self.queue_family_for(queue) else {
            log!("capture: unknown queue family for presenting queue, skipping capture this frame");
            return;
        };

        let mut map = self.swapchains.lock().unwrap();
        let Some(state) = map.get_mut(&swapchain.as_raw()) else {
            return;
        };
        let Some(&src_image) = state.images.get(image_index) else {
            return;
        };

        state.present_count += 1;
        let psn = state.present_count;
        let verbose = should_sample(psn);

        let slot_idx = state.next_slot;
        state.next_slot = (slot_idx + 1) % CAPTURE_SLOT_COUNT;

        if state.slots.len() == slot_idx {
            let format = state.format;
            let extent = state.extent;
            let candidate_modifiers = state.candidate_modifiers.clone();
            match self.create_capture_slot(format, extent, &candidate_modifiers, queue_family_index)
            {
                Some(slot) => state.slots.push(slot),
                None => {
                    log!("capture: failed to create capture slot, skipping capture this frame");
                    return;
                }
            }
        } else if state.slots.len() < slot_idx {
            return;
        }

        let format = state.format;
        let extent = state.extent;
        let slot = &mut state.slots[slot_idx];

        if slot.fence_pending {
            let status = unsafe { self.device.get_fence_status(slot.fence) };
            match status {
                Ok(true) => slot.fence_pending = false,
                Ok(false) => match unsafe {
                    self.device
                        .wait_for_fences(&[slot.fence], true, SLOT_FENCE_TIMEOUT_NS)
                } {
                    Ok(()) => slot.fence_pending = false,
                    Err(_) => {
                        log!(
                            "capture: slot {slot_idx} still busy after {SLOT_FENCE_TIMEOUT_NS}ns, skipping capture this frame"
                        );
                        return;
                    }
                },
                Err(e) => {
                    log!("capture: get_fence_status failed: {e:?}, skipping capture this frame");
                    return;
                }
            }
        }

        if let Err(e) = unsafe { self.device.reset_fences(&[slot.fence]) } {
            log!("reset_fences failed: {e:?}");
            return;
        }
        if let Err(e) = unsafe {
            self.device
                .reset_command_buffer(slot.cmd, vk::CommandBufferResetFlags::empty())
        } {
            log!("reset_command_buffer failed: {e:?}");
            return;
        }

        let ok = self.record_and_submit_copy(queue, queue_family_index, slot, src_image, extent);
        if verbose {
            log!("swapchain={swapchain:?} present #{psn}: slot={slot_idx} copy submitted -> {ok}");
        }
        if !ok {
            return;
        }

        slot.ownership = SlotOwnership::ReleasedToForeign;
        slot.fence_pending = true;
        let need_buf_msg = !slot.buf_sent;
        let (stride, modifier, memory, ready_semaphore) = (
            slot.stride,
            slot.modifier,
            slot.memory,
            slot.ready_semaphore,
        );

        drop(map);

        let sync_get_fd_info = vk::SemaphoreGetFdInfoKHR::builder()
            .semaphore(ready_semaphore)
            .handle_type(vk::ExternalSemaphoreHandleTypeFlags::SYNC_FD);
        let mut sync_fd: RawFd = -1;
        let sync_res = unsafe {
            (self.procs.get_semaphore_fd_khr)(
                self.device.handle(),
                &*sync_get_fd_info,
                &mut sync_fd,
            )
        };
        if sync_res != vk::Result::SUCCESS {
            log!("vkGetSemaphoreFdKHR failed: {sync_res:?}, consumer will sample unsynchronized");
            sync_fd = -1;
        }

        if need_buf_msg {
            let get_fd_info = vk::MemoryGetFdInfoKHR::builder()
                .memory(memory)
                .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
            let mut fd: RawFd = -1;
            let res = unsafe {
                (self.procs.get_memory_fd_khr)(self.device.handle(), &*get_fd_info, &mut fd)
            };
            if res != vk::Result::SUCCESS {
                log!("vkGetMemoryFdKHR failed: {res:?}");
                if sync_fd >= 0 {
                    unsafe { libc::close(sync_fd) };
                }
                return;
            }
            let sent = self.capture_link.notify_buffer(BufferAnnounce {
                slot: slot_idx as u32,
                width: extent.width,
                height: extent.height,
                format,
                stride,
                modifier,
                fd,
                sync_fd,
            });
            if sent {
                if let Some(state) = self.swapchains.lock().unwrap().get_mut(&swapchain.as_raw()) {
                    if let Some(slot) = state.slots.get_mut(slot_idx) {
                        slot.buf_sent = true;
                    }
                }
            }
        } else {
            self.capture_link.notify_frame(slot_idx as u32, sync_fd);
        }
    }
}
