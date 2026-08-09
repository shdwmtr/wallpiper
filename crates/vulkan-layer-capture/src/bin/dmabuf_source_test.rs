use ash::vk;
use std::io::Write;
use std::os::unix::io::RawFd;
use std::os::unix::net::UnixStream;

const SOCKET_PATH: &str = "/tmp/wallpiper-capture.sock";
const WIDTH: u32 = 256;
const HEIGHT: u32 = 256;

macro_rules! logln {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        let _ = std::io::stdout().flush();
    }};
}

unsafe fn send_fd(stream: &UnixStream, header: &str, fd: RawFd) {
    use std::os::unix::io::AsRawFd;

    let header_bytes = header.as_bytes();
    let mut iov = libc::iovec {
        iov_base: header_bytes.as_ptr() as *mut libc::c_void,
        iov_len: header_bytes.len(),
    };

    let mut cmsg_buf = [0u8; 64];
    let mut msg: libc::msghdr = std::mem::zeroed();
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = libc::CMSG_SPACE(std::mem::size_of::<libc::c_int>() as u32) as usize;

    let cmsg = libc::CMSG_FIRSTHDR(&msg);
    (*cmsg).cmsg_level = libc::SOL_SOCKET;
    (*cmsg).cmsg_type = libc::SCM_RIGHTS;
    (*cmsg).cmsg_len = libc::CMSG_LEN(std::mem::size_of::<libc::c_int>() as u32) as usize;
    let data_ptr = libc::CMSG_DATA(cmsg) as *mut libc::c_int;
    std::ptr::write_unaligned(data_ptr, fd);

    let n = libc::sendmsg(stream.as_raw_fd(), &msg, 0);
    logln!("sendmsg raw return: {n} bytes (header was {} bytes)", header_bytes.len());
}

