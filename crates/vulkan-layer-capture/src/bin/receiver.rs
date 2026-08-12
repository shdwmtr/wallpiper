use ash::vk;
use std::io::Write;
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixListener;

const SOCKET_PATH: &str = "/tmp/wallpiper-capture.sock";

macro_rules! logln {
    ($($arg:tt)*) => {{
        println!($($arg)*);
        let _ = std::io::stdout().flush();
    }};
}

unsafe fn recv_fd(sock_fd: RawFd) -> std::io::Result<(String, RawFd)> {
    let mut header_buf = [0u8; 256];
    let mut cmsg_buf = [0u8; 64];

    let mut iov = libc::iovec {
        iov_base: header_buf.as_mut_ptr() as *mut libc::c_void,
        iov_len: header_buf.len(),
    };

    let mut msg: libc::msghdr = std::mem::zeroed();
    msg.msg_iov = &mut iov;
    msg.msg_iovlen = 1;
    msg.msg_control = cmsg_buf.as_mut_ptr() as *mut libc::c_void;
    msg.msg_controllen = cmsg_buf.len();

    let n = libc::recvmsg(sock_fd, &mut msg, 0);
    if n < 0 {
        return Err(std::io::Error::last_os_error());
    }

    let header = String::from_utf8_lossy(&header_buf[..n as usize])
        .trim()
        .to_string();

    let cmsg = libc::CMSG_FIRSTHDR(&msg);
    if cmsg.is_null() {
        return Err(std::io::Error::other("no cmsg / no fd received"));
    }
    let data_ptr = libc::CMSG_DATA(cmsg) as *const libc::c_int;
    let fd = std::ptr::read_unaligned(data_ptr);

    Ok((header, fd))
}

