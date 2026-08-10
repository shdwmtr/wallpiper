# wallpiper

A runtime-interpose translation layer for [Wallpaper Engine](https://www.wallpaperengine.io/en) on top of [Proton GE](https://github.com/gloriouseggroll/proton-ge-custom) to allow native Wallpaper Engine support on Linux.

It's exactly what you'd expect, opposed to existing libraries like [linux-wallpaperengine](https://github.com/Almamu/linux-wallpaperengine) (arguably misled by AI), wallpiper runs Wallpaper Engine natively without re-inventing the wheel. 

If it works on Windows, it works on Linux. no headaches. 

## Component Model

#### wallpiperd
Control daemon, brokers vulkan user buffers between the capture pipeline and the active portal.

#### libwallpiper-preload.so
The interposed translation layer that re-implements and pipes Wallpaper Engines renderer buffer to the control daemon.  

#### VkLayer_wallpiper_capture
A [Khronos-layer-spec-conformant](https://github.com/KhronosGroup/Vulkan-Loader/tree/main/docs) Vulkan layer that captures each presented image as a [DMA-BUF](https://docs.kernel.org/driver-api/dma-buf.html) and hands it off over IPC.

#### wallpiper-protocol
Shared wire format for the [UDS](https://en.wikipedia.org/wiki/Unix_domain_socket) transport connecting the capture layer, `wallpiperd`, and the portals.


## License

Apache-2.0 see [`LICENSE.md`](./LICENSE.md).
