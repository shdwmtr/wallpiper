use std::ops::Deref;
use std::sync::{Arc, LazyLock};

use ash::vk;
use vulkan_layer::{declare_introspection_queries, Global, Layer, LayerManifest, StubGlobalHooks};

use crate::device::WallpiperDeviceInfo;
use crate::instance::{set_global_instance, WallpiperInstanceHooks};

#[derive(Default)]
pub(crate) struct WallpiperLayer(StubGlobalHooks);

impl Layer for WallpiperLayer {
    type GlobalHooksInfo = StubGlobalHooks;
    type InstanceInfo = WallpiperInstanceHooks;
    type DeviceInfo = WallpiperDeviceInfo;
    type InstanceInfoContainer = WallpiperInstanceHooks;
    type DeviceInfoContainer = WallpiperDeviceInfo;

    fn global_instance() -> impl Deref<Target = Global<Self>> + 'static {
        static GLOBAL: LazyLock<Global<WallpiperLayer>> = LazyLock::new(Default::default);
        &*GLOBAL
    }

    fn manifest() -> LayerManifest {
        let mut manifest = LayerManifest::default();
        manifest.name = "VK_LAYER_wallpiper_capture";
        manifest.spec_version = vk::API_VERSION_1_1;
        manifest.description = "Wallpiper frame capture layer";
        manifest
    }

    fn global_hooks_info(&self) -> &Self::GlobalHooksInfo {
        &self.0
    }

    fn create_instance_info(
        &self,
        _: &vk::InstanceCreateInfo,
        _: Option<&vk::AllocationCallbacks>,
        instance: Arc<ash::Instance>,
        _next_get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ) -> Self::InstanceInfoContainer {
        set_global_instance(instance);
        Default::default()
    }

    fn create_device_info(
        &self,
        physical_device: vk::PhysicalDevice,
        _: &vk::DeviceCreateInfo,
        _: Option<&vk::AllocationCallbacks>,
        device: Arc<ash::Device>,
        next_get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    ) -> Self::DeviceInfoContainer {
        WallpiperDeviceInfo::new(physical_device, device, next_get_device_proc_addr)
    }
}

declare_introspection_queries!(Global::<WallpiperLayer>);