fn main() {
    let entry = unsafe { ash::Entry::load() }.expect("load vulkan entry");
    let app_info = vk::ApplicationInfo::builder().api_version(vk::API_VERSION_1_1);
    let instance_info = vk::InstanceCreateInfo::builder().application_info(&app_info);
    let instance = unsafe { entry.create_instance(&instance_info, None) }.expect("create_instance");

    let phys_devices = unsafe { instance.enumerate_physical_devices() }.expect("enumerate");
    let phys_device = phys_devices[0];
    let props = unsafe { instance.get_physical_device_properties(phys_device) };
    let name = unsafe {
        std::ffi::CStr::from_ptr(props.device_name.as_ptr())
            .to_string_lossy()
            .into_owned()
    };
    logln!("using physical device: {name}");

    let priorities = [1.0f32];
    let queue_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(0)
        .queue_priorities(&priorities)
        .build();
    let queue_infos = [queue_info];

    let ext_names = [
        ash::extensions::khr::ExternalMemoryFd::name().as_ptr(),
        vk::ExtExternalMemoryDmaBufFn::name().as_ptr(),
        vk::ExtImageDrmFormatModifierFn::name().as_ptr(),
        vk::KhrImageFormatListFn::name().as_ptr(),
    ];

    let device_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&ext_names);

    let device = unsafe { instance.create_device(phys_device, &device_info, None) }
        .expect("create_device");

    let format = vk::Format::B8G8R8A8_UNORM;

    let mut modifier_list_query = vk::DrmFormatModifierPropertiesListEXT::builder();
    let mut format_props2 = vk::FormatProperties2::builder().push_next(&mut modifier_list_query);
    unsafe { instance.get_physical_device_format_properties2(phys_device, format, &mut format_props2) };
    let modifier_count = modifier_list_query.drm_format_modifier_count;
    logln!("driver reports {modifier_count} supported DRM format modifiers for {format:?}");

    let mut modifier_props = vec![vk::DrmFormatModifierPropertiesEXT::default(); modifier_count as usize];
    let mut modifier_list_query2 =
        vk::DrmFormatModifierPropertiesListEXT::builder().drm_format_modifier_properties(&mut modifier_props);
    let mut format_props2b = vk::FormatProperties2::builder().push_next(&mut modifier_list_query2);
    unsafe { instance.get_physical_device_format_properties2(phys_device, format, &mut format_props2b) };

    for m in &modifier_props {
        logln!(
            "  modifier={} plane_count={} tiling_features={:?}",
            m.drm_format_modifier, m.drm_format_modifier_plane_count, m.drm_format_modifier_tiling_features
        );
    }

    let required = vk::FormatFeatureFlags::TRANSFER_DST | vk::FormatFeatureFlags::TRANSFER_SRC;
    let candidate_modifiers: Vec<u64> = modifier_props
        .iter()
        .filter(|m| m.drm_format_modifier_plane_count == 1 && m.drm_format_modifier_tiling_features.contains(required))
        .map(|m| m.drm_format_modifier)
        .collect();
    logln!("single-plane candidates usable for our transfer usage: {candidate_modifiers:?}");
    assert!(!candidate_modifiers.is_empty(), "no usable single-plane DRM format modifier found");

    let mut ext_image_info = vk::ExternalMemoryImageCreateInfo::builder()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
    let mut modifier_create_info =
        vk::ImageDrmFormatModifierListCreateInfoEXT::builder().drm_format_modifiers(&candidate_modifiers);

    let image_info = vk::ImageCreateInfo::builder()
        .push_next(&mut ext_image_info)
        .push_next(&mut modifier_create_info)
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D { width: WIDTH, height: HEIGHT, depth: 1 })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .tiling(vk::ImageTiling::DRM_FORMAT_MODIFIER_EXT)
        .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE)
        .initial_layout(vk::ImageLayout::UNDEFINED);

    let image = unsafe { device.create_image(&image_info, None) }.expect("create_image");
    logln!("created image: {image:?}");

    let drm_modifier_ext = ash::extensions::ext::ImageDrmFormatModifier::new(&instance, &device);
    let mut chosen_modifier_props = vk::ImageDrmFormatModifierPropertiesEXT::default();
    unsafe {
        drm_modifier_ext
            .get_image_drm_format_modifier_properties(image, &mut chosen_modifier_props)
            .expect("get_image_drm_format_modifier_properties")
    };
    let chosen_modifier = chosen_modifier_props.drm_format_modifier;
    logln!("driver chose modifier: {chosen_modifier}");

    let mem_req = unsafe { device.get_image_memory_requirements(image) };
    let mem_props = unsafe { instance.get_physical_device_memory_properties(phys_device) };
    let mem_type_index = (0..mem_props.memory_type_count)
        .find(|&i| (mem_req.memory_type_bits & (1 << i)) != 0)
        .expect("no compatible memory type");

    let mut export_info = vk::ExportMemoryAllocateInfo::builder()
        .handle_types(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
    let mut dedicated_info = vk::MemoryDedicatedAllocateInfo::builder().image(image);

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .push_next(&mut export_info)
        .push_next(&mut dedicated_info)
        .allocation_size(mem_req.size)
        .memory_type_index(mem_type_index);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }.expect("allocate_memory");
    unsafe { device.bind_image_memory(image, memory, 0) }.expect("bind_image_memory");
    logln!("bound memory: {memory:?}");

    let subresource = vk::ImageSubresource::builder()
        .aspect_mask(vk::ImageAspectFlags::MEMORY_PLANE_0_EXT)
        .mip_level(0)
        .array_layer(0);
    let layout = unsafe { device.get_image_subresource_layout(image, *subresource) };
    let stride = layout.row_pitch as u32;
    logln!("row_pitch (stride): {stride}, offset: {}", layout.offset);

    let get_memory_fd_khr: vk::PFN_vkGetMemoryFdKHR = unsafe {
        std::mem::transmute(
            instance
                .get_device_proc_addr(device.handle(), c"vkGetMemoryFdKHR".as_ptr())
                .expect("vkGetMemoryFdKHR not found"),
        )
    };

    let fd_info = vk::MemoryGetFdInfoKHR::builder()
        .memory(memory)
        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT);
    let mut fd: RawFd = -1;
    let result = unsafe { get_memory_fd_khr(device.handle(), &*fd_info, &mut fd) };
    assert_eq!(result, vk::Result::SUCCESS, "vkGetMemoryFdKHR failed");
    logln!("exported dma-buf fd={fd}");

    let pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(0);
    let pool = unsafe { device.create_command_pool(&pool_info, None) }.expect("create_command_pool");
    let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd = unsafe { device.allocate_command_buffers(&cmd_alloc_info) }.expect("alloc cmd buf")[0];

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(cmd, &begin_info) }.expect("begin_command_buffer");

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(1)
        .base_array_layer(0)
        .layer_count(1);

    let to_transfer_dst = vk::ImageMemoryBarrier::builder()
        .old_layout(vk::ImageLayout::UNDEFINED)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(*subresource_range)
        .src_access_mask(vk::AccessFlags::empty())
        .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);

    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*to_transfer_dst],
        );
    }

    let clear_color = vk::ClearColorValue {
        float32: [1.0, 0.0, 0.0, 1.0],
    };
    unsafe {
        device.cmd_clear_color_image(
            cmd,
            image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            &clear_color,
            &[*subresource_range],
        );
    }

    let release_to_foreign = vk::ImageMemoryBarrier::builder()
        .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        .src_queue_family_index(0)
        .dst_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
        .image(image)
        .subresource_range(*subresource_range)
        .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
        .dst_access_mask(vk::AccessFlags::empty());

    unsafe {
        device.cmd_pipeline_barrier(
            cmd,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &[*release_to_foreign],
        );
    }

    unsafe { device.end_command_buffer(cmd) }.expect("end_command_buffer");

    let fence_info = vk::FenceCreateInfo::builder();
    let fence = unsafe { device.create_fence(&fence_info, None) }.expect("create_fence");
    let cmds = [cmd];
    let submit_info = vk::SubmitInfo::builder().command_buffers(&cmds);
    let queue = unsafe { device.get_device_queue(0, 0) };
    unsafe { device.queue_submit(queue, &[*submit_info], fence) }.expect("queue_submit");
    unsafe { device.wait_for_fences(&[fence], true, u64::MAX) }.expect("wait_for_fences");
    logln!("clear + foreign-queue-release submitted and complete");

    let stream = UnixStream::connect(SOCKET_PATH).expect("connect to capture socket");
    let header = format!("{WIDTH} {HEIGHT} {} {stride} {chosen_modifier}\n", format.as_raw());
    unsafe { send_fd(&stream, &header, fd) };
    logln!("sent fd to wallpiper-display, sleeping to keep it alive...");

    std::thread::sleep(std::time::Duration::from_secs(60));
}