fn main() {
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH).expect("bind capture socket");
    logln!("listening on {SOCKET_PATH}, waiting for a frame...");

    let (header, fd) = loop {
        logln!("calling accept()...");
        let (stream, addr) = listener.accept().expect("accept");
        logln!(
            "accept() returned, peer addr={addr:?}, fd={}",
            stream.as_raw_fd()
        );
        match unsafe { recv_fd(stream.as_raw_fd()) } {
            Ok((header, fd)) => {
                break (header, fd);
            }
            Err(e) => {
                logln!("recv_fd on this connection failed: {e}, trying next accept()");
                continue;
            }
        }
    };

    logln!("received header={header:?} fd={fd}");

    let parts: Vec<&str> = header.split_whitespace().collect();
    let width: u32 = parts[0].parse().expect("width");
    let height: u32 = parts[1].parse().expect("height");
    let format_raw: i32 = parts[2].parse().expect("format");
    let format = vk::Format::from_raw(format_raw);
    let stride: u32 = parts.get(3).and_then(|s| s.parse().ok()).unwrap_or(0);

    logln!("importing dmabuf fd={fd} as {width}x{height} format={format:?} stride={stride}");

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
    ];

    let device_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(&queue_infos)
        .enabled_extension_names(&ext_names);

    let device = unsafe { instance.create_device(phys_device, &device_info, None) }
        .expect("create_device (does it support the needed extensions?)");

    let row_stride = if stride > 0 {
        stride as u64
    } else {
        width as u64 * 4
    };
    let imported_size = row_stride * height as u64;

    let mut import_info = vk::ImportMemoryFdInfoKHR::builder()
        .handle_type(vk::ExternalMemoryHandleTypeFlags::DMA_BUF_EXT)
        .fd(fd);

    let imported_buffer_info = vk::BufferCreateInfo::builder()
        .size(imported_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let imported_buffer = unsafe { device.create_buffer(&imported_buffer_info, None) }
        .expect("create_buffer (imported)");
    logln!("created local buffer handle: {imported_buffer:?} size={imported_size}");

    let mem_req = unsafe { device.get_buffer_memory_requirements(imported_buffer) };
    let mem_props = unsafe { instance.get_physical_device_memory_properties(phys_device) };
    let mem_type_index = (0..mem_props.memory_type_count)
        .find(|&i| (mem_req.memory_type_bits & (1 << i)) != 0)
        .expect("no compatible memory type for import");

    let alloc_info = vk::MemoryAllocateInfo::builder()
        .push_next(&mut import_info)
        .allocation_size(mem_req.size)
        .memory_type_index(mem_type_index);

    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .expect("allocate_memory (import) - THIS is the real test");
    logln!("IMPORT SUCCEEDED: memory={memory:?}");

    unsafe { device.bind_buffer_memory(imported_buffer, memory, 0) }.expect("bind_buffer_memory");
    logln!("bind_buffer_memory succeeded — dmabuf round-trip is fully working (buffer path, no image tiling ambiguity)");

    let staging_info = vk::BufferCreateInfo::builder()
        .size(imported_size)
        .usage(vk::BufferUsageFlags::TRANSFER_DST)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);
    let staging_buffer =
        unsafe { device.create_buffer(&staging_info, None) }.expect("create_buffer (staging)");

    let stage_mem_req = unsafe { device.get_buffer_memory_requirements(staging_buffer) };
    let stage_mem_type_index = (0..mem_props.memory_type_count)
        .find(|&i| {
            (stage_mem_req.memory_type_bits & (1 << i)) != 0
                && mem_props.memory_types[i as usize].property_flags.contains(
                    vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
        })
        .expect("no host-visible memory type for readback buffer");
    let stage_alloc_info = vk::MemoryAllocateInfo::builder()
        .allocation_size(stage_mem_req.size)
        .memory_type_index(stage_mem_type_index);
    let staging_memory = unsafe { device.allocate_memory(&stage_alloc_info, None) }
        .expect("allocate_memory (staging)");
    unsafe { device.bind_buffer_memory(staging_buffer, staging_memory, 0) }
        .expect("bind_buffer_memory");

    let pool_info = vk::CommandPoolCreateInfo::builder().queue_family_index(0);
    let pool =
        unsafe { device.create_command_pool(&pool_info, None) }.expect("create_command_pool");
    let cmd_alloc_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let cmd =
        unsafe { device.allocate_command_buffers(&cmd_alloc_info) }.expect("alloc cmd buf")[0];

    let begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    unsafe { device.begin_command_buffer(cmd, &begin_info) }.expect("begin_command_buffer");

    let region = vk::BufferCopy::builder()
        .src_offset(0)
        .dst_offset(0)
        .size(imported_size);
    unsafe {
        device.cmd_copy_buffer(cmd, imported_buffer, staging_buffer, &[*region]);
    }

    unsafe { device.end_command_buffer(cmd) }.expect("end_command_buffer");

    let fence_info = vk::FenceCreateInfo::builder();
    let fence = unsafe { device.create_fence(&fence_info, None) }.expect("create_fence");
    let cmds = [cmd];
    let submit_info = vk::SubmitInfo::builder().command_buffers(&cmds);
    let queue = unsafe { device.get_device_queue(0, 0) };
    unsafe { device.queue_submit(queue, &[*submit_info], fence) }.expect("queue_submit");
    unsafe { device.wait_for_fences(&[fence], true, u64::MAX) }.expect("wait_for_fences");
    logln!("readback copy complete");

    let data_ptr = unsafe {
        device.map_memory(
            staging_memory,
            0,
            imported_size,
            vk::MemoryMapFlags::empty(),
        )
    }
    .expect("map_memory") as *const u8;
    let raw = unsafe { std::slice::from_raw_parts(data_ptr, imported_size as usize) };

    let sample_row_offset = (height as u64 / 2) * row_stride;
    let sample_offset = (sample_row_offset + (width as u64 / 2) * 4) as usize;
    logln!(
        "center pixel BGRA bytes: {:?}",
        &raw[sample_offset..sample_offset + 4]
    );
    let nonzero = raw.iter().any(|&b| b != 0);
    logln!("buffer has any nonzero bytes: {nonzero}");

    let ppm_path = "/tmp/wallpiper-capture.ppm";
    let mut ppm = std::fs::File::create(ppm_path).expect("create ppm");
    write!(ppm, "P6\n{width} {height}\n255\n").unwrap();
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    for row in 0..height as u64 {
        let row_start = (row * row_stride) as usize;
        let row_bytes = &raw[row_start..row_start + (width as usize * 4)];
        for chunk in row_bytes.chunks_exact(4) {
            rgb.push(chunk[2]);
            rgb.push(chunk[1]);
            rgb.push(chunk[0]);
        }
    }
    ppm.write_all(&rgb).unwrap();
    logln!("wrote {ppm_path}");

    unsafe { device.unmap_memory(staging_memory) };
}
