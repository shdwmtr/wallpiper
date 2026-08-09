// Copyright 2025 Google LLC
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

#![allow(missing_docs)]
// This file is generated from the Vulkan XML API registry.
#![allow(clippy::too_many_arguments)]
use std::{
    ffi::{c_int, c_void},
    mem::MaybeUninit,
};

use ash::{prelude::VkResult, vk};

use super::{LayerResult, TryFromVulkanCommandError};
use crate::VkLayerDeviceLink;

// Unhandled commands:
// * vkMapMemory2KHR: The ash Rust binding doesn't have proper bindings yet.
// * vkUnmapMemory2KHR: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdSetDiscardRectangleEnableEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdSetDiscardRectangleModeEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdSetExclusiveScissorEnableNV: The ash Rust binding doesn't have proper bindings yet.
// * vkGetDeviceFaultInfoEXT: The length info and the data pointer are nested in structs.
// * vkCmdDrawClusterHUAWEI: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdDrawClusterIndirectHUAWEI: The ash Rust binding doesn't have proper bindings yet.
// * vkCreateShadersEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkDestroyShaderEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkGetShaderBinaryDataEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdBindShadersEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdSetAttachmentFeedbackLoopEnableEXT: The ash Rust binding doesn't have proper bindings yet.
// * vkCmdBuildAccelerationStructuresKHR: Dynamic multi-dimensional array bindings are not supported
//   yet.
// * vkCmdBuildAccelerationStructuresIndirectKHR: Dynamic multi-dimensional array bindings are not
//   supported yet.
// * vkBuildAccelerationStructuresKHR: Dynamic multi-dimensional array bindings are not supported
//   yet.
#[derive(Debug, PartialOrd, Ord, PartialEq, Eq, Hash, Clone)]
pub enum VulkanCommand {
    CreateInstance,
    GetPhysicalDeviceFeatures,
    GetPhysicalDeviceFormatProperties,
    GetPhysicalDeviceImageFormatProperties,
    GetPhysicalDeviceProperties,
    GetPhysicalDeviceQueueFamilyProperties,
    GetPhysicalDeviceMemoryProperties,
    GetInstanceProcAddr,
    CreateDevice,
    GetPhysicalDeviceSparseImageFormatProperties,
    GetPhysicalDeviceFeatures2,
    GetPhysicalDeviceProperties2,
    GetPhysicalDeviceFormatProperties2,
    GetPhysicalDeviceImageFormatProperties2,
    GetPhysicalDeviceQueueFamilyProperties2,
    GetPhysicalDeviceMemoryProperties2,
    GetPhysicalDeviceSparseImageFormatProperties2,
    GetPhysicalDeviceExternalBufferProperties,
    GetPhysicalDeviceExternalFenceProperties,
    GetPhysicalDeviceExternalSemaphoreProperties,
    GetPhysicalDeviceToolProperties,
    DestroySurfaceKhr,
    GetPhysicalDeviceSurfaceSupportKhr,
    GetPhysicalDeviceSurfaceCapabilitiesKhr,
    GetPhysicalDeviceSurfaceFormatsKhr,
    GetPhysicalDeviceSurfacePresentModesKhr,
    GetPhysicalDevicePresentRectanglesKhr,
    GetPhysicalDeviceDisplayPropertiesKhr,
    GetPhysicalDeviceDisplayPlanePropertiesKhr,
    GetDisplayPlaneSupportedDisplaysKhr,
    GetDisplayModePropertiesKhr,
    CreateDisplayModeKhr,
    GetDisplayPlaneCapabilitiesKhr,
    CreateDisplayPlaneSurfaceKhr,
    CreateXlibSurfaceKhr,
    GetPhysicalDeviceXlibPresentationSupportKhr,
    CreateXcbSurfaceKhr,
    GetPhysicalDeviceXcbPresentationSupportKhr,
    CreateWaylandSurfaceKhr,
    GetPhysicalDeviceWaylandPresentationSupportKhr,
    CreateAndroidSurfaceKhr,
    CreateWin32SurfaceKhr,
    GetPhysicalDeviceWin32PresentationSupportKhr,
    GetPhysicalDeviceVideoCapabilitiesKhr,
    GetPhysicalDeviceVideoFormatPropertiesKhr,
    EnumeratePhysicalDeviceQueueFamilyPerformanceQueryCountersKhr,
    GetPhysicalDeviceQueueFamilyPerformanceQueryPassesKhr,
    GetPhysicalDeviceSurfaceCapabilities2Khr,
    GetPhysicalDeviceSurfaceFormats2Khr,
    GetPhysicalDeviceDisplayProperties2Khr,
    GetPhysicalDeviceDisplayPlaneProperties2Khr,
    GetDisplayModeProperties2Khr,
    GetDisplayPlaneCapabilities2Khr,
    GetPhysicalDeviceFragmentShadingRatesKhr,
    CreateDebugReportCallbackExt,
    DestroyDebugReportCallbackExt,
    DebugReportMessageExt,
    CreateStreamDescriptorSurfaceGgp,
    GetPhysicalDeviceExternalImageFormatPropertiesNv,
    CreateViSurfaceNn,
    ReleaseDisplayExt,
    AcquireXlibDisplayExt,
    GetRandROutputDisplayExt,
    GetPhysicalDeviceSurfaceCapabilities2Ext,
    CreateIosSurfaceMvk,
    CreateMacOsSurfaceMvk,
    CreateDebugUtilsMessengerExt,
    DestroyDebugUtilsMessengerExt,
    SubmitDebugUtilsMessageExt,
    GetPhysicalDeviceMultisamplePropertiesExt,
    GetPhysicalDeviceCalibrateableTimeDomainsExt,
    CreateImagePipeSurfaceFuchsia,
    CreateMetalSurfaceExt,
    GetPhysicalDeviceCooperativeMatrixPropertiesNv,
    GetPhysicalDeviceSupportedFramebufferMixedSamplesCombinationsNv,
    GetPhysicalDeviceSurfacePresentModes2Ext,
    CreateHeadlessSurfaceExt,
    AcquireDrmDisplayExt,
    GetDrmDisplayExt,
    AcquireWinrtDisplayNv,
    GetWinrtDisplayNv,
    CreateDirectFbSurfaceExt,
    GetPhysicalDeviceDirectFbPresentationSupportExt,
    CreateScreenSurfaceQnx,
    GetPhysicalDeviceScreenPresentationSupportQnx,
    GetPhysicalDeviceOpticalFlowImageFormatsNv,
    GetDeviceProcAddr,
    GetDeviceQueue,
    QueueSubmit,
    QueueWaitIdle,
    DeviceWaitIdle,
    AllocateMemory,
    FreeMemory,
    MapMemory,
    UnmapMemory,
    FlushMappedMemoryRanges,
    InvalidateMappedMemoryRanges,
    GetDeviceMemoryCommitment,
    BindBufferMemory,
    BindImageMemory,
    GetBufferMemoryRequirements,
    GetImageMemoryRequirements,
    GetImageSparseMemoryRequirements,
    QueueBindSparse,
    CreateFence,
    DestroyFence,
    ResetFences,
    GetFenceStatus,
    WaitForFences,
    CreateSemaphore,
    DestroySemaphore,
    CreateEvent,
    DestroyEvent,
    GetEventStatus,
    SetEvent,
    ResetEvent,
    CreateQueryPool,
    DestroyQueryPool,
    GetQueryPoolResults,
    CreateBuffer,
    DestroyBuffer,
    CreateBufferView,
    DestroyBufferView,
    CreateImage,
    DestroyImage,
    GetImageSubresourceLayout,
    CreateImageView,
    DestroyImageView,
    CreateShaderModule,
    DestroyShaderModule,
    CreatePipelineCache,
    DestroyPipelineCache,
    GetPipelineCacheData,
    MergePipelineCaches,
    CreateGraphicsPipelines,
    CreateComputePipelines,
    DestroyPipeline,
    CreatePipelineLayout,
    DestroyPipelineLayout,
    CreateSampler,
    DestroySampler,
    CreateDescriptorSetLayout,
    DestroyDescriptorSetLayout,
    CreateDescriptorPool,
    DestroyDescriptorPool,
    ResetDescriptorPool,
    AllocateDescriptorSets,
    FreeDescriptorSets,
    UpdateDescriptorSets,
    CreateFramebuffer,
    DestroyFramebuffer,
    CreateRenderPass,
    DestroyRenderPass,
    GetRenderAreaGranularity,
    CreateCommandPool,
    DestroyCommandPool,
    ResetCommandPool,
    AllocateCommandBuffers,
    FreeCommandBuffers,
    BeginCommandBuffer,
    EndCommandBuffer,
    ResetCommandBuffer,
    CmdBindPipeline,
    CmdSetViewport,
    CmdSetScissor,
    CmdSetLineWidth,
    CmdSetDepthBias,
    CmdSetBlendConstants,
    CmdSetDepthBounds,
    CmdSetStencilCompareMask,
    CmdSetStencilWriteMask,
    CmdSetStencilReference,
    CmdBindDescriptorSets,
    CmdBindIndexBuffer,
    CmdBindVertexBuffers,
    CmdDraw,
    CmdDrawIndexed,
    CmdDrawIndirect,
    CmdDrawIndexedIndirect,
    CmdDispatch,
    CmdDispatchIndirect,
    CmdCopyBuffer,
    CmdCopyImage,
    CmdBlitImage,
    CmdCopyBufferToImage,
    CmdCopyImageToBuffer,
    CmdUpdateBuffer,
    CmdFillBuffer,
    CmdClearColorImage,
    CmdClearDepthStencilImage,
    CmdClearAttachments,
    CmdResolveImage,
    CmdSetEvent,
    CmdResetEvent,
    CmdWaitEvents,
    CmdPipelineBarrier,
    CmdBeginQuery,
    CmdEndQuery,
    CmdResetQueryPool,
    CmdWriteTimestamp,
    CmdCopyQueryPoolResults,
    CmdPushConstants,
    CmdBeginRenderPass,
    CmdNextSubpass,
    CmdEndRenderPass,
    CmdExecuteCommands,
    BindBufferMemory2,
    BindImageMemory2,
    GetDeviceGroupPeerMemoryFeatures,
    CmdSetDeviceMask,
    CmdDispatchBase,
    GetImageMemoryRequirements2,
    GetBufferMemoryRequirements2,
    GetImageSparseMemoryRequirements2,
    TrimCommandPool,
    GetDeviceQueue2,
    CreateSamplerYcbcrConversion,
    DestroySamplerYcbcrConversion,
    CreateDescriptorUpdateTemplate,
    DestroyDescriptorUpdateTemplate,
    UpdateDescriptorSetWithTemplate,
    GetDescriptorSetLayoutSupport,
    CmdDrawIndirectCount,
    CmdDrawIndexedIndirectCount,
    CreateRenderPass2,
    CmdBeginRenderPass2,
    CmdNextSubpass2,
    CmdEndRenderPass2,
    ResetQueryPool,
    GetSemaphoreCounterValue,
    WaitSemaphores,
    SignalSemaphore,
    GetBufferDeviceAddress,
    GetBufferOpaqueCaptureAddress,
    GetDeviceMemoryOpaqueCaptureAddress,
    CreatePrivateDataSlot,
    DestroyPrivateDataSlot,
    SetPrivateData,
    GetPrivateData,
    CmdSetEvent2,
    CmdResetEvent2,
    CmdWaitEvents2,
    CmdPipelineBarrier2,
    CmdWriteTimestamp2,
    QueueSubmit2,
    CmdCopyBuffer2,
    CmdCopyImage2,
    CmdCopyBufferToImage2,
    CmdCopyImageToBuffer2,
    CmdBlitImage2,
    CmdResolveImage2,
    CmdBeginRendering,
    CmdEndRendering,
    CmdSetCullMode,
    CmdSetFrontFace,
    CmdSetPrimitiveTopology,
    CmdSetViewportWithCount,
    CmdSetScissorWithCount,
    CmdBindVertexBuffers2,
    CmdSetDepthTestEnable,
    CmdSetDepthWriteEnable,
    CmdSetDepthCompareOp,
    CmdSetDepthBoundsTestEnable,
    CmdSetStencilTestEnable,
    CmdSetStencilOp,
    CmdSetRasterizerDiscardEnable,
    CmdSetDepthBiasEnable,
    CmdSetPrimitiveRestartEnable,
    GetDeviceBufferMemoryRequirements,
    GetDeviceImageMemoryRequirements,
    GetDeviceImageSparseMemoryRequirements,
    CreateSwapchainKhr,
    DestroySwapchainKhr,
    GetSwapchainImagesKhr,
    AcquireNextImageKhr,
    QueuePresentKhr,
    GetDeviceGroupPresentCapabilitiesKhr,
    GetDeviceGroupSurfacePresentModesKhr,
    AcquireNextImage2Khr,
    CreateSharedSwapchainsKhr,
    CreateVideoSessionKhr,
    DestroyVideoSessionKhr,
    GetVideoSessionMemoryRequirementsKhr,
    BindVideoSessionMemoryKhr,
    CreateVideoSessionParametersKhr,
    UpdateVideoSessionParametersKhr,
    DestroyVideoSessionParametersKhr,
    CmdBeginVideoCodingKhr,
    CmdEndVideoCodingKhr,
    CmdControlVideoCodingKhr,
    CmdDecodeVideoKhr,
    GetMemoryWin32HandleKhr,
    GetMemoryWin32HandlePropertiesKhr,
    GetMemoryFdKhr,
    GetMemoryFdPropertiesKhr,
    ImportSemaphoreWin32HandleKhr,
    GetSemaphoreWin32HandleKhr,
    ImportSemaphoreFdKhr,
    GetSemaphoreFdKhr,
    CmdPushDescriptorSetKhr,
    CmdPushDescriptorSetWithTemplateKhr,
    GetSwapchainStatusKhr,
    ImportFenceWin32HandleKhr,
    GetFenceWin32HandleKhr,
    ImportFenceFdKhr,
    GetFenceFdKhr,
    AcquireProfilingLockKhr,
    ReleaseProfilingLockKhr,
    CmdSetFragmentShadingRateKhr,
    WaitForPresentKhr,
    CreateDeferredOperationKhr,
    DestroyDeferredOperationKhr,
    GetDeferredOperationMaxConcurrencyKhr,
    GetDeferredOperationResultKhr,
    DeferredOperationJoinKhr,
    GetPipelineExecutablePropertiesKhr,
    GetPipelineExecutableStatisticsKhr,
    GetPipelineExecutableInternalRepresentationsKhr,
    CmdEncodeVideoKhr,
    CmdWriteBufferMarker2Amd,
    GetQueueCheckpointData2Nv,
    CmdTraceRaysIndirect2Khr,
    GetSwapchainGrallocUsageAndroid,
    AcquireImageAndroid,
    QueueSignalReleaseImageAndroid,
    GetSwapchainGrallocUsage2Android,
    DebugMarkerSetObjectTagExt,
    DebugMarkerSetObjectNameExt,
    CmdDebugMarkerBeginExt,
    CmdDebugMarkerEndExt,
    CmdDebugMarkerInsertExt,
    CmdBindTransformFeedbackBuffersExt,
    CmdBeginTransformFeedbackExt,
    CmdEndTransformFeedbackExt,
    CmdBeginQueryIndexedExt,
    CmdEndQueryIndexedExt,
    CmdDrawIndirectByteCountExt,
    CreateCuModuleNvx,
    CreateCuFunctionNvx,
    DestroyCuModuleNvx,
    DestroyCuFunctionNvx,
    CmdCuLaunchKernelNvx,
    GetImageViewHandleNvx,
    GetImageViewAddressNvx,
    GetShaderInfoAmd,
    GetMemoryWin32HandleNv,
    CmdBeginConditionalRenderingExt,
    CmdEndConditionalRenderingExt,
    CmdSetViewportWScalingNv,
    DisplayPowerControlExt,
    RegisterDeviceEventExt,
    RegisterDisplayEventExt,
    GetSwapchainCounterExt,
    GetRefreshCycleDurationGoogle,
    GetPastPresentationTimingGoogle,
    CmdSetDiscardRectangleExt,
    SetHdrMetadataExt,
    SetDebugUtilsObjectNameExt,
    SetDebugUtilsObjectTagExt,
    QueueBeginDebugUtilsLabelExt,
    QueueEndDebugUtilsLabelExt,
    QueueInsertDebugUtilsLabelExt,
    CmdBeginDebugUtilsLabelExt,
    CmdEndDebugUtilsLabelExt,
    CmdInsertDebugUtilsLabelExt,
    GetAndroidHardwareBufferPropertiesAndroid,
    GetMemoryAndroidHardwareBufferAndroid,
    CmdSetSampleLocationsExt,
    GetImageDrmFormatModifierPropertiesExt,
    CreateValidationCacheExt,
    DestroyValidationCacheExt,
    MergeValidationCachesExt,
    GetValidationCacheDataExt,
    CmdBindShadingRateImageNv,
    CmdSetViewportShadingRatePaletteNv,
    CmdSetCoarseSampleOrderNv,
    CreateAccelerationStructureNv,
    DestroyAccelerationStructureNv,
    GetAccelerationStructureMemoryRequirementsNv,
    BindAccelerationStructureMemoryNv,
    CmdBuildAccelerationStructureNv,
    CmdCopyAccelerationStructureNv,
    CmdTraceRaysNv,
    CreateRayTracingPipelinesNv,
    GetRayTracingShaderGroupHandlesKhr,
    GetAccelerationStructureHandleNv,
    CmdWriteAccelerationStructuresPropertiesNv,
    CompileDeferredNv,
    GetMemoryHostPointerPropertiesExt,
    CmdWriteBufferMarkerAmd,
    GetCalibratedTimestampsExt,
    CmdDrawMeshTasksNv,
    CmdDrawMeshTasksIndirectNv,
    CmdDrawMeshTasksIndirectCountNv,
    CmdSetExclusiveScissorNv,
    CmdSetCheckpointNv,
    GetQueueCheckpointDataNv,
    InitializePerformanceApiIntel,
    UninitializePerformanceApiIntel,
    CmdSetPerformanceMarkerIntel,
    CmdSetPerformanceStreamMarkerIntel,
    CmdSetPerformanceOverrideIntel,
    AcquirePerformanceConfigurationIntel,
    ReleasePerformanceConfigurationIntel,
    QueueSetPerformanceConfigurationIntel,
    GetPerformanceParameterIntel,
    SetLocalDimmingAmd,
    AcquireFullScreenExclusiveModeExt,
    ReleaseFullScreenExclusiveModeExt,
    GetDeviceGroupSurfacePresentModes2Ext,
    CmdSetLineStippleExt,
    ReleaseSwapchainImagesExt,
    GetGeneratedCommandsMemoryRequirementsNv,
    CmdPreprocessGeneratedCommandsNv,
    CmdExecuteGeneratedCommandsNv,
    CmdBindPipelineShaderGroupNv,
    CreateIndirectCommandsLayoutNv,
    DestroyIndirectCommandsLayoutNv,
    ExportMetalObjectsExt,
    GetDescriptorSetLayoutSizeExt,
    GetDescriptorSetLayoutBindingOffsetExt,
    GetDescriptorExt,
    CmdBindDescriptorBuffersExt,
    CmdSetDescriptorBufferOffsetsExt,
    CmdBindDescriptorBufferEmbeddedSamplersExt,
    GetBufferOpaqueCaptureDescriptorDataExt,
    GetImageOpaqueCaptureDescriptorDataExt,
    GetImageViewOpaqueCaptureDescriptorDataExt,
    GetSamplerOpaqueCaptureDescriptorDataExt,
    GetAccelerationStructureOpaqueCaptureDescriptorDataExt,
    CmdSetFragmentShadingRateEnumNv,
    GetImageSubresourceLayout2Ext,
    CmdSetVertexInputExt,
    GetMemoryZirconHandleFuchsia,
    GetMemoryZirconHandlePropertiesFuchsia,
    ImportSemaphoreZirconHandleFuchsia,
    GetSemaphoreZirconHandleFuchsia,
    CreateBufferCollectionFuchsia,
    SetBufferCollectionImageConstraintsFuchsia,
    SetBufferCollectionBufferConstraintsFuchsia,
    DestroyBufferCollectionFuchsia,
    GetBufferCollectionPropertiesFuchsia,
    GetDeviceSubpassShadingMaxWorkgroupSizeHuawei,
    CmdSubpassShadingHuawei,
    CmdBindInvocationMaskHuawei,
    GetMemoryRemoteAddressNv,
    GetPipelinePropertiesExt,
    CmdSetPatchControlPointsExt,
    CmdSetLogicOpExt,
    CmdSetColorWriteEnableExt,
    CmdDrawMultiExt,
    CmdDrawMultiIndexedExt,
    CreateMicromapExt,
    DestroyMicromapExt,
    CmdBuildMicromapsExt,
    BuildMicromapsExt,
    CopyMicromapExt,
    CopyMicromapToMemoryExt,
    CopyMemoryToMicromapExt,
    WriteMicromapsPropertiesExt,
    CmdCopyMicromapExt,
    CmdCopyMicromapToMemoryExt,
    CmdCopyMemoryToMicromapExt,
    CmdWriteMicromapsPropertiesExt,
    GetDeviceMicromapCompatibilityExt,
    GetMicromapBuildSizesExt,
    SetDeviceMemoryPriorityExt,
    GetDescriptorSetLayoutHostMappingInfoValve,
    GetDescriptorSetHostMappingValve,
    CmdCopyMemoryIndirectNv,
    CmdCopyMemoryToImageIndirectNv,
    CmdDecompressMemoryNv,
    CmdDecompressMemoryIndirectCountNv,
    CmdSetTessellationDomainOriginExt,
    CmdSetDepthClampEnableExt,
    CmdSetPolygonModeExt,
    CmdSetRasterizationSamplesExt,
    CmdSetSampleMaskExt,
    CmdSetAlphaToCoverageEnableExt,
    CmdSetAlphaToOneEnableExt,
    CmdSetLogicOpEnableExt,
    CmdSetColorBlendEnableExt,
    CmdSetColorBlendEquationExt,
    CmdSetColorWriteMaskExt,
    CmdSetRasterizationStreamExt,
    CmdSetConservativeRasterizationModeExt,
    CmdSetExtraPrimitiveOverestimationSizeExt,
    CmdSetDepthClipEnableExt,
    CmdSetSampleLocationsEnableExt,
    CmdSetColorBlendAdvancedExt,
    CmdSetProvokingVertexModeExt,
    CmdSetLineRasterizationModeExt,
    CmdSetLineStippleEnableExt,
    CmdSetDepthClipNegativeOneToOneExt,
    CmdSetViewportWScalingEnableNv,
    CmdSetViewportSwizzleNv,
    CmdSetCoverageToColorEnableNv,
    CmdSetCoverageToColorLocationNv,
    CmdSetCoverageModulationModeNv,
    CmdSetCoverageModulationTableEnableNv,
    CmdSetCoverageModulationTableNv,
    CmdSetShadingRateImageEnableNv,
    CmdSetRepresentativeFragmentTestEnableNv,
    CmdSetCoverageReductionModeNv,
    GetShaderModuleIdentifierExt,
    GetShaderModuleCreateInfoIdentifierExt,
    CreateOpticalFlowSessionNv,
    DestroyOpticalFlowSessionNv,
    BindOpticalFlowSessionImageNv,
    CmdOpticalFlowExecuteNv,
    GetFramebufferTilePropertiesQcom,
    GetDynamicRenderingTilePropertiesQcom,
    CreateAccelerationStructureKhr,
    DestroyAccelerationStructureKhr,
    CopyAccelerationStructureKhr,
    CopyAccelerationStructureToMemoryKhr,
    CopyMemoryToAccelerationStructureKhr,
    WriteAccelerationStructuresPropertiesKhr,
    CmdCopyAccelerationStructureKhr,
    CmdCopyAccelerationStructureToMemoryKhr,
    CmdCopyMemoryToAccelerationStructureKhr,
    GetAccelerationStructureDeviceAddressKhr,
    CmdWriteAccelerationStructuresPropertiesKhr,
    GetDeviceAccelerationStructureCompatibilityKhr,
    GetAccelerationStructureBuildSizesKhr,
    CmdTraceRaysKhr,
    CreateRayTracingPipelinesKhr,
    GetRayTracingCaptureReplayShaderGroupHandlesKhr,
    CmdTraceRaysIndirectKhr,
    GetRayTracingShaderGroupStackSizeKhr,
    CmdSetRayTracingPipelineStackSizeKhr,
    CmdDrawMeshTasksExt,
    CmdDrawMeshTasksIndirectExt,
    CmdDrawMeshTasksIndirectCountExt,
}

impl TryFrom<&str> for VulkanCommand {
    type Error = TryFromVulkanCommandError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "vkCreateInstance" => Ok(VulkanCommand::CreateInstance),
            "vkGetPhysicalDeviceFeatures" => Ok(VulkanCommand::GetPhysicalDeviceFeatures),
            "vkGetPhysicalDeviceFormatProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceFormatProperties)
            }
            "vkGetPhysicalDeviceImageFormatProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceImageFormatProperties)
            }
            "vkGetPhysicalDeviceProperties" => Ok(VulkanCommand::GetPhysicalDeviceProperties),
            "vkGetPhysicalDeviceQueueFamilyProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceQueueFamilyProperties)
            }
            "vkGetPhysicalDeviceMemoryProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceMemoryProperties)
            }
            "vkGetInstanceProcAddr" => Ok(VulkanCommand::GetInstanceProcAddr),
            "vkCreateDevice" => Ok(VulkanCommand::CreateDevice),
            "vkGetPhysicalDeviceSparseImageFormatProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceSparseImageFormatProperties)
            }
            "vkGetPhysicalDeviceFeatures2" => Ok(VulkanCommand::GetPhysicalDeviceFeatures2),
            "vkGetPhysicalDeviceProperties2" => Ok(VulkanCommand::GetPhysicalDeviceProperties2),
            "vkGetPhysicalDeviceFormatProperties2" => {
                Ok(VulkanCommand::GetPhysicalDeviceFormatProperties2)
            }
            "vkGetPhysicalDeviceImageFormatProperties2" => {
                Ok(VulkanCommand::GetPhysicalDeviceImageFormatProperties2)
            }
            "vkGetPhysicalDeviceQueueFamilyProperties2" => {
                Ok(VulkanCommand::GetPhysicalDeviceQueueFamilyProperties2)
            }
            "vkGetPhysicalDeviceMemoryProperties2" => {
                Ok(VulkanCommand::GetPhysicalDeviceMemoryProperties2)
            }
            "vkGetPhysicalDeviceSparseImageFormatProperties2" => {
                Ok(VulkanCommand::GetPhysicalDeviceSparseImageFormatProperties2)
            }
            "vkGetPhysicalDeviceExternalBufferProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceExternalBufferProperties)
            }
            "vkGetPhysicalDeviceExternalFenceProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceExternalFenceProperties)
            }
            "vkGetPhysicalDeviceExternalSemaphoreProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceExternalSemaphoreProperties)
            }
            "vkGetPhysicalDeviceToolProperties" => {
                Ok(VulkanCommand::GetPhysicalDeviceToolProperties)
            }
            "vkDestroySurfaceKHR" => Ok(VulkanCommand::DestroySurfaceKhr),
            "vkGetPhysicalDeviceSurfaceSupportKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfaceSupportKhr)
            }
            "vkGetPhysicalDeviceSurfaceCapabilitiesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfaceCapabilitiesKhr)
            }
            "vkGetPhysicalDeviceSurfaceFormatsKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfaceFormatsKhr)
            }
            "vkGetPhysicalDeviceSurfacePresentModesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfacePresentModesKhr)
            }
            "vkGetPhysicalDevicePresentRectanglesKHR" => {
                Ok(VulkanCommand::GetPhysicalDevicePresentRectanglesKhr)
            }
            "vkGetPhysicalDeviceDisplayPropertiesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceDisplayPropertiesKhr)
            }
            "vkGetPhysicalDeviceDisplayPlanePropertiesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceDisplayPlanePropertiesKhr)
            }
            "vkGetDisplayPlaneSupportedDisplaysKHR" => {
                Ok(VulkanCommand::GetDisplayPlaneSupportedDisplaysKhr)
            }
            "vkGetDisplayModePropertiesKHR" => Ok(VulkanCommand::GetDisplayModePropertiesKhr),
            "vkCreateDisplayModeKHR" => Ok(VulkanCommand::CreateDisplayModeKhr),
            "vkGetDisplayPlaneCapabilitiesKHR" => Ok(VulkanCommand::GetDisplayPlaneCapabilitiesKhr),
            "vkCreateDisplayPlaneSurfaceKHR" => Ok(VulkanCommand::CreateDisplayPlaneSurfaceKhr),
            "vkCreateXlibSurfaceKHR" => Ok(VulkanCommand::CreateXlibSurfaceKhr),
            "vkGetPhysicalDeviceXlibPresentationSupportKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceXlibPresentationSupportKhr)
            }
            "vkCreateXcbSurfaceKHR" => Ok(VulkanCommand::CreateXcbSurfaceKhr),
            "vkGetPhysicalDeviceXcbPresentationSupportKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceXcbPresentationSupportKhr)
            }
            "vkCreateWaylandSurfaceKHR" => Ok(VulkanCommand::CreateWaylandSurfaceKhr),
            "vkGetPhysicalDeviceWaylandPresentationSupportKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceWaylandPresentationSupportKhr)
            }
            "vkCreateAndroidSurfaceKHR" => Ok(VulkanCommand::CreateAndroidSurfaceKhr),
            "vkCreateWin32SurfaceKHR" => Ok(VulkanCommand::CreateWin32SurfaceKhr),
            "vkGetPhysicalDeviceWin32PresentationSupportKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceWin32PresentationSupportKhr)
            }
            "vkGetPhysicalDeviceVideoCapabilitiesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceVideoCapabilitiesKhr)
            }
            "vkGetPhysicalDeviceVideoFormatPropertiesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceVideoFormatPropertiesKhr)
            }
            "vkEnumeratePhysicalDeviceQueueFamilyPerformanceQueryCountersKHR" => {
                Ok(VulkanCommand::EnumeratePhysicalDeviceQueueFamilyPerformanceQueryCountersKhr)
            }
            "vkGetPhysicalDeviceQueueFamilyPerformanceQueryPassesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceQueueFamilyPerformanceQueryPassesKhr)
            }
            "vkGetPhysicalDeviceSurfaceCapabilities2KHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfaceCapabilities2Khr)
            }
            "vkGetPhysicalDeviceSurfaceFormats2KHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfaceFormats2Khr)
            }
            "vkGetPhysicalDeviceDisplayProperties2KHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceDisplayProperties2Khr)
            }
            "vkGetPhysicalDeviceDisplayPlaneProperties2KHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceDisplayPlaneProperties2Khr)
            }
            "vkGetDisplayModeProperties2KHR" => Ok(VulkanCommand::GetDisplayModeProperties2Khr),
            "vkGetDisplayPlaneCapabilities2KHR" => {
                Ok(VulkanCommand::GetDisplayPlaneCapabilities2Khr)
            }
            "vkGetPhysicalDeviceFragmentShadingRatesKHR" => {
                Ok(VulkanCommand::GetPhysicalDeviceFragmentShadingRatesKhr)
            }
            "vkCreateDebugReportCallbackEXT" => Ok(VulkanCommand::CreateDebugReportCallbackExt),
            "vkDestroyDebugReportCallbackEXT" => Ok(VulkanCommand::DestroyDebugReportCallbackExt),
            "vkDebugReportMessageEXT" => Ok(VulkanCommand::DebugReportMessageExt),
            "vkCreateStreamDescriptorSurfaceGGP" => {
                Ok(VulkanCommand::CreateStreamDescriptorSurfaceGgp)
            }
            "vkGetPhysicalDeviceExternalImageFormatPropertiesNV" => {
                Ok(VulkanCommand::GetPhysicalDeviceExternalImageFormatPropertiesNv)
            }
            "vkCreateViSurfaceNN" => Ok(VulkanCommand::CreateViSurfaceNn),
            "vkReleaseDisplayEXT" => Ok(VulkanCommand::ReleaseDisplayExt),
            "vkAcquireXlibDisplayEXT" => Ok(VulkanCommand::AcquireXlibDisplayExt),
            "vkGetRandROutputDisplayEXT" => Ok(VulkanCommand::GetRandROutputDisplayExt),
            "vkGetPhysicalDeviceSurfaceCapabilities2EXT" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfaceCapabilities2Ext)
            }
            "vkCreateIOSSurfaceMVK" => Ok(VulkanCommand::CreateIosSurfaceMvk),
            "vkCreateMacOSSurfaceMVK" => Ok(VulkanCommand::CreateMacOsSurfaceMvk),
            "vkCreateDebugUtilsMessengerEXT" => Ok(VulkanCommand::CreateDebugUtilsMessengerExt),
            "vkDestroyDebugUtilsMessengerEXT" => Ok(VulkanCommand::DestroyDebugUtilsMessengerExt),
            "vkSubmitDebugUtilsMessageEXT" => Ok(VulkanCommand::SubmitDebugUtilsMessageExt),
            "vkGetPhysicalDeviceMultisamplePropertiesEXT" => {
                Ok(VulkanCommand::GetPhysicalDeviceMultisamplePropertiesExt)
            }
            "vkGetPhysicalDeviceCalibrateableTimeDomainsEXT" => {
                Ok(VulkanCommand::GetPhysicalDeviceCalibrateableTimeDomainsExt)
            }
            "vkCreateImagePipeSurfaceFUCHSIA" => Ok(VulkanCommand::CreateImagePipeSurfaceFuchsia),
            "vkCreateMetalSurfaceEXT" => Ok(VulkanCommand::CreateMetalSurfaceExt),
            "vkGetPhysicalDeviceCooperativeMatrixPropertiesNV" => {
                Ok(VulkanCommand::GetPhysicalDeviceCooperativeMatrixPropertiesNv)
            }
            "vkGetPhysicalDeviceSupportedFramebufferMixedSamplesCombinationsNV" => {
                Ok(VulkanCommand::GetPhysicalDeviceSupportedFramebufferMixedSamplesCombinationsNv)
            }
            "vkGetPhysicalDeviceSurfacePresentModes2EXT" => {
                Ok(VulkanCommand::GetPhysicalDeviceSurfacePresentModes2Ext)
            }
            "vkCreateHeadlessSurfaceEXT" => Ok(VulkanCommand::CreateHeadlessSurfaceExt),
            "vkAcquireDrmDisplayEXT" => Ok(VulkanCommand::AcquireDrmDisplayExt),
            "vkGetDrmDisplayEXT" => Ok(VulkanCommand::GetDrmDisplayExt),
            "vkAcquireWinrtDisplayNV" => Ok(VulkanCommand::AcquireWinrtDisplayNv),
            "vkGetWinrtDisplayNV" => Ok(VulkanCommand::GetWinrtDisplayNv),
            "vkCreateDirectFBSurfaceEXT" => Ok(VulkanCommand::CreateDirectFbSurfaceExt),
            "vkGetPhysicalDeviceDirectFBPresentationSupportEXT" => {
                Ok(VulkanCommand::GetPhysicalDeviceDirectFbPresentationSupportExt)
            }
            "vkCreateScreenSurfaceQNX" => Ok(VulkanCommand::CreateScreenSurfaceQnx),
            "vkGetPhysicalDeviceScreenPresentationSupportQNX" => {
                Ok(VulkanCommand::GetPhysicalDeviceScreenPresentationSupportQnx)
            }
            "vkGetPhysicalDeviceOpticalFlowImageFormatsNV" => {
                Ok(VulkanCommand::GetPhysicalDeviceOpticalFlowImageFormatsNv)
            }
            "vkGetDeviceProcAddr" => Ok(VulkanCommand::GetDeviceProcAddr),
            "vkGetDeviceQueue" => Ok(VulkanCommand::GetDeviceQueue),
            "vkQueueSubmit" => Ok(VulkanCommand::QueueSubmit),
            "vkQueueWaitIdle" => Ok(VulkanCommand::QueueWaitIdle),
            "vkDeviceWaitIdle" => Ok(VulkanCommand::DeviceWaitIdle),
            "vkAllocateMemory" => Ok(VulkanCommand::AllocateMemory),
            "vkFreeMemory" => Ok(VulkanCommand::FreeMemory),
            "vkMapMemory" => Ok(VulkanCommand::MapMemory),
            "vkUnmapMemory" => Ok(VulkanCommand::UnmapMemory),
            "vkFlushMappedMemoryRanges" => Ok(VulkanCommand::FlushMappedMemoryRanges),
            "vkInvalidateMappedMemoryRanges" => Ok(VulkanCommand::InvalidateMappedMemoryRanges),
            "vkGetDeviceMemoryCommitment" => Ok(VulkanCommand::GetDeviceMemoryCommitment),
            "vkBindBufferMemory" => Ok(VulkanCommand::BindBufferMemory),
            "vkBindImageMemory" => Ok(VulkanCommand::BindImageMemory),
            "vkGetBufferMemoryRequirements" => Ok(VulkanCommand::GetBufferMemoryRequirements),
            "vkGetImageMemoryRequirements" => Ok(VulkanCommand::GetImageMemoryRequirements),
            "vkGetImageSparseMemoryRequirements" => {
                Ok(VulkanCommand::GetImageSparseMemoryRequirements)
            }
            "vkQueueBindSparse" => Ok(VulkanCommand::QueueBindSparse),
            "vkCreateFence" => Ok(VulkanCommand::CreateFence),
            "vkDestroyFence" => Ok(VulkanCommand::DestroyFence),
            "vkResetFences" => Ok(VulkanCommand::ResetFences),
            "vkGetFenceStatus" => Ok(VulkanCommand::GetFenceStatus),
            "vkWaitForFences" => Ok(VulkanCommand::WaitForFences),
            "vkCreateSemaphore" => Ok(VulkanCommand::CreateSemaphore),
            "vkDestroySemaphore" => Ok(VulkanCommand::DestroySemaphore),
            "vkCreateEvent" => Ok(VulkanCommand::CreateEvent),
            "vkDestroyEvent" => Ok(VulkanCommand::DestroyEvent),
            "vkGetEventStatus" => Ok(VulkanCommand::GetEventStatus),
            "vkSetEvent" => Ok(VulkanCommand::SetEvent),
            "vkResetEvent" => Ok(VulkanCommand::ResetEvent),
            "vkCreateQueryPool" => Ok(VulkanCommand::CreateQueryPool),
            "vkDestroyQueryPool" => Ok(VulkanCommand::DestroyQueryPool),
            "vkGetQueryPoolResults" => Ok(VulkanCommand::GetQueryPoolResults),
            "vkCreateBuffer" => Ok(VulkanCommand::CreateBuffer),
            "vkDestroyBuffer" => Ok(VulkanCommand::DestroyBuffer),
            "vkCreateBufferView" => Ok(VulkanCommand::CreateBufferView),
            "vkDestroyBufferView" => Ok(VulkanCommand::DestroyBufferView),
            "vkCreateImage" => Ok(VulkanCommand::CreateImage),
            "vkDestroyImage" => Ok(VulkanCommand::DestroyImage),
            "vkGetImageSubresourceLayout" => Ok(VulkanCommand::GetImageSubresourceLayout),
            "vkCreateImageView" => Ok(VulkanCommand::CreateImageView),
            "vkDestroyImageView" => Ok(VulkanCommand::DestroyImageView),
            "vkCreateShaderModule" => Ok(VulkanCommand::CreateShaderModule),
            "vkDestroyShaderModule" => Ok(VulkanCommand::DestroyShaderModule),
            "vkCreatePipelineCache" => Ok(VulkanCommand::CreatePipelineCache),
            "vkDestroyPipelineCache" => Ok(VulkanCommand::DestroyPipelineCache),
            "vkGetPipelineCacheData" => Ok(VulkanCommand::GetPipelineCacheData),
            "vkMergePipelineCaches" => Ok(VulkanCommand::MergePipelineCaches),
            "vkCreateGraphicsPipelines" => Ok(VulkanCommand::CreateGraphicsPipelines),
            "vkCreateComputePipelines" => Ok(VulkanCommand::CreateComputePipelines),
            "vkDestroyPipeline" => Ok(VulkanCommand::DestroyPipeline),
            "vkCreatePipelineLayout" => Ok(VulkanCommand::CreatePipelineLayout),
            "vkDestroyPipelineLayout" => Ok(VulkanCommand::DestroyPipelineLayout),
            "vkCreateSampler" => Ok(VulkanCommand::CreateSampler),
            "vkDestroySampler" => Ok(VulkanCommand::DestroySampler),
            "vkCreateDescriptorSetLayout" => Ok(VulkanCommand::CreateDescriptorSetLayout),
            "vkDestroyDescriptorSetLayout" => Ok(VulkanCommand::DestroyDescriptorSetLayout),
            "vkCreateDescriptorPool" => Ok(VulkanCommand::CreateDescriptorPool),
            "vkDestroyDescriptorPool" => Ok(VulkanCommand::DestroyDescriptorPool),
            "vkResetDescriptorPool" => Ok(VulkanCommand::ResetDescriptorPool),
            "vkAllocateDescriptorSets" => Ok(VulkanCommand::AllocateDescriptorSets),
            "vkFreeDescriptorSets" => Ok(VulkanCommand::FreeDescriptorSets),
            "vkUpdateDescriptorSets" => Ok(VulkanCommand::UpdateDescriptorSets),
            "vkCreateFramebuffer" => Ok(VulkanCommand::CreateFramebuffer),
            "vkDestroyFramebuffer" => Ok(VulkanCommand::DestroyFramebuffer),
            "vkCreateRenderPass" => Ok(VulkanCommand::CreateRenderPass),
            "vkDestroyRenderPass" => Ok(VulkanCommand::DestroyRenderPass),
            "vkGetRenderAreaGranularity" => Ok(VulkanCommand::GetRenderAreaGranularity),
            "vkCreateCommandPool" => Ok(VulkanCommand::CreateCommandPool),
            "vkDestroyCommandPool" => Ok(VulkanCommand::DestroyCommandPool),
            "vkResetCommandPool" => Ok(VulkanCommand::ResetCommandPool),
            "vkAllocateCommandBuffers" => Ok(VulkanCommand::AllocateCommandBuffers),
            "vkFreeCommandBuffers" => Ok(VulkanCommand::FreeCommandBuffers),
            "vkBeginCommandBuffer" => Ok(VulkanCommand::BeginCommandBuffer),
            "vkEndCommandBuffer" => Ok(VulkanCommand::EndCommandBuffer),
            "vkResetCommandBuffer" => Ok(VulkanCommand::ResetCommandBuffer),
            "vkCmdBindPipeline" => Ok(VulkanCommand::CmdBindPipeline),
            "vkCmdSetViewport" => Ok(VulkanCommand::CmdSetViewport),
            "vkCmdSetScissor" => Ok(VulkanCommand::CmdSetScissor),
            "vkCmdSetLineWidth" => Ok(VulkanCommand::CmdSetLineWidth),
            "vkCmdSetDepthBias" => Ok(VulkanCommand::CmdSetDepthBias),
            "vkCmdSetBlendConstants" => Ok(VulkanCommand::CmdSetBlendConstants),
            "vkCmdSetDepthBounds" => Ok(VulkanCommand::CmdSetDepthBounds),
            "vkCmdSetStencilCompareMask" => Ok(VulkanCommand::CmdSetStencilCompareMask),
            "vkCmdSetStencilWriteMask" => Ok(VulkanCommand::CmdSetStencilWriteMask),
            "vkCmdSetStencilReference" => Ok(VulkanCommand::CmdSetStencilReference),
            "vkCmdBindDescriptorSets" => Ok(VulkanCommand::CmdBindDescriptorSets),
            "vkCmdBindIndexBuffer" => Ok(VulkanCommand::CmdBindIndexBuffer),
            "vkCmdBindVertexBuffers" => Ok(VulkanCommand::CmdBindVertexBuffers),
            "vkCmdDraw" => Ok(VulkanCommand::CmdDraw),
            "vkCmdDrawIndexed" => Ok(VulkanCommand::CmdDrawIndexed),
            "vkCmdDrawIndirect" => Ok(VulkanCommand::CmdDrawIndirect),
            "vkCmdDrawIndexedIndirect" => Ok(VulkanCommand::CmdDrawIndexedIndirect),
            "vkCmdDispatch" => Ok(VulkanCommand::CmdDispatch),
            "vkCmdDispatchIndirect" => Ok(VulkanCommand::CmdDispatchIndirect),
            "vkCmdCopyBuffer" => Ok(VulkanCommand::CmdCopyBuffer),
            "vkCmdCopyImage" => Ok(VulkanCommand::CmdCopyImage),
            "vkCmdBlitImage" => Ok(VulkanCommand::CmdBlitImage),
            "vkCmdCopyBufferToImage" => Ok(VulkanCommand::CmdCopyBufferToImage),
            "vkCmdCopyImageToBuffer" => Ok(VulkanCommand::CmdCopyImageToBuffer),
            "vkCmdUpdateBuffer" => Ok(VulkanCommand::CmdUpdateBuffer),
            "vkCmdFillBuffer" => Ok(VulkanCommand::CmdFillBuffer),
            "vkCmdClearColorImage" => Ok(VulkanCommand::CmdClearColorImage),
            "vkCmdClearDepthStencilImage" => Ok(VulkanCommand::CmdClearDepthStencilImage),
            "vkCmdClearAttachments" => Ok(VulkanCommand::CmdClearAttachments),
            "vkCmdResolveImage" => Ok(VulkanCommand::CmdResolveImage),
            "vkCmdSetEvent" => Ok(VulkanCommand::CmdSetEvent),
            "vkCmdResetEvent" => Ok(VulkanCommand::CmdResetEvent),
            "vkCmdWaitEvents" => Ok(VulkanCommand::CmdWaitEvents),
            "vkCmdPipelineBarrier" => Ok(VulkanCommand::CmdPipelineBarrier),
            "vkCmdBeginQuery" => Ok(VulkanCommand::CmdBeginQuery),
            "vkCmdEndQuery" => Ok(VulkanCommand::CmdEndQuery),
            "vkCmdResetQueryPool" => Ok(VulkanCommand::CmdResetQueryPool),
            "vkCmdWriteTimestamp" => Ok(VulkanCommand::CmdWriteTimestamp),
            "vkCmdCopyQueryPoolResults" => Ok(VulkanCommand::CmdCopyQueryPoolResults),
            "vkCmdPushConstants" => Ok(VulkanCommand::CmdPushConstants),
            "vkCmdBeginRenderPass" => Ok(VulkanCommand::CmdBeginRenderPass),
            "vkCmdNextSubpass" => Ok(VulkanCommand::CmdNextSubpass),
            "vkCmdEndRenderPass" => Ok(VulkanCommand::CmdEndRenderPass),
            "vkCmdExecuteCommands" => Ok(VulkanCommand::CmdExecuteCommands),
            "vkBindBufferMemory2" => Ok(VulkanCommand::BindBufferMemory2),
            "vkBindImageMemory2" => Ok(VulkanCommand::BindImageMemory2),
            "vkGetDeviceGroupPeerMemoryFeatures" => {
                Ok(VulkanCommand::GetDeviceGroupPeerMemoryFeatures)
            }
            "vkCmdSetDeviceMask" => Ok(VulkanCommand::CmdSetDeviceMask),
            "vkCmdDispatchBase" => Ok(VulkanCommand::CmdDispatchBase),
            "vkGetImageMemoryRequirements2" => Ok(VulkanCommand::GetImageMemoryRequirements2),
            "vkGetBufferMemoryRequirements2" => Ok(VulkanCommand::GetBufferMemoryRequirements2),
            "vkGetImageSparseMemoryRequirements2" => {
                Ok(VulkanCommand::GetImageSparseMemoryRequirements2)
            }
            "vkTrimCommandPool" => Ok(VulkanCommand::TrimCommandPool),
            "vkGetDeviceQueue2" => Ok(VulkanCommand::GetDeviceQueue2),
            "vkCreateSamplerYcbcrConversion" => Ok(VulkanCommand::CreateSamplerYcbcrConversion),
            "vkDestroySamplerYcbcrConversion" => Ok(VulkanCommand::DestroySamplerYcbcrConversion),
            "vkCreateDescriptorUpdateTemplate" => Ok(VulkanCommand::CreateDescriptorUpdateTemplate),
            "vkDestroyDescriptorUpdateTemplate" => {
                Ok(VulkanCommand::DestroyDescriptorUpdateTemplate)
            }
            "vkUpdateDescriptorSetWithTemplate" => {
                Ok(VulkanCommand::UpdateDescriptorSetWithTemplate)
            }
            "vkGetDescriptorSetLayoutSupport" => Ok(VulkanCommand::GetDescriptorSetLayoutSupport),
            "vkCmdDrawIndirectCount" => Ok(VulkanCommand::CmdDrawIndirectCount),
            "vkCmdDrawIndexedIndirectCount" => Ok(VulkanCommand::CmdDrawIndexedIndirectCount),
            "vkCreateRenderPass2" => Ok(VulkanCommand::CreateRenderPass2),
            "vkCmdBeginRenderPass2" => Ok(VulkanCommand::CmdBeginRenderPass2),
            "vkCmdNextSubpass2" => Ok(VulkanCommand::CmdNextSubpass2),
            "vkCmdEndRenderPass2" => Ok(VulkanCommand::CmdEndRenderPass2),
            "vkResetQueryPool" => Ok(VulkanCommand::ResetQueryPool),
            "vkGetSemaphoreCounterValue" => Ok(VulkanCommand::GetSemaphoreCounterValue),
            "vkWaitSemaphores" => Ok(VulkanCommand::WaitSemaphores),
            "vkSignalSemaphore" => Ok(VulkanCommand::SignalSemaphore),
            "vkGetBufferDeviceAddress" => Ok(VulkanCommand::GetBufferDeviceAddress),
            "vkGetBufferOpaqueCaptureAddress" => Ok(VulkanCommand::GetBufferOpaqueCaptureAddress),
            "vkGetDeviceMemoryOpaqueCaptureAddress" => {
                Ok(VulkanCommand::GetDeviceMemoryOpaqueCaptureAddress)
            }
            "vkCreatePrivateDataSlot" => Ok(VulkanCommand::CreatePrivateDataSlot),
            "vkDestroyPrivateDataSlot" => Ok(VulkanCommand::DestroyPrivateDataSlot),
            "vkSetPrivateData" => Ok(VulkanCommand::SetPrivateData),
            "vkGetPrivateData" => Ok(VulkanCommand::GetPrivateData),
            "vkCmdSetEvent2" => Ok(VulkanCommand::CmdSetEvent2),
            "vkCmdResetEvent2" => Ok(VulkanCommand::CmdResetEvent2),
            "vkCmdWaitEvents2" => Ok(VulkanCommand::CmdWaitEvents2),
            "vkCmdPipelineBarrier2" => Ok(VulkanCommand::CmdPipelineBarrier2),
            "vkCmdWriteTimestamp2" => Ok(VulkanCommand::CmdWriteTimestamp2),
            "vkQueueSubmit2" => Ok(VulkanCommand::QueueSubmit2),
            "vkCmdCopyBuffer2" => Ok(VulkanCommand::CmdCopyBuffer2),
            "vkCmdCopyImage2" => Ok(VulkanCommand::CmdCopyImage2),
            "vkCmdCopyBufferToImage2" => Ok(VulkanCommand::CmdCopyBufferToImage2),
            "vkCmdCopyImageToBuffer2" => Ok(VulkanCommand::CmdCopyImageToBuffer2),
            "vkCmdBlitImage2" => Ok(VulkanCommand::CmdBlitImage2),
            "vkCmdResolveImage2" => Ok(VulkanCommand::CmdResolveImage2),
            "vkCmdBeginRendering" => Ok(VulkanCommand::CmdBeginRendering),
            "vkCmdEndRendering" => Ok(VulkanCommand::CmdEndRendering),
            "vkCmdSetCullMode" => Ok(VulkanCommand::CmdSetCullMode),
            "vkCmdSetFrontFace" => Ok(VulkanCommand::CmdSetFrontFace),
            "vkCmdSetPrimitiveTopology" => Ok(VulkanCommand::CmdSetPrimitiveTopology),
            "vkCmdSetViewportWithCount" => Ok(VulkanCommand::CmdSetViewportWithCount),
            "vkCmdSetScissorWithCount" => Ok(VulkanCommand::CmdSetScissorWithCount),
            "vkCmdBindVertexBuffers2" => Ok(VulkanCommand::CmdBindVertexBuffers2),
            "vkCmdSetDepthTestEnable" => Ok(VulkanCommand::CmdSetDepthTestEnable),
            "vkCmdSetDepthWriteEnable" => Ok(VulkanCommand::CmdSetDepthWriteEnable),
            "vkCmdSetDepthCompareOp" => Ok(VulkanCommand::CmdSetDepthCompareOp),
            "vkCmdSetDepthBoundsTestEnable" => Ok(VulkanCommand::CmdSetDepthBoundsTestEnable),
            "vkCmdSetStencilTestEnable" => Ok(VulkanCommand::CmdSetStencilTestEnable),
            "vkCmdSetStencilOp" => Ok(VulkanCommand::CmdSetStencilOp),
            "vkCmdSetRasterizerDiscardEnable" => Ok(VulkanCommand::CmdSetRasterizerDiscardEnable),
            "vkCmdSetDepthBiasEnable" => Ok(VulkanCommand::CmdSetDepthBiasEnable),
            "vkCmdSetPrimitiveRestartEnable" => Ok(VulkanCommand::CmdSetPrimitiveRestartEnable),
            "vkGetDeviceBufferMemoryRequirements" => {
                Ok(VulkanCommand::GetDeviceBufferMemoryRequirements)
            }
            "vkGetDeviceImageMemoryRequirements" => {
                Ok(VulkanCommand::GetDeviceImageMemoryRequirements)
            }
            "vkGetDeviceImageSparseMemoryRequirements" => {
                Ok(VulkanCommand::GetDeviceImageSparseMemoryRequirements)
            }
            "vkCreateSwapchainKHR" => Ok(VulkanCommand::CreateSwapchainKhr),
            "vkDestroySwapchainKHR" => Ok(VulkanCommand::DestroySwapchainKhr),
            "vkGetSwapchainImagesKHR" => Ok(VulkanCommand::GetSwapchainImagesKhr),
            "vkAcquireNextImageKHR" => Ok(VulkanCommand::AcquireNextImageKhr),
            "vkQueuePresentKHR" => Ok(VulkanCommand::QueuePresentKhr),
            "vkGetDeviceGroupPresentCapabilitiesKHR" => {
                Ok(VulkanCommand::GetDeviceGroupPresentCapabilitiesKhr)
            }
            "vkGetDeviceGroupSurfacePresentModesKHR" => {
                Ok(VulkanCommand::GetDeviceGroupSurfacePresentModesKhr)
            }
            "vkAcquireNextImage2KHR" => Ok(VulkanCommand::AcquireNextImage2Khr),
            "vkCreateSharedSwapchainsKHR" => Ok(VulkanCommand::CreateSharedSwapchainsKhr),
            "vkCreateVideoSessionKHR" => Ok(VulkanCommand::CreateVideoSessionKhr),
            "vkDestroyVideoSessionKHR" => Ok(VulkanCommand::DestroyVideoSessionKhr),
            "vkGetVideoSessionMemoryRequirementsKHR" => {
                Ok(VulkanCommand::GetVideoSessionMemoryRequirementsKhr)
            }
            "vkBindVideoSessionMemoryKHR" => Ok(VulkanCommand::BindVideoSessionMemoryKhr),
            "vkCreateVideoSessionParametersKHR" => {
                Ok(VulkanCommand::CreateVideoSessionParametersKhr)
            }
            "vkUpdateVideoSessionParametersKHR" => {
                Ok(VulkanCommand::UpdateVideoSessionParametersKhr)
            }
            "vkDestroyVideoSessionParametersKHR" => {
                Ok(VulkanCommand::DestroyVideoSessionParametersKhr)
            }
            "vkCmdBeginVideoCodingKHR" => Ok(VulkanCommand::CmdBeginVideoCodingKhr),
            "vkCmdEndVideoCodingKHR" => Ok(VulkanCommand::CmdEndVideoCodingKhr),
            "vkCmdControlVideoCodingKHR" => Ok(VulkanCommand::CmdControlVideoCodingKhr),
            "vkCmdDecodeVideoKHR" => Ok(VulkanCommand::CmdDecodeVideoKhr),
            "vkGetMemoryWin32HandleKHR" => Ok(VulkanCommand::GetMemoryWin32HandleKhr),
            "vkGetMemoryWin32HandlePropertiesKHR" => {
                Ok(VulkanCommand::GetMemoryWin32HandlePropertiesKhr)
            }
            "vkGetMemoryFdKHR" => Ok(VulkanCommand::GetMemoryFdKhr),
            "vkGetMemoryFdPropertiesKHR" => Ok(VulkanCommand::GetMemoryFdPropertiesKhr),
            "vkImportSemaphoreWin32HandleKHR" => Ok(VulkanCommand::ImportSemaphoreWin32HandleKhr),
            "vkGetSemaphoreWin32HandleKHR" => Ok(VulkanCommand::GetSemaphoreWin32HandleKhr),
            "vkImportSemaphoreFdKHR" => Ok(VulkanCommand::ImportSemaphoreFdKhr),
            "vkGetSemaphoreFdKHR" => Ok(VulkanCommand::GetSemaphoreFdKhr),
            "vkCmdPushDescriptorSetKHR" => Ok(VulkanCommand::CmdPushDescriptorSetKhr),
            "vkCmdPushDescriptorSetWithTemplateKHR" => {
                Ok(VulkanCommand::CmdPushDescriptorSetWithTemplateKhr)
            }
            "vkGetSwapchainStatusKHR" => Ok(VulkanCommand::GetSwapchainStatusKhr),
            "vkImportFenceWin32HandleKHR" => Ok(VulkanCommand::ImportFenceWin32HandleKhr),
            "vkGetFenceWin32HandleKHR" => Ok(VulkanCommand::GetFenceWin32HandleKhr),
            "vkImportFenceFdKHR" => Ok(VulkanCommand::ImportFenceFdKhr),
            "vkGetFenceFdKHR" => Ok(VulkanCommand::GetFenceFdKhr),
            "vkAcquireProfilingLockKHR" => Ok(VulkanCommand::AcquireProfilingLockKhr),
            "vkReleaseProfilingLockKHR" => Ok(VulkanCommand::ReleaseProfilingLockKhr),
            "vkCmdSetFragmentShadingRateKHR" => Ok(VulkanCommand::CmdSetFragmentShadingRateKhr),
            "vkWaitForPresentKHR" => Ok(VulkanCommand::WaitForPresentKhr),
            "vkCreateDeferredOperationKHR" => Ok(VulkanCommand::CreateDeferredOperationKhr),
            "vkDestroyDeferredOperationKHR" => Ok(VulkanCommand::DestroyDeferredOperationKhr),
            "vkGetDeferredOperationMaxConcurrencyKHR" => {
                Ok(VulkanCommand::GetDeferredOperationMaxConcurrencyKhr)
            }
            "vkGetDeferredOperationResultKHR" => Ok(VulkanCommand::GetDeferredOperationResultKhr),
            "vkDeferredOperationJoinKHR" => Ok(VulkanCommand::DeferredOperationJoinKhr),
            "vkGetPipelineExecutablePropertiesKHR" => {
                Ok(VulkanCommand::GetPipelineExecutablePropertiesKhr)
            }
            "vkGetPipelineExecutableStatisticsKHR" => {
                Ok(VulkanCommand::GetPipelineExecutableStatisticsKhr)
            }
            "vkGetPipelineExecutableInternalRepresentationsKHR" => {
                Ok(VulkanCommand::GetPipelineExecutableInternalRepresentationsKhr)
            }
            "vkCmdEncodeVideoKHR" => Ok(VulkanCommand::CmdEncodeVideoKhr),
            "vkCmdWriteBufferMarker2AMD" => Ok(VulkanCommand::CmdWriteBufferMarker2Amd),
            "vkGetQueueCheckpointData2NV" => Ok(VulkanCommand::GetQueueCheckpointData2Nv),
            "vkCmdTraceRaysIndirect2KHR" => Ok(VulkanCommand::CmdTraceRaysIndirect2Khr),
            "vkGetSwapchainGrallocUsageANDROID" => {
                Ok(VulkanCommand::GetSwapchainGrallocUsageAndroid)
            }
            "vkAcquireImageANDROID" => Ok(VulkanCommand::AcquireImageAndroid),
            "vkQueueSignalReleaseImageANDROID" => Ok(VulkanCommand::QueueSignalReleaseImageAndroid),
            "vkGetSwapchainGrallocUsage2ANDROID" => {
                Ok(VulkanCommand::GetSwapchainGrallocUsage2Android)
            }
            "vkDebugMarkerSetObjectTagEXT" => Ok(VulkanCommand::DebugMarkerSetObjectTagExt),
            "vkDebugMarkerSetObjectNameEXT" => Ok(VulkanCommand::DebugMarkerSetObjectNameExt),
            "vkCmdDebugMarkerBeginEXT" => Ok(VulkanCommand::CmdDebugMarkerBeginExt),
            "vkCmdDebugMarkerEndEXT" => Ok(VulkanCommand::CmdDebugMarkerEndExt),
            "vkCmdDebugMarkerInsertEXT" => Ok(VulkanCommand::CmdDebugMarkerInsertExt),
            "vkCmdBindTransformFeedbackBuffersEXT" => {
                Ok(VulkanCommand::CmdBindTransformFeedbackBuffersExt)
            }
            "vkCmdBeginTransformFeedbackEXT" => Ok(VulkanCommand::CmdBeginTransformFeedbackExt),
            "vkCmdEndTransformFeedbackEXT" => Ok(VulkanCommand::CmdEndTransformFeedbackExt),
            "vkCmdBeginQueryIndexedEXT" => Ok(VulkanCommand::CmdBeginQueryIndexedExt),
            "vkCmdEndQueryIndexedEXT" => Ok(VulkanCommand::CmdEndQueryIndexedExt),
            "vkCmdDrawIndirectByteCountEXT" => Ok(VulkanCommand::CmdDrawIndirectByteCountExt),
            "vkCreateCuModuleNVX" => Ok(VulkanCommand::CreateCuModuleNvx),
            "vkCreateCuFunctionNVX" => Ok(VulkanCommand::CreateCuFunctionNvx),
            "vkDestroyCuModuleNVX" => Ok(VulkanCommand::DestroyCuModuleNvx),
            "vkDestroyCuFunctionNVX" => Ok(VulkanCommand::DestroyCuFunctionNvx),
            "vkCmdCuLaunchKernelNVX" => Ok(VulkanCommand::CmdCuLaunchKernelNvx),
            "vkGetImageViewHandleNVX" => Ok(VulkanCommand::GetImageViewHandleNvx),
            "vkGetImageViewAddressNVX" => Ok(VulkanCommand::GetImageViewAddressNvx),
            "vkGetShaderInfoAMD" => Ok(VulkanCommand::GetShaderInfoAmd),
            "vkGetMemoryWin32HandleNV" => Ok(VulkanCommand::GetMemoryWin32HandleNv),
            "vkCmdBeginConditionalRenderingEXT" => {
                Ok(VulkanCommand::CmdBeginConditionalRenderingExt)
            }
            "vkCmdEndConditionalRenderingEXT" => Ok(VulkanCommand::CmdEndConditionalRenderingExt),
            "vkCmdSetViewportWScalingNV" => Ok(VulkanCommand::CmdSetViewportWScalingNv),
            "vkDisplayPowerControlEXT" => Ok(VulkanCommand::DisplayPowerControlExt),
            "vkRegisterDeviceEventEXT" => Ok(VulkanCommand::RegisterDeviceEventExt),
            "vkRegisterDisplayEventEXT" => Ok(VulkanCommand::RegisterDisplayEventExt),
            "vkGetSwapchainCounterEXT" => Ok(VulkanCommand::GetSwapchainCounterExt),
            "vkGetRefreshCycleDurationGOOGLE" => Ok(VulkanCommand::GetRefreshCycleDurationGoogle),
            "vkGetPastPresentationTimingGOOGLE" => {
                Ok(VulkanCommand::GetPastPresentationTimingGoogle)
            }
            "vkCmdSetDiscardRectangleEXT" => Ok(VulkanCommand::CmdSetDiscardRectangleExt),
            "vkSetHdrMetadataEXT" => Ok(VulkanCommand::SetHdrMetadataExt),
            "vkSetDebugUtilsObjectNameEXT" => Ok(VulkanCommand::SetDebugUtilsObjectNameExt),
            "vkSetDebugUtilsObjectTagEXT" => Ok(VulkanCommand::SetDebugUtilsObjectTagExt),
            "vkQueueBeginDebugUtilsLabelEXT" => Ok(VulkanCommand::QueueBeginDebugUtilsLabelExt),
            "vkQueueEndDebugUtilsLabelEXT" => Ok(VulkanCommand::QueueEndDebugUtilsLabelExt),
            "vkQueueInsertDebugUtilsLabelEXT" => Ok(VulkanCommand::QueueInsertDebugUtilsLabelExt),
            "vkCmdBeginDebugUtilsLabelEXT" => Ok(VulkanCommand::CmdBeginDebugUtilsLabelExt),
            "vkCmdEndDebugUtilsLabelEXT" => Ok(VulkanCommand::CmdEndDebugUtilsLabelExt),
            "vkCmdInsertDebugUtilsLabelEXT" => Ok(VulkanCommand::CmdInsertDebugUtilsLabelExt),
            "vkGetAndroidHardwareBufferPropertiesANDROID" => {
                Ok(VulkanCommand::GetAndroidHardwareBufferPropertiesAndroid)
            }
            "vkGetMemoryAndroidHardwareBufferANDROID" => {
                Ok(VulkanCommand::GetMemoryAndroidHardwareBufferAndroid)
            }
            "vkCmdSetSampleLocationsEXT" => Ok(VulkanCommand::CmdSetSampleLocationsExt),
            "vkGetImageDrmFormatModifierPropertiesEXT" => {
                Ok(VulkanCommand::GetImageDrmFormatModifierPropertiesExt)
            }
            "vkCreateValidationCacheEXT" => Ok(VulkanCommand::CreateValidationCacheExt),
            "vkDestroyValidationCacheEXT" => Ok(VulkanCommand::DestroyValidationCacheExt),
            "vkMergeValidationCachesEXT" => Ok(VulkanCommand::MergeValidationCachesExt),
            "vkGetValidationCacheDataEXT" => Ok(VulkanCommand::GetValidationCacheDataExt),
            "vkCmdBindShadingRateImageNV" => Ok(VulkanCommand::CmdBindShadingRateImageNv),
            "vkCmdSetViewportShadingRatePaletteNV" => {
                Ok(VulkanCommand::CmdSetViewportShadingRatePaletteNv)
            }
            "vkCmdSetCoarseSampleOrderNV" => Ok(VulkanCommand::CmdSetCoarseSampleOrderNv),
            "vkCreateAccelerationStructureNV" => Ok(VulkanCommand::CreateAccelerationStructureNv),
            "vkDestroyAccelerationStructureNV" => Ok(VulkanCommand::DestroyAccelerationStructureNv),
            "vkGetAccelerationStructureMemoryRequirementsNV" => {
                Ok(VulkanCommand::GetAccelerationStructureMemoryRequirementsNv)
            }
            "vkBindAccelerationStructureMemoryNV" => {
                Ok(VulkanCommand::BindAccelerationStructureMemoryNv)
            }
            "vkCmdBuildAccelerationStructureNV" => {
                Ok(VulkanCommand::CmdBuildAccelerationStructureNv)
            }
            "vkCmdCopyAccelerationStructureNV" => Ok(VulkanCommand::CmdCopyAccelerationStructureNv),
            "vkCmdTraceRaysNV" => Ok(VulkanCommand::CmdTraceRaysNv),
            "vkCreateRayTracingPipelinesNV" => Ok(VulkanCommand::CreateRayTracingPipelinesNv),
            "vkGetRayTracingShaderGroupHandlesKHR" => {
                Ok(VulkanCommand::GetRayTracingShaderGroupHandlesKhr)
            }
            "vkGetAccelerationStructureHandleNV" => {
                Ok(VulkanCommand::GetAccelerationStructureHandleNv)
            }
            "vkCmdWriteAccelerationStructuresPropertiesNV" => {
                Ok(VulkanCommand::CmdWriteAccelerationStructuresPropertiesNv)
            }
            "vkCompileDeferredNV" => Ok(VulkanCommand::CompileDeferredNv),
            "vkGetMemoryHostPointerPropertiesEXT" => {
                Ok(VulkanCommand::GetMemoryHostPointerPropertiesExt)
            }
            "vkCmdWriteBufferMarkerAMD" => Ok(VulkanCommand::CmdWriteBufferMarkerAmd),
            "vkGetCalibratedTimestampsEXT" => Ok(VulkanCommand::GetCalibratedTimestampsExt),
            "vkCmdDrawMeshTasksNV" => Ok(VulkanCommand::CmdDrawMeshTasksNv),
            "vkCmdDrawMeshTasksIndirectNV" => Ok(VulkanCommand::CmdDrawMeshTasksIndirectNv),
            "vkCmdDrawMeshTasksIndirectCountNV" => {
                Ok(VulkanCommand::CmdDrawMeshTasksIndirectCountNv)
            }
            "vkCmdSetExclusiveScissorNV" => Ok(VulkanCommand::CmdSetExclusiveScissorNv),
            "vkCmdSetCheckpointNV" => Ok(VulkanCommand::CmdSetCheckpointNv),
            "vkGetQueueCheckpointDataNV" => Ok(VulkanCommand::GetQueueCheckpointDataNv),
            "vkInitializePerformanceApiINTEL" => Ok(VulkanCommand::InitializePerformanceApiIntel),
            "vkUninitializePerformanceApiINTEL" => {
                Ok(VulkanCommand::UninitializePerformanceApiIntel)
            }
            "vkCmdSetPerformanceMarkerINTEL" => Ok(VulkanCommand::CmdSetPerformanceMarkerIntel),
            "vkCmdSetPerformanceStreamMarkerINTEL" => {
                Ok(VulkanCommand::CmdSetPerformanceStreamMarkerIntel)
            }
            "vkCmdSetPerformanceOverrideINTEL" => Ok(VulkanCommand::CmdSetPerformanceOverrideIntel),
            "vkAcquirePerformanceConfigurationINTEL" => {
                Ok(VulkanCommand::AcquirePerformanceConfigurationIntel)
            }
            "vkReleasePerformanceConfigurationINTEL" => {
                Ok(VulkanCommand::ReleasePerformanceConfigurationIntel)
            }
            "vkQueueSetPerformanceConfigurationINTEL" => {
                Ok(VulkanCommand::QueueSetPerformanceConfigurationIntel)
            }
            "vkGetPerformanceParameterINTEL" => Ok(VulkanCommand::GetPerformanceParameterIntel),
            "vkSetLocalDimmingAMD" => Ok(VulkanCommand::SetLocalDimmingAmd),
            "vkAcquireFullScreenExclusiveModeEXT" => {
                Ok(VulkanCommand::AcquireFullScreenExclusiveModeExt)
            }
            "vkReleaseFullScreenExclusiveModeEXT" => {
                Ok(VulkanCommand::ReleaseFullScreenExclusiveModeExt)
            }
            "vkGetDeviceGroupSurfacePresentModes2EXT" => {
                Ok(VulkanCommand::GetDeviceGroupSurfacePresentModes2Ext)
            }
            "vkCmdSetLineStippleEXT" => Ok(VulkanCommand::CmdSetLineStippleExt),
            "vkReleaseSwapchainImagesEXT" => Ok(VulkanCommand::ReleaseSwapchainImagesExt),
            "vkGetGeneratedCommandsMemoryRequirementsNV" => {
                Ok(VulkanCommand::GetGeneratedCommandsMemoryRequirementsNv)
            }
            "vkCmdPreprocessGeneratedCommandsNV" => {
                Ok(VulkanCommand::CmdPreprocessGeneratedCommandsNv)
            }
            "vkCmdExecuteGeneratedCommandsNV" => Ok(VulkanCommand::CmdExecuteGeneratedCommandsNv),
            "vkCmdBindPipelineShaderGroupNV" => Ok(VulkanCommand::CmdBindPipelineShaderGroupNv),
            "vkCreateIndirectCommandsLayoutNV" => Ok(VulkanCommand::CreateIndirectCommandsLayoutNv),
            "vkDestroyIndirectCommandsLayoutNV" => {
                Ok(VulkanCommand::DestroyIndirectCommandsLayoutNv)
            }
            "vkExportMetalObjectsEXT" => Ok(VulkanCommand::ExportMetalObjectsExt),
            "vkGetDescriptorSetLayoutSizeEXT" => Ok(VulkanCommand::GetDescriptorSetLayoutSizeExt),
            "vkGetDescriptorSetLayoutBindingOffsetEXT" => {
                Ok(VulkanCommand::GetDescriptorSetLayoutBindingOffsetExt)
            }
            "vkGetDescriptorEXT" => Ok(VulkanCommand::GetDescriptorExt),
            "vkCmdBindDescriptorBuffersEXT" => Ok(VulkanCommand::CmdBindDescriptorBuffersExt),
            "vkCmdSetDescriptorBufferOffsetsEXT" => {
                Ok(VulkanCommand::CmdSetDescriptorBufferOffsetsExt)
            }
            "vkCmdBindDescriptorBufferEmbeddedSamplersEXT" => {
                Ok(VulkanCommand::CmdBindDescriptorBufferEmbeddedSamplersExt)
            }
            "vkGetBufferOpaqueCaptureDescriptorDataEXT" => {
                Ok(VulkanCommand::GetBufferOpaqueCaptureDescriptorDataExt)
            }
            "vkGetImageOpaqueCaptureDescriptorDataEXT" => {
                Ok(VulkanCommand::GetImageOpaqueCaptureDescriptorDataExt)
            }
            "vkGetImageViewOpaqueCaptureDescriptorDataEXT" => {
                Ok(VulkanCommand::GetImageViewOpaqueCaptureDescriptorDataExt)
            }
            "vkGetSamplerOpaqueCaptureDescriptorDataEXT" => {
                Ok(VulkanCommand::GetSamplerOpaqueCaptureDescriptorDataExt)
            }
            "vkGetAccelerationStructureOpaqueCaptureDescriptorDataEXT" => {
                Ok(VulkanCommand::GetAccelerationStructureOpaqueCaptureDescriptorDataExt)
            }
            "vkCmdSetFragmentShadingRateEnumNV" => {
                Ok(VulkanCommand::CmdSetFragmentShadingRateEnumNv)
            }
            "vkGetImageSubresourceLayout2EXT" => Ok(VulkanCommand::GetImageSubresourceLayout2Ext),
            "vkCmdSetVertexInputEXT" => Ok(VulkanCommand::CmdSetVertexInputExt),
            "vkGetMemoryZirconHandleFUCHSIA" => Ok(VulkanCommand::GetMemoryZirconHandleFuchsia),
            "vkGetMemoryZirconHandlePropertiesFUCHSIA" => {
                Ok(VulkanCommand::GetMemoryZirconHandlePropertiesFuchsia)
            }
            "vkImportSemaphoreZirconHandleFUCHSIA" => {
                Ok(VulkanCommand::ImportSemaphoreZirconHandleFuchsia)
            }
            "vkGetSemaphoreZirconHandleFUCHSIA" => {
                Ok(VulkanCommand::GetSemaphoreZirconHandleFuchsia)
            }
            "vkCreateBufferCollectionFUCHSIA" => Ok(VulkanCommand::CreateBufferCollectionFuchsia),
            "vkSetBufferCollectionImageConstraintsFUCHSIA" => {
                Ok(VulkanCommand::SetBufferCollectionImageConstraintsFuchsia)
            }
            "vkSetBufferCollectionBufferConstraintsFUCHSIA" => {
                Ok(VulkanCommand::SetBufferCollectionBufferConstraintsFuchsia)
            }
            "vkDestroyBufferCollectionFUCHSIA" => Ok(VulkanCommand::DestroyBufferCollectionFuchsia),
            "vkGetBufferCollectionPropertiesFUCHSIA" => {
                Ok(VulkanCommand::GetBufferCollectionPropertiesFuchsia)
            }
            "vkGetDeviceSubpassShadingMaxWorkgroupSizeHUAWEI" => {
                Ok(VulkanCommand::GetDeviceSubpassShadingMaxWorkgroupSizeHuawei)
            }
            "vkCmdSubpassShadingHUAWEI" => Ok(VulkanCommand::CmdSubpassShadingHuawei),
            "vkCmdBindInvocationMaskHUAWEI" => Ok(VulkanCommand::CmdBindInvocationMaskHuawei),
            "vkGetMemoryRemoteAddressNV" => Ok(VulkanCommand::GetMemoryRemoteAddressNv),
            "vkGetPipelinePropertiesEXT" => Ok(VulkanCommand::GetPipelinePropertiesExt),
            "vkCmdSetPatchControlPointsEXT" => Ok(VulkanCommand::CmdSetPatchControlPointsExt),
            "vkCmdSetLogicOpEXT" => Ok(VulkanCommand::CmdSetLogicOpExt),
            "vkCmdSetColorWriteEnableEXT" => Ok(VulkanCommand::CmdSetColorWriteEnableExt),
            "vkCmdDrawMultiEXT" => Ok(VulkanCommand::CmdDrawMultiExt),
            "vkCmdDrawMultiIndexedEXT" => Ok(VulkanCommand::CmdDrawMultiIndexedExt),
            "vkCreateMicromapEXT" => Ok(VulkanCommand::CreateMicromapExt),
            "vkDestroyMicromapEXT" => Ok(VulkanCommand::DestroyMicromapExt),
            "vkCmdBuildMicromapsEXT" => Ok(VulkanCommand::CmdBuildMicromapsExt),
            "vkBuildMicromapsEXT" => Ok(VulkanCommand::BuildMicromapsExt),
            "vkCopyMicromapEXT" => Ok(VulkanCommand::CopyMicromapExt),
            "vkCopyMicromapToMemoryEXT" => Ok(VulkanCommand::CopyMicromapToMemoryExt),
            "vkCopyMemoryToMicromapEXT" => Ok(VulkanCommand::CopyMemoryToMicromapExt),
            "vkWriteMicromapsPropertiesEXT" => Ok(VulkanCommand::WriteMicromapsPropertiesExt),
            "vkCmdCopyMicromapEXT" => Ok(VulkanCommand::CmdCopyMicromapExt),
            "vkCmdCopyMicromapToMemoryEXT" => Ok(VulkanCommand::CmdCopyMicromapToMemoryExt),
            "vkCmdCopyMemoryToMicromapEXT" => Ok(VulkanCommand::CmdCopyMemoryToMicromapExt),
            "vkCmdWriteMicromapsPropertiesEXT" => Ok(VulkanCommand::CmdWriteMicromapsPropertiesExt),
            "vkGetDeviceMicromapCompatibilityEXT" => {
                Ok(VulkanCommand::GetDeviceMicromapCompatibilityExt)
            }
            "vkGetMicromapBuildSizesEXT" => Ok(VulkanCommand::GetMicromapBuildSizesExt),
            "vkSetDeviceMemoryPriorityEXT" => Ok(VulkanCommand::SetDeviceMemoryPriorityExt),
            "vkGetDescriptorSetLayoutHostMappingInfoVALVE" => {
                Ok(VulkanCommand::GetDescriptorSetLayoutHostMappingInfoValve)
            }
            "vkGetDescriptorSetHostMappingVALVE" => {
                Ok(VulkanCommand::GetDescriptorSetHostMappingValve)
            }
            "vkCmdCopyMemoryIndirectNV" => Ok(VulkanCommand::CmdCopyMemoryIndirectNv),
            "vkCmdCopyMemoryToImageIndirectNV" => Ok(VulkanCommand::CmdCopyMemoryToImageIndirectNv),
            "vkCmdDecompressMemoryNV" => Ok(VulkanCommand::CmdDecompressMemoryNv),
            "vkCmdDecompressMemoryIndirectCountNV" => {
                Ok(VulkanCommand::CmdDecompressMemoryIndirectCountNv)
            }
            "vkCmdSetTessellationDomainOriginEXT" => {
                Ok(VulkanCommand::CmdSetTessellationDomainOriginExt)
            }
            "vkCmdSetDepthClampEnableEXT" => Ok(VulkanCommand::CmdSetDepthClampEnableExt),
            "vkCmdSetPolygonModeEXT" => Ok(VulkanCommand::CmdSetPolygonModeExt),
            "vkCmdSetRasterizationSamplesEXT" => Ok(VulkanCommand::CmdSetRasterizationSamplesExt),
            "vkCmdSetSampleMaskEXT" => Ok(VulkanCommand::CmdSetSampleMaskExt),
            "vkCmdSetAlphaToCoverageEnableEXT" => Ok(VulkanCommand::CmdSetAlphaToCoverageEnableExt),
            "vkCmdSetAlphaToOneEnableEXT" => Ok(VulkanCommand::CmdSetAlphaToOneEnableExt),
            "vkCmdSetLogicOpEnableEXT" => Ok(VulkanCommand::CmdSetLogicOpEnableExt),
            "vkCmdSetColorBlendEnableEXT" => Ok(VulkanCommand::CmdSetColorBlendEnableExt),
            "vkCmdSetColorBlendEquationEXT" => Ok(VulkanCommand::CmdSetColorBlendEquationExt),
            "vkCmdSetColorWriteMaskEXT" => Ok(VulkanCommand::CmdSetColorWriteMaskExt),
            "vkCmdSetRasterizationStreamEXT" => Ok(VulkanCommand::CmdSetRasterizationStreamExt),
            "vkCmdSetConservativeRasterizationModeEXT" => {
                Ok(VulkanCommand::CmdSetConservativeRasterizationModeExt)
            }
            "vkCmdSetExtraPrimitiveOverestimationSizeEXT" => {
                Ok(VulkanCommand::CmdSetExtraPrimitiveOverestimationSizeExt)
            }
            "vkCmdSetDepthClipEnableEXT" => Ok(VulkanCommand::CmdSetDepthClipEnableExt),
            "vkCmdSetSampleLocationsEnableEXT" => Ok(VulkanCommand::CmdSetSampleLocationsEnableExt),
            "vkCmdSetColorBlendAdvancedEXT" => Ok(VulkanCommand::CmdSetColorBlendAdvancedExt),
            "vkCmdSetProvokingVertexModeEXT" => Ok(VulkanCommand::CmdSetProvokingVertexModeExt),
            "vkCmdSetLineRasterizationModeEXT" => Ok(VulkanCommand::CmdSetLineRasterizationModeExt),
            "vkCmdSetLineStippleEnableEXT" => Ok(VulkanCommand::CmdSetLineStippleEnableExt),
            "vkCmdSetDepthClipNegativeOneToOneEXT" => {
                Ok(VulkanCommand::CmdSetDepthClipNegativeOneToOneExt)
            }
            "vkCmdSetViewportWScalingEnableNV" => Ok(VulkanCommand::CmdSetViewportWScalingEnableNv),
            "vkCmdSetViewportSwizzleNV" => Ok(VulkanCommand::CmdSetViewportSwizzleNv),
            "vkCmdSetCoverageToColorEnableNV" => Ok(VulkanCommand::CmdSetCoverageToColorEnableNv),
            "vkCmdSetCoverageToColorLocationNV" => {
                Ok(VulkanCommand::CmdSetCoverageToColorLocationNv)
            }
            "vkCmdSetCoverageModulationModeNV" => Ok(VulkanCommand::CmdSetCoverageModulationModeNv),
            "vkCmdSetCoverageModulationTableEnableNV" => {
                Ok(VulkanCommand::CmdSetCoverageModulationTableEnableNv)
            }
            "vkCmdSetCoverageModulationTableNV" => {
                Ok(VulkanCommand::CmdSetCoverageModulationTableNv)
            }
            "vkCmdSetShadingRateImageEnableNV" => Ok(VulkanCommand::CmdSetShadingRateImageEnableNv),
            "vkCmdSetRepresentativeFragmentTestEnableNV" => {
                Ok(VulkanCommand::CmdSetRepresentativeFragmentTestEnableNv)
            }
            "vkCmdSetCoverageReductionModeNV" => Ok(VulkanCommand::CmdSetCoverageReductionModeNv),
            "vkGetShaderModuleIdentifierEXT" => Ok(VulkanCommand::GetShaderModuleIdentifierExt),
            "vkGetShaderModuleCreateInfoIdentifierEXT" => {
                Ok(VulkanCommand::GetShaderModuleCreateInfoIdentifierExt)
            }
            "vkCreateOpticalFlowSessionNV" => Ok(VulkanCommand::CreateOpticalFlowSessionNv),
            "vkDestroyOpticalFlowSessionNV" => Ok(VulkanCommand::DestroyOpticalFlowSessionNv),
            "vkBindOpticalFlowSessionImageNV" => Ok(VulkanCommand::BindOpticalFlowSessionImageNv),
            "vkCmdOpticalFlowExecuteNV" => Ok(VulkanCommand::CmdOpticalFlowExecuteNv),
            "vkGetFramebufferTilePropertiesQCOM" => {
                Ok(VulkanCommand::GetFramebufferTilePropertiesQcom)
            }
            "vkGetDynamicRenderingTilePropertiesQCOM" => {
                Ok(VulkanCommand::GetDynamicRenderingTilePropertiesQcom)
            }
            "vkCreateAccelerationStructureKHR" => Ok(VulkanCommand::CreateAccelerationStructureKhr),
            "vkDestroyAccelerationStructureKHR" => {
                Ok(VulkanCommand::DestroyAccelerationStructureKhr)
            }
            "vkCopyAccelerationStructureKHR" => Ok(VulkanCommand::CopyAccelerationStructureKhr),
            "vkCopyAccelerationStructureToMemoryKHR" => {
                Ok(VulkanCommand::CopyAccelerationStructureToMemoryKhr)
            }
            "vkCopyMemoryToAccelerationStructureKHR" => {
                Ok(VulkanCommand::CopyMemoryToAccelerationStructureKhr)
            }
            "vkWriteAccelerationStructuresPropertiesKHR" => {
                Ok(VulkanCommand::WriteAccelerationStructuresPropertiesKhr)
            }
            "vkCmdCopyAccelerationStructureKHR" => {
                Ok(VulkanCommand::CmdCopyAccelerationStructureKhr)
            }
            "vkCmdCopyAccelerationStructureToMemoryKHR" => {
                Ok(VulkanCommand::CmdCopyAccelerationStructureToMemoryKhr)
            }
            "vkCmdCopyMemoryToAccelerationStructureKHR" => {
                Ok(VulkanCommand::CmdCopyMemoryToAccelerationStructureKhr)
            }
            "vkGetAccelerationStructureDeviceAddressKHR" => {
                Ok(VulkanCommand::GetAccelerationStructureDeviceAddressKhr)
            }
            "vkCmdWriteAccelerationStructuresPropertiesKHR" => {
                Ok(VulkanCommand::CmdWriteAccelerationStructuresPropertiesKhr)
            }
            "vkGetDeviceAccelerationStructureCompatibilityKHR" => {
                Ok(VulkanCommand::GetDeviceAccelerationStructureCompatibilityKhr)
            }
            "vkGetAccelerationStructureBuildSizesKHR" => {
                Ok(VulkanCommand::GetAccelerationStructureBuildSizesKhr)
            }
            "vkCmdTraceRaysKHR" => Ok(VulkanCommand::CmdTraceRaysKhr),
            "vkCreateRayTracingPipelinesKHR" => Ok(VulkanCommand::CreateRayTracingPipelinesKhr),
            "vkGetRayTracingCaptureReplayShaderGroupHandlesKHR" => {
                Ok(VulkanCommand::GetRayTracingCaptureReplayShaderGroupHandlesKhr)
            }
            "vkCmdTraceRaysIndirectKHR" => Ok(VulkanCommand::CmdTraceRaysIndirectKhr),
            "vkGetRayTracingShaderGroupStackSizeKHR" => {
                Ok(VulkanCommand::GetRayTracingShaderGroupStackSizeKhr)
            }
            "vkCmdSetRayTracingPipelineStackSizeKHR" => {
                Ok(VulkanCommand::CmdSetRayTracingPipelineStackSizeKhr)
            }
            "vkCmdDrawMeshTasksEXT" => Ok(VulkanCommand::CmdDrawMeshTasksExt),
            "vkCmdDrawMeshTasksIndirectEXT" => Ok(VulkanCommand::CmdDrawMeshTasksIndirectExt),
            "vkCmdDrawMeshTasksIndirectCountEXT" => {
                Ok(VulkanCommand::CmdDrawMeshTasksIndirectCountExt)
            }
            _ => Err(TryFromVulkanCommandError::UnknownCommand(value.to_owned())),
        }
    }
}

pub trait DeviceHooks: Send + Sync {
    fn get_device_proc_addr(&self, _p_name: &str) -> LayerResult<vk::PFN_vkVoidFunction> {
        LayerResult::Unhandled
    }
    fn get_device_queue(
        &self,
        _queue_family_index: u32,
        _queue_index: u32,
    ) -> LayerResult<vk::Queue> {
        LayerResult::Unhandled
    }
    fn queue_submit(
        &self,
        _queue: vk::Queue,
        _p_submits: &[vk::SubmitInfo],
        _fence: vk::Fence,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn queue_wait_idle(&self, _queue: vk::Queue) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn device_wait_idle(&self) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn allocate_memory(
        &self,
        _p_allocate_info: &vk::MemoryAllocateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DeviceMemory>> {
        LayerResult::Unhandled
    }
    fn free_memory(
        &self,
        _memory: vk::DeviceMemory,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn map_memory(
        &self,
        _memory: vk::DeviceMemory,
        _offset: vk::DeviceSize,
        _size: vk::DeviceSize,
        _flags: vk::MemoryMapFlags,
    ) -> LayerResult<VkResult<Option<*mut c_void>>> {
        LayerResult::Unhandled
    }
    fn unmap_memory(&self, _memory: vk::DeviceMemory) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn flush_mapped_memory_ranges(
        &self,
        _p_memory_ranges: &[vk::MappedMemoryRange],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn invalidate_mapped_memory_ranges(
        &self,
        _p_memory_ranges: &[vk::MappedMemoryRange],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_device_memory_commitment(
        &self,
        _memory: vk::DeviceMemory,
    ) -> LayerResult<vk::DeviceSize> {
        LayerResult::Unhandled
    }
    fn bind_buffer_memory(
        &self,
        _buffer: vk::Buffer,
        _memory: vk::DeviceMemory,
        _memory_offset: vk::DeviceSize,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn bind_image_memory(
        &self,
        _image: vk::Image,
        _memory: vk::DeviceMemory,
        _memory_offset: vk::DeviceSize,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_buffer_memory_requirements(
        &self,
        _buffer: vk::Buffer,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_memory_requirements(
        &self,
        _image: vk::Image,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_sparse_memory_requirements(
        &self,
        _image: vk::Image,
        _p_sparse_memory_requirement_count: &mut MaybeUninit<u32>,
        _p_sparse_memory_requirements: Option<
            &mut [MaybeUninit<vk::SparseImageMemoryRequirements>],
        >,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn queue_bind_sparse(
        &self,
        _queue: vk::Queue,
        _p_bind_info: &[vk::BindSparseInfo],
        _fence: vk::Fence,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_fence(
        &self,
        _p_create_info: &vk::FenceCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Fence>> {
        LayerResult::Unhandled
    }
    fn destroy_fence(
        &self,
        _fence: vk::Fence,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn reset_fences(&self, _p_fences: &[vk::Fence]) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_fence_status(&self, _fence: vk::Fence) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn wait_for_fences(
        &self,
        _p_fences: &[vk::Fence],
        _wait_all: bool,
        _timeout: u64,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_semaphore(
        &self,
        _p_create_info: &vk::SemaphoreCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Semaphore>> {
        LayerResult::Unhandled
    }
    fn destroy_semaphore(
        &self,
        _semaphore: vk::Semaphore,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_event(
        &self,
        _p_create_info: &vk::EventCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Event>> {
        LayerResult::Unhandled
    }
    fn destroy_event(
        &self,
        _event: vk::Event,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_event_status(&self, _event: vk::Event) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn set_event(&self, _event: vk::Event) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn reset_event(&self, _event: vk::Event) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_query_pool(
        &self,
        _p_create_info: &vk::QueryPoolCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::QueryPool>> {
        LayerResult::Unhandled
    }
    fn destroy_query_pool(
        &self,
        _query_pool: vk::QueryPool,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_query_pool_results(
        &self,
        _query_pool: vk::QueryPool,
        _first_query: u32,
        _query_count: u32,
        _p_data: &mut [MaybeUninit<u8>],
        _stride: vk::DeviceSize,
        _flags: vk::QueryResultFlags,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_buffer(
        &self,
        _p_create_info: &vk::BufferCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Buffer>> {
        LayerResult::Unhandled
    }
    fn destroy_buffer(
        &self,
        _buffer: vk::Buffer,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_buffer_view(
        &self,
        _p_create_info: &vk::BufferViewCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::BufferView>> {
        LayerResult::Unhandled
    }
    fn destroy_buffer_view(
        &self,
        _buffer_view: vk::BufferView,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_image(
        &self,
        _p_create_info: &vk::ImageCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Image>> {
        LayerResult::Unhandled
    }
    fn destroy_image(
        &self,
        _image: vk::Image,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_subresource_layout(
        &self,
        _image: vk::Image,
        _p_subresource: &vk::ImageSubresource,
        _p_layout: &mut MaybeUninit<vk::SubresourceLayout>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_image_view(
        &self,
        _p_create_info: &vk::ImageViewCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::ImageView>> {
        LayerResult::Unhandled
    }
    fn destroy_image_view(
        &self,
        _image_view: vk::ImageView,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_shader_module(
        &self,
        _p_create_info: &vk::ShaderModuleCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::ShaderModule>> {
        LayerResult::Unhandled
    }
    fn destroy_shader_module(
        &self,
        _shader_module: vk::ShaderModule,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_pipeline_cache(
        &self,
        _p_create_info: &vk::PipelineCacheCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::PipelineCache>> {
        LayerResult::Unhandled
    }
    fn destroy_pipeline_cache(
        &self,
        _pipeline_cache: vk::PipelineCache,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_pipeline_cache_data(
        &self,
        _pipeline_cache: vk::PipelineCache,
        _p_data_size: &mut MaybeUninit<usize>,
        _p_data: Option<&mut [MaybeUninit<u8>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn merge_pipeline_caches(
        &self,
        _dst_cache: vk::PipelineCache,
        _p_src_caches: &[vk::PipelineCache],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_graphics_pipelines(
        &self,
        _pipeline_cache: vk::PipelineCache,
        _p_create_infos: &[vk::GraphicsPipelineCreateInfo],
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<Vec<vk::Pipeline>>> {
        LayerResult::Unhandled
    }
    fn create_compute_pipelines(
        &self,
        _pipeline_cache: vk::PipelineCache,
        _p_create_infos: &[vk::ComputePipelineCreateInfo],
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<Vec<vk::Pipeline>>> {
        LayerResult::Unhandled
    }
    fn destroy_pipeline(
        &self,
        _pipeline: vk::Pipeline,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_pipeline_layout(
        &self,
        _p_create_info: &vk::PipelineLayoutCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::PipelineLayout>> {
        LayerResult::Unhandled
    }
    fn destroy_pipeline_layout(
        &self,
        _pipeline_layout: vk::PipelineLayout,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_sampler(
        &self,
        _p_create_info: &vk::SamplerCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Sampler>> {
        LayerResult::Unhandled
    }
    fn destroy_sampler(
        &self,
        _sampler: vk::Sampler,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_descriptor_set_layout(
        &self,
        _p_create_info: &vk::DescriptorSetLayoutCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DescriptorSetLayout>> {
        LayerResult::Unhandled
    }
    fn destroy_descriptor_set_layout(
        &self,
        _descriptor_set_layout: vk::DescriptorSetLayout,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_descriptor_pool(
        &self,
        _p_create_info: &vk::DescriptorPoolCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DescriptorPool>> {
        LayerResult::Unhandled
    }
    fn destroy_descriptor_pool(
        &self,
        _descriptor_pool: vk::DescriptorPool,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn reset_descriptor_pool(
        &self,
        _descriptor_pool: vk::DescriptorPool,
        _flags: vk::DescriptorPoolResetFlags,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn allocate_descriptor_sets(
        &self,
        _p_allocate_info: &vk::DescriptorSetAllocateInfo,
    ) -> LayerResult<VkResult<Vec<vk::DescriptorSet>>> {
        LayerResult::Unhandled
    }
    fn free_descriptor_sets(
        &self,
        _descriptor_pool: vk::DescriptorPool,
        _p_descriptor_sets: &[vk::DescriptorSet],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn update_descriptor_sets(
        &self,
        _p_descriptor_writes: &[vk::WriteDescriptorSet],
        _p_descriptor_copies: &[vk::CopyDescriptorSet],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_framebuffer(
        &self,
        _p_create_info: &vk::FramebufferCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Framebuffer>> {
        LayerResult::Unhandled
    }
    fn destroy_framebuffer(
        &self,
        _framebuffer: vk::Framebuffer,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_render_pass(
        &self,
        _p_create_info: &vk::RenderPassCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::RenderPass>> {
        LayerResult::Unhandled
    }
    fn destroy_render_pass(
        &self,
        _render_pass: vk::RenderPass,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_render_area_granularity(
        &self,
        _render_pass: vk::RenderPass,
        _p_granularity: &mut MaybeUninit<vk::Extent2D>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_command_pool(
        &self,
        _p_create_info: &vk::CommandPoolCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::CommandPool>> {
        LayerResult::Unhandled
    }
    fn destroy_command_pool(
        &self,
        _command_pool: vk::CommandPool,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn reset_command_pool(
        &self,
        _command_pool: vk::CommandPool,
        _flags: vk::CommandPoolResetFlags,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn allocate_command_buffers(
        &self,
        _p_allocate_info: &vk::CommandBufferAllocateInfo,
    ) -> LayerResult<VkResult<Vec<vk::CommandBuffer>>> {
        LayerResult::Unhandled
    }
    fn free_command_buffers(
        &self,
        _command_pool: vk::CommandPool,
        _p_command_buffers: &[vk::CommandBuffer],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn begin_command_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_begin_info: &vk::CommandBufferBeginInfo,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn end_command_buffer(&self, _command_buffer: vk::CommandBuffer) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn reset_command_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _flags: vk::CommandBufferResetFlags,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_bind_pipeline(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_bind_point: vk::PipelineBindPoint,
        _pipeline: vk::Pipeline,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_viewport(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_viewport: u32,
        _p_viewports: &[vk::Viewport],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_scissor(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_scissor: u32,
        _p_scissors: &[vk::Rect2D],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_line_width(
        &self,
        _command_buffer: vk::CommandBuffer,
        _line_width: f32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_bias(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_bias_constant_factor: f32,
        _depth_bias_clamp: f32,
        _depth_bias_slope_factor: f32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_blend_constants(
        &self,
        _command_buffer: vk::CommandBuffer,
        _blend_constants: &[f32; 4],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_bounds(
        &self,
        _command_buffer: vk::CommandBuffer,
        _min_depth_bounds: f32,
        _max_depth_bounds: f32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_stencil_compare_mask(
        &self,
        _command_buffer: vk::CommandBuffer,
        _face_mask: vk::StencilFaceFlags,
        _compare_mask: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_stencil_write_mask(
        &self,
        _command_buffer: vk::CommandBuffer,
        _face_mask: vk::StencilFaceFlags,
        _write_mask: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_stencil_reference(
        &self,
        _command_buffer: vk::CommandBuffer,
        _face_mask: vk::StencilFaceFlags,
        _reference: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_descriptor_sets(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_bind_point: vk::PipelineBindPoint,
        _layout: vk::PipelineLayout,
        _first_set: u32,
        _p_descriptor_sets: &[vk::DescriptorSet],
        _p_dynamic_offsets: &[u32],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_index_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _index_type: vk::IndexType,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_vertex_buffers(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_binding: u32,
        _p_buffers: &[vk::Buffer],
        _p_offsets: &[vk::DeviceSize],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw(
        &self,
        _command_buffer: vk::CommandBuffer,
        _vertex_count: u32,
        _instance_count: u32,
        _first_vertex: u32,
        _first_instance: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_indexed(
        &self,
        _command_buffer: vk::CommandBuffer,
        _index_count: u32,
        _instance_count: u32,
        _first_index: u32,
        _vertex_offset: i32,
        _first_instance: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_indirect(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_indexed_indirect(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_dispatch(
        &self,
        _command_buffer: vk::CommandBuffer,
        _group_countx: u32,
        _group_county: u32,
        _group_countz: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_dispatch_indirect(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_buffer: vk::Buffer,
        _dst_buffer: vk::Buffer,
        _p_regions: &[vk::BufferCopy],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_image(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_image: vk::Image,
        _src_image_layout: vk::ImageLayout,
        _dst_image: vk::Image,
        _dst_image_layout: vk::ImageLayout,
        _p_regions: &[vk::ImageCopy],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_blit_image(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_image: vk::Image,
        _src_image_layout: vk::ImageLayout,
        _dst_image: vk::Image,
        _dst_image_layout: vk::ImageLayout,
        _p_regions: &[vk::ImageBlit],
        _filter: vk::Filter,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_buffer_to_image(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_buffer: vk::Buffer,
        _dst_image: vk::Image,
        _dst_image_layout: vk::ImageLayout,
        _p_regions: &[vk::BufferImageCopy],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_image_to_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_image: vk::Image,
        _src_image_layout: vk::ImageLayout,
        _dst_buffer: vk::Buffer,
        _p_regions: &[vk::BufferImageCopy],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_update_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _dst_buffer: vk::Buffer,
        _dst_offset: vk::DeviceSize,
        _p_data: &[u8],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_fill_buffer(
        &self,
        _command_buffer: vk::CommandBuffer,
        _dst_buffer: vk::Buffer,
        _dst_offset: vk::DeviceSize,
        _size: vk::DeviceSize,
        _data: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_clear_color_image(
        &self,
        _command_buffer: vk::CommandBuffer,
        _image: vk::Image,
        _image_layout: vk::ImageLayout,
        _p_color: &vk::ClearColorValue,
        _p_ranges: &[vk::ImageSubresourceRange],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_clear_depth_stencil_image(
        &self,
        _command_buffer: vk::CommandBuffer,
        _image: vk::Image,
        _image_layout: vk::ImageLayout,
        _p_depth_stencil: &vk::ClearDepthStencilValue,
        _p_ranges: &[vk::ImageSubresourceRange],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_clear_attachments(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_attachments: &[vk::ClearAttachment],
        _p_rects: &[vk::ClearRect],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_resolve_image(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_image: vk::Image,
        _src_image_layout: vk::ImageLayout,
        _dst_image: vk::Image,
        _dst_image_layout: vk::ImageLayout,
        _p_regions: &[vk::ImageResolve],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_event(
        &self,
        _command_buffer: vk::CommandBuffer,
        _event: vk::Event,
        _stage_mask: vk::PipelineStageFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_reset_event(
        &self,
        _command_buffer: vk::CommandBuffer,
        _event: vk::Event,
        _stage_mask: vk::PipelineStageFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_wait_events(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_events: &[vk::Event],
        _src_stage_mask: vk::PipelineStageFlags,
        _dst_stage_mask: vk::PipelineStageFlags,
        _p_memory_barriers: &[vk::MemoryBarrier],
        _p_buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        _p_image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_pipeline_barrier(
        &self,
        _command_buffer: vk::CommandBuffer,
        _src_stage_mask: vk::PipelineStageFlags,
        _dst_stage_mask: vk::PipelineStageFlags,
        _dependency_flags: vk::DependencyFlags,
        _p_memory_barriers: &[vk::MemoryBarrier],
        _p_buffer_memory_barriers: &[vk::BufferMemoryBarrier],
        _p_image_memory_barriers: &[vk::ImageMemoryBarrier],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_query(
        &self,
        _command_buffer: vk::CommandBuffer,
        _query_pool: vk::QueryPool,
        _query: u32,
        _flags: vk::QueryControlFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_query(
        &self,
        _command_buffer: vk::CommandBuffer,
        _query_pool: vk::QueryPool,
        _query: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_reset_query_pool(
        &self,
        _command_buffer: vk::CommandBuffer,
        _query_pool: vk::QueryPool,
        _first_query: u32,
        _query_count: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_write_timestamp(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_stage: vk::PipelineStageFlags,
        _query_pool: vk::QueryPool,
        _query: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_query_pool_results(
        &self,
        _command_buffer: vk::CommandBuffer,
        _query_pool: vk::QueryPool,
        _first_query: u32,
        _query_count: u32,
        _dst_buffer: vk::Buffer,
        _dst_offset: vk::DeviceSize,
        _stride: vk::DeviceSize,
        _flags: vk::QueryResultFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_push_constants(
        &self,
        _command_buffer: vk::CommandBuffer,
        _layout: vk::PipelineLayout,
        _stage_flags: vk::ShaderStageFlags,
        _offset: u32,
        _p_values: &[u8],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_render_pass(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_render_pass_begin: &vk::RenderPassBeginInfo,
        _contents: vk::SubpassContents,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_next_subpass(
        &self,
        _command_buffer: vk::CommandBuffer,
        _contents: vk::SubpassContents,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_render_pass(&self, _command_buffer: vk::CommandBuffer) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_execute_commands(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_command_buffers: &[vk::CommandBuffer],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn bind_buffer_memory2(
        &self,
        _p_bind_infos: &[vk::BindBufferMemoryInfo],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn bind_image_memory2(
        &self,
        _p_bind_infos: &[vk::BindImageMemoryInfo],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_device_group_peer_memory_features(
        &self,
        _heap_index: u32,
        _local_device_index: u32,
        _remote_device_index: u32,
    ) -> LayerResult<vk::PeerMemoryFeatureFlags> {
        LayerResult::Unhandled
    }
    fn cmd_set_device_mask(
        &self,
        _command_buffer: vk::CommandBuffer,
        _device_mask: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_dispatch_base(
        &self,
        _command_buffer: vk::CommandBuffer,
        _base_groupx: u32,
        _base_groupy: u32,
        _base_groupz: u32,
        _group_countx: u32,
        _group_county: u32,
        _group_countz: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_memory_requirements2(
        &self,
        _p_info: &vk::ImageMemoryRequirementsInfo2,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_buffer_memory_requirements2(
        &self,
        _p_info: &vk::BufferMemoryRequirementsInfo2,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_sparse_memory_requirements2(
        &self,
        _p_info: &vk::ImageSparseMemoryRequirementsInfo2,
        _p_sparse_memory_requirement_count: &mut MaybeUninit<u32>,
        _p_sparse_memory_requirements: Option<
            &mut [MaybeUninit<vk::SparseImageMemoryRequirements2>],
        >,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn trim_command_pool(
        &self,
        _command_pool: vk::CommandPool,
        _flags: vk::CommandPoolTrimFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_device_queue2(&self, _p_queue_info: &vk::DeviceQueueInfo2) -> LayerResult<vk::Queue> {
        LayerResult::Unhandled
    }
    fn create_sampler_ycbcr_conversion(
        &self,
        _p_create_info: &vk::SamplerYcbcrConversionCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SamplerYcbcrConversion>> {
        LayerResult::Unhandled
    }
    fn destroy_sampler_ycbcr_conversion(
        &self,
        _ycbcr_conversion: vk::SamplerYcbcrConversion,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_descriptor_update_template(
        &self,
        _p_create_info: &vk::DescriptorUpdateTemplateCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DescriptorUpdateTemplate>> {
        LayerResult::Unhandled
    }
    fn destroy_descriptor_update_template(
        &self,
        _descriptor_update_template: vk::DescriptorUpdateTemplate,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn update_descriptor_set_with_template(
        &self,
        _descriptor_set: vk::DescriptorSet,
        _descriptor_update_template: vk::DescriptorUpdateTemplate,
        _p_data: *const c_void,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_descriptor_set_layout_support(
        &self,
        _p_create_info: &vk::DescriptorSetLayoutCreateInfo,
        _p_support: &mut MaybeUninit<vk::DescriptorSetLayoutSupport>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_indirect_count(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _count_buffer: vk::Buffer,
        _count_buffer_offset: vk::DeviceSize,
        _max_draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_indexed_indirect_count(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _count_buffer: vk::Buffer,
        _count_buffer_offset: vk::DeviceSize,
        _max_draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_render_pass2(
        &self,
        _p_create_info: &vk::RenderPassCreateInfo2,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::RenderPass>> {
        LayerResult::Unhandled
    }
    fn cmd_begin_render_pass2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_render_pass_begin: &vk::RenderPassBeginInfo,
        _p_subpass_begin_info: &vk::SubpassBeginInfo,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_next_subpass2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_subpass_begin_info: &vk::SubpassBeginInfo,
        _p_subpass_end_info: &vk::SubpassEndInfo,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_render_pass2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_subpass_end_info: &vk::SubpassEndInfo,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn reset_query_pool(
        &self,
        _query_pool: vk::QueryPool,
        _first_query: u32,
        _query_count: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_semaphore_counter_value(&self, _semaphore: vk::Semaphore) -> LayerResult<VkResult<u64>> {
        LayerResult::Unhandled
    }
    fn wait_semaphores(
        &self,
        _p_wait_info: &vk::SemaphoreWaitInfo,
        _timeout: u64,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn signal_semaphore(
        &self,
        _p_signal_info: &vk::SemaphoreSignalInfo,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_buffer_device_address(
        &self,
        _p_info: &vk::BufferDeviceAddressInfo,
    ) -> LayerResult<vk::DeviceAddress> {
        LayerResult::Unhandled
    }
    fn get_buffer_opaque_capture_address(
        &self,
        _p_info: &vk::BufferDeviceAddressInfo,
    ) -> LayerResult<u64> {
        LayerResult::Unhandled
    }
    fn get_device_memory_opaque_capture_address(
        &self,
        _p_info: &vk::DeviceMemoryOpaqueCaptureAddressInfo,
    ) -> LayerResult<u64> {
        LayerResult::Unhandled
    }
    fn create_private_data_slot(
        &self,
        _p_create_info: &vk::PrivateDataSlotCreateInfo,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::PrivateDataSlot>> {
        LayerResult::Unhandled
    }
    fn destroy_private_data_slot(
        &self,
        _private_data_slot: vk::PrivateDataSlot,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn set_private_data(
        &self,
        _object_type: vk::ObjectType,
        _object_handle: u64,
        _private_data_slot: vk::PrivateDataSlot,
        _data: u64,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_private_data(
        &self,
        _object_type: vk::ObjectType,
        _object_handle: u64,
        _private_data_slot: vk::PrivateDataSlot,
    ) -> LayerResult<u64> {
        LayerResult::Unhandled
    }
    fn cmd_set_event2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _event: vk::Event,
        _p_dependency_info: &vk::DependencyInfo,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_reset_event2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _event: vk::Event,
        _stage_mask: vk::PipelineStageFlags2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_wait_events2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_events: &[vk::Event],
        _p_dependency_infos: &[vk::DependencyInfo],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_pipeline_barrier2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_dependency_info: &vk::DependencyInfo,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_write_timestamp2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _stage: vk::PipelineStageFlags2,
        _query_pool: vk::QueryPool,
        _query: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn queue_submit2(
        &self,
        _queue: vk::Queue,
        _p_submits: &[vk::SubmitInfo2],
        _fence: vk::Fence,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_copy_buffer2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_copy_buffer_info: &vk::CopyBufferInfo2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_image2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_copy_image_info: &vk::CopyImageInfo2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_buffer_to_image2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_copy_buffer_to_image_info: &vk::CopyBufferToImageInfo2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_image_to_buffer2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_copy_image_to_buffer_info: &vk::CopyImageToBufferInfo2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_blit_image2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_blit_image_info: &vk::BlitImageInfo2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_resolve_image2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_resolve_image_info: &vk::ResolveImageInfo2,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_rendering(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_rendering_info: &vk::RenderingInfo,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_rendering(&self, _command_buffer: vk::CommandBuffer) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_cull_mode(
        &self,
        _command_buffer: vk::CommandBuffer,
        _cull_mode: vk::CullModeFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_front_face(
        &self,
        _command_buffer: vk::CommandBuffer,
        _front_face: vk::FrontFace,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_primitive_topology(
        &self,
        _command_buffer: vk::CommandBuffer,
        _primitive_topology: vk::PrimitiveTopology,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_viewport_with_count(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_viewports: &[vk::Viewport],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_scissor_with_count(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_scissors: &[vk::Rect2D],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_vertex_buffers2(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_binding: u32,
        _p_buffers: &[vk::Buffer],
        _p_offsets: &[vk::DeviceSize],
        _p_sizes: Option<&[vk::DeviceSize]>,
        _p_strides: Option<&[vk::DeviceSize]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_test_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_test_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_write_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_write_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_compare_op(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_compare_op: vk::CompareOp,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_bounds_test_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_bounds_test_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_stencil_test_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _stencil_test_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_stencil_op(
        &self,
        _command_buffer: vk::CommandBuffer,
        _face_mask: vk::StencilFaceFlags,
        _fail_op: vk::StencilOp,
        _pass_op: vk::StencilOp,
        _depth_fail_op: vk::StencilOp,
        _compare_op: vk::CompareOp,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_rasterizer_discard_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _rasterizer_discard_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_bias_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_bias_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_primitive_restart_enable(
        &self,
        _command_buffer: vk::CommandBuffer,
        _primitive_restart_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_device_buffer_memory_requirements(
        &self,
        _p_info: &vk::DeviceBufferMemoryRequirements,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_device_image_memory_requirements(
        &self,
        _p_info: &vk::DeviceImageMemoryRequirements,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_device_image_sparse_memory_requirements(
        &self,
        _p_info: &vk::DeviceImageMemoryRequirements,
        _p_sparse_memory_requirement_count: &mut MaybeUninit<u32>,
        _p_sparse_memory_requirements: Option<
            &mut [MaybeUninit<vk::SparseImageMemoryRequirements2>],
        >,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_swapchain_khr(
        &self,
        _p_create_info: &vk::SwapchainCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SwapchainKHR>> {
        LayerResult::Unhandled
    }
    fn destroy_swapchain_khr(
        &self,
        _swapchain: vk::SwapchainKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_swapchain_images_khr(
        &self,
        _swapchain: vk::SwapchainKHR,
    ) -> LayerResult<VkResult<Vec<vk::Image>>> {
        LayerResult::Unhandled
    }
    fn acquire_next_image_khr(
        &self,
        _swapchain: vk::SwapchainKHR,
        _timeout: u64,
        _semaphore: vk::Semaphore,
        _fence: vk::Fence,
    ) -> LayerResult<VkResult<u32>> {
        LayerResult::Unhandled
    }
    fn queue_present_khr(
        &self,
        _queue: vk::Queue,
        _p_present_info: &vk::PresentInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_device_group_present_capabilities_khr(
        &self,
        _p_device_group_present_capabilities: &mut MaybeUninit<
            vk::DeviceGroupPresentCapabilitiesKHR,
        >,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_device_group_surface_present_modes_khr(
        &self,
        _surface: vk::SurfaceKHR,
    ) -> LayerResult<VkResult<vk::DeviceGroupPresentModeFlagsKHR>> {
        LayerResult::Unhandled
    }
    fn acquire_next_image2_khr(
        &self,
        _p_acquire_info: &vk::AcquireNextImageInfoKHR,
    ) -> LayerResult<VkResult<u32>> {
        LayerResult::Unhandled
    }
    fn create_shared_swapchains_khr(
        &self,
        _p_create_infos: &[vk::SwapchainCreateInfoKHR],
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<Vec<vk::SwapchainKHR>>> {
        LayerResult::Unhandled
    }
    fn create_video_session_khr(
        &self,
        _p_create_info: &vk::VideoSessionCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::VideoSessionKHR>> {
        LayerResult::Unhandled
    }
    fn destroy_video_session_khr(
        &self,
        _video_session: vk::VideoSessionKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_video_session_memory_requirements_khr(
        &self,
        _video_session: vk::VideoSessionKHR,
        _p_memory_requirements_count: &mut MaybeUninit<u32>,
        _p_memory_requirements: Option<&mut [MaybeUninit<vk::VideoSessionMemoryRequirementsKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn bind_video_session_memory_khr(
        &self,
        _video_session: vk::VideoSessionKHR,
        _p_bind_session_memory_infos: &[vk::BindVideoSessionMemoryInfoKHR],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_video_session_parameters_khr(
        &self,
        _p_create_info: &vk::VideoSessionParametersCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::VideoSessionParametersKHR>> {
        LayerResult::Unhandled
    }
    fn update_video_session_parameters_khr(
        &self,
        _video_session_parameters: vk::VideoSessionParametersKHR,
        _p_update_info: &vk::VideoSessionParametersUpdateInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn destroy_video_session_parameters_khr(
        &self,
        _video_session_parameters: vk::VideoSessionParametersKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_video_coding_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_begin_info: &vk::VideoBeginCodingInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_video_coding_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_end_coding_info: &vk::VideoEndCodingInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_control_video_coding_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_coding_control_info: &vk::VideoCodingControlInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_decode_video_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_decode_info: &vk::VideoDecodeInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_memory_win32_handle_khr(
        &self,
        _p_get_win32_handle_info: &vk::MemoryGetWin32HandleInfoKHR,
    ) -> LayerResult<VkResult<vk::HANDLE>> {
        LayerResult::Unhandled
    }
    fn get_memory_win32_handle_properties_khr(
        &self,
        _handle_type: vk::ExternalMemoryHandleTypeFlags,
        _handle: vk::HANDLE,
        _p_memory_win32_handle_properties: &mut MaybeUninit<vk::MemoryWin32HandlePropertiesKHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_memory_fd_khr(
        &self,
        _p_get_fd_info: &vk::MemoryGetFdInfoKHR,
    ) -> LayerResult<VkResult<c_int>> {
        LayerResult::Unhandled
    }
    fn get_memory_fd_properties_khr(
        &self,
        _handle_type: vk::ExternalMemoryHandleTypeFlags,
        _fd: c_int,
        _p_memory_fd_properties: &mut MaybeUninit<vk::MemoryFdPropertiesKHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn import_semaphore_win32_handle_khr(
        &self,
        _p_import_semaphore_win32_handle_info: &vk::ImportSemaphoreWin32HandleInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_semaphore_win32_handle_khr(
        &self,
        _p_get_win32_handle_info: &vk::SemaphoreGetWin32HandleInfoKHR,
    ) -> LayerResult<VkResult<vk::HANDLE>> {
        LayerResult::Unhandled
    }
    fn import_semaphore_fd_khr(
        &self,
        _p_import_semaphore_fd_info: &vk::ImportSemaphoreFdInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_semaphore_fd_khr(
        &self,
        _p_get_fd_info: &vk::SemaphoreGetFdInfoKHR,
    ) -> LayerResult<VkResult<c_int>> {
        LayerResult::Unhandled
    }
    fn cmd_push_descriptor_set_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_bind_point: vk::PipelineBindPoint,
        _layout: vk::PipelineLayout,
        _set: u32,
        _p_descriptor_writes: &[vk::WriteDescriptorSet],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_push_descriptor_set_with_template_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _descriptor_update_template: vk::DescriptorUpdateTemplate,
        _layout: vk::PipelineLayout,
        _set: u32,
        _p_data: *const c_void,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_swapchain_status_khr(&self, _swapchain: vk::SwapchainKHR) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn import_fence_win32_handle_khr(
        &self,
        _p_import_fence_win32_handle_info: &vk::ImportFenceWin32HandleInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_fence_win32_handle_khr(
        &self,
        _p_get_win32_handle_info: &vk::FenceGetWin32HandleInfoKHR,
    ) -> LayerResult<VkResult<vk::HANDLE>> {
        LayerResult::Unhandled
    }
    fn import_fence_fd_khr(
        &self,
        _p_import_fence_fd_info: &vk::ImportFenceFdInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_fence_fd_khr(
        &self,
        _p_get_fd_info: &vk::FenceGetFdInfoKHR,
    ) -> LayerResult<VkResult<c_int>> {
        LayerResult::Unhandled
    }
    fn acquire_profiling_lock_khr(
        &self,
        _p_info: &vk::AcquireProfilingLockInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn release_profiling_lock_khr(&self) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_fragment_shading_rate_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_fragment_size: &vk::Extent2D,
        _combiner_ops: &[vk::FragmentShadingRateCombinerOpKHR; 2],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn wait_for_present_khr(
        &self,
        _swapchain: vk::SwapchainKHR,
        _present_id: u64,
        _timeout: u64,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_deferred_operation_khr(
        &self,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DeferredOperationKHR>> {
        LayerResult::Unhandled
    }
    fn destroy_deferred_operation_khr(
        &self,
        _operation: vk::DeferredOperationKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_deferred_operation_max_concurrency_khr(
        &self,
        _operation: vk::DeferredOperationKHR,
    ) -> LayerResult<u32> {
        LayerResult::Unhandled
    }
    fn get_deferred_operation_result_khr(
        &self,
        _operation: vk::DeferredOperationKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn deferred_operation_join_khr(
        &self,
        _operation: vk::DeferredOperationKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_pipeline_executable_properties_khr(
        &self,
        _p_pipeline_info: &vk::PipelineInfoKHR,
        _p_executable_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::PipelineExecutablePropertiesKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_pipeline_executable_statistics_khr(
        &self,
        _p_executable_info: &vk::PipelineExecutableInfoKHR,
        _p_statistic_count: &mut MaybeUninit<u32>,
        _p_statistics: Option<&mut [MaybeUninit<vk::PipelineExecutableStatisticKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_pipeline_executable_internal_representations_khr(
        &self,
        _p_executable_info: &vk::PipelineExecutableInfoKHR,
        _p_internal_representation_count: &mut MaybeUninit<u32>,
        _p_internal_representations: Option<
            &mut [MaybeUninit<vk::PipelineExecutableInternalRepresentationKHR>],
        >,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_encode_video_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_encode_info: &vk::VideoEncodeInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_write_buffer_marker2_amd(
        &self,
        _command_buffer: vk::CommandBuffer,
        _stage: vk::PipelineStageFlags2,
        _dst_buffer: vk::Buffer,
        _dst_offset: vk::DeviceSize,
        _marker: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_queue_checkpoint_data2_nv(
        &self,
        _queue: vk::Queue,
        _p_checkpoint_data_count: &mut MaybeUninit<u32>,
        _p_checkpoint_data: Option<&mut [MaybeUninit<vk::CheckpointData2NV>]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_trace_rays_indirect2_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _indirect_device_address: vk::DeviceAddress,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_swapchain_gralloc_usage_android(
        &self,
        _format: vk::Format,
        _image_usage: vk::ImageUsageFlags,
    ) -> LayerResult<VkResult<c_int>> {
        LayerResult::Unhandled
    }
    fn acquire_image_android(
        &self,
        _image: vk::Image,
        _native_fence_fd: c_int,
        _semaphore: vk::Semaphore,
        _fence: vk::Fence,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn queue_signal_release_image_android(
        &self,
        _queue: vk::Queue,
        _p_wait_semaphores: &[vk::Semaphore],
        _image: vk::Image,
    ) -> LayerResult<VkResult<c_int>> {
        LayerResult::Unhandled
    }
    fn get_swapchain_gralloc_usage2_android(
        &self,
        _format: vk::Format,
        _image_usage: vk::ImageUsageFlags,
        _swapchain_image_usage: vk::SwapchainImageUsageFlagsANDROID,
        _gralloc_consumer_usage: &mut MaybeUninit<u64>,
    ) -> LayerResult<VkResult<u64>> {
        LayerResult::Unhandled
    }
    fn debug_marker_set_object_tag_ext(
        &self,
        _p_tag_info: &vk::DebugMarkerObjectTagInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn debug_marker_set_object_name_ext(
        &self,
        _p_name_info: &vk::DebugMarkerObjectNameInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_debug_marker_begin_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_marker_info: &vk::DebugMarkerMarkerInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_debug_marker_end_ext(&self, _command_buffer: vk::CommandBuffer) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_debug_marker_insert_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_marker_info: &vk::DebugMarkerMarkerInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_transform_feedback_buffers_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_binding: u32,
        _p_buffers: &[vk::Buffer],
        _p_offsets: &[vk::DeviceSize],
        _p_sizes: Option<&[vk::DeviceSize]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_transform_feedback_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_counter_buffer: u32,
        _p_counter_buffers: &[vk::Buffer],
        _p_counter_buffer_offsets: Option<&[vk::DeviceSize]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_transform_feedback_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_counter_buffer: u32,
        _p_counter_buffers: &[vk::Buffer],
        _p_counter_buffer_offsets: Option<&[vk::DeviceSize]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_query_indexed_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _query_pool: vk::QueryPool,
        _query: u32,
        _flags: vk::QueryControlFlags,
        _index: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_query_indexed_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _query_pool: vk::QueryPool,
        _query: u32,
        _index: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_indirect_byte_count_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _instance_count: u32,
        _first_instance: u32,
        _counter_buffer: vk::Buffer,
        _counter_buffer_offset: vk::DeviceSize,
        _counter_offset: u32,
        _vertex_stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_cu_module_nvx(
        &self,
        _p_create_info: &vk::CuModuleCreateInfoNVX,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::CuModuleNVX>> {
        LayerResult::Unhandled
    }
    fn create_cu_function_nvx(
        &self,
        _p_create_info: &vk::CuFunctionCreateInfoNVX,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::CuFunctionNVX>> {
        LayerResult::Unhandled
    }
    fn destroy_cu_module_nvx(
        &self,
        _module: vk::CuModuleNVX,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn destroy_cu_function_nvx(
        &self,
        _function: vk::CuFunctionNVX,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_cu_launch_kernel_nvx(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_launch_info: &vk::CuLaunchInfoNVX,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_view_handle_nvx(&self, _p_info: &vk::ImageViewHandleInfoNVX) -> LayerResult<u32> {
        LayerResult::Unhandled
    }
    fn get_image_view_address_nvx(
        &self,
        _image_view: vk::ImageView,
        _p_properties: &mut MaybeUninit<vk::ImageViewAddressPropertiesNVX>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_shader_info_amd(
        &self,
        _pipeline: vk::Pipeline,
        _shader_stage: vk::ShaderStageFlags,
        _info_type: vk::ShaderInfoTypeAMD,
        _p_info_size: &mut MaybeUninit<usize>,
        _p_info: Option<&mut [MaybeUninit<u8>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_memory_win32_handle_nv(
        &self,
        _memory: vk::DeviceMemory,
        _handle_type: vk::ExternalMemoryHandleTypeFlagsNV,
    ) -> LayerResult<VkResult<vk::HANDLE>> {
        LayerResult::Unhandled
    }
    fn cmd_begin_conditional_rendering_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_conditional_rendering_begin: &vk::ConditionalRenderingBeginInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_conditional_rendering_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_viewport_w_scaling_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_viewport: u32,
        _p_viewport_w_scalings: &[vk::ViewportWScalingNV],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn display_power_control_ext(
        &self,
        _display: vk::DisplayKHR,
        _p_display_power_info: &vk::DisplayPowerInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn register_device_event_ext(
        &self,
        _p_device_event_info: &vk::DeviceEventInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Fence>> {
        LayerResult::Unhandled
    }
    fn register_display_event_ext(
        &self,
        _display: vk::DisplayKHR,
        _p_display_event_info: &vk::DisplayEventInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::Fence>> {
        LayerResult::Unhandled
    }
    fn get_swapchain_counter_ext(
        &self,
        _swapchain: vk::SwapchainKHR,
        _counter: vk::SurfaceCounterFlagsEXT,
    ) -> LayerResult<VkResult<u64>> {
        LayerResult::Unhandled
    }
    fn get_refresh_cycle_duration_google(
        &self,
        _swapchain: vk::SwapchainKHR,
        _p_display_timing_properties: &mut MaybeUninit<vk::RefreshCycleDurationGOOGLE>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_past_presentation_timing_google(
        &self,
        _swapchain: vk::SwapchainKHR,
        _p_presentation_timing_count: &mut MaybeUninit<u32>,
        _p_presentation_timings: Option<&mut [MaybeUninit<vk::PastPresentationTimingGOOGLE>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_set_discard_rectangle_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_discard_rectangle: u32,
        _p_discard_rectangles: &[vk::Rect2D],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn set_hdr_metadata_ext(
        &self,
        _p_swapchains: &[vk::SwapchainKHR],
        _p_metadata: &[vk::HdrMetadataEXT],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn set_debug_utils_object_name_ext(
        &self,
        _p_name_info: &vk::DebugUtilsObjectNameInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn set_debug_utils_object_tag_ext(
        &self,
        _p_tag_info: &vk::DebugUtilsObjectTagInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn queue_begin_debug_utils_label_ext(
        &self,
        _queue: vk::Queue,
        _p_label_info: &vk::DebugUtilsLabelEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn queue_end_debug_utils_label_ext(&self, _queue: vk::Queue) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn queue_insert_debug_utils_label_ext(
        &self,
        _queue: vk::Queue,
        _p_label_info: &vk::DebugUtilsLabelEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_begin_debug_utils_label_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_label_info: &vk::DebugUtilsLabelEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_end_debug_utils_label_ext(&self, _command_buffer: vk::CommandBuffer) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_insert_debug_utils_label_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_label_info: &vk::DebugUtilsLabelEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_android_hardware_buffer_properties_android(
        &self,
        _buffer: *const vk::AHardwareBuffer,
        _p_properties: &mut MaybeUninit<vk::AndroidHardwareBufferPropertiesANDROID>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_memory_android_hardware_buffer_android(
        &self,
        _p_info: &vk::MemoryGetAndroidHardwareBufferInfoANDROID,
    ) -> LayerResult<VkResult<*mut vk::AHardwareBuffer>> {
        LayerResult::Unhandled
    }
    fn cmd_set_sample_locations_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_sample_locations_info: &vk::SampleLocationsInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_drm_format_modifier_properties_ext(
        &self,
        _image: vk::Image,
        _p_properties: &mut MaybeUninit<vk::ImageDrmFormatModifierPropertiesEXT>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_validation_cache_ext(
        &self,
        _p_create_info: &vk::ValidationCacheCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::ValidationCacheEXT>> {
        LayerResult::Unhandled
    }
    fn destroy_validation_cache_ext(
        &self,
        _validation_cache: vk::ValidationCacheEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn merge_validation_caches_ext(
        &self,
        _dst_cache: vk::ValidationCacheEXT,
        _p_src_caches: &[vk::ValidationCacheEXT],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_validation_cache_data_ext(
        &self,
        _validation_cache: vk::ValidationCacheEXT,
        _p_data_size: &mut MaybeUninit<usize>,
        _p_data: Option<&mut [MaybeUninit<u8>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_bind_shading_rate_image_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _image_view: vk::ImageView,
        _image_layout: vk::ImageLayout,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_viewport_shading_rate_palette_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_viewport: u32,
        _p_shading_rate_palettes: &[vk::ShadingRatePaletteNV],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coarse_sample_order_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _sample_order_type: vk::CoarseSampleOrderTypeNV,
        _p_custom_sample_orders: &[vk::CoarseSampleOrderCustomNV],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_acceleration_structure_nv(
        &self,
        _p_create_info: &vk::AccelerationStructureCreateInfoNV,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::AccelerationStructureNV>> {
        LayerResult::Unhandled
    }
    fn destroy_acceleration_structure_nv(
        &self,
        _acceleration_structure: vk::AccelerationStructureNV,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_acceleration_structure_memory_requirements_nv(
        &self,
        _p_info: &vk::AccelerationStructureMemoryRequirementsInfoNV,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements2KHR>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn bind_acceleration_structure_memory_nv(
        &self,
        _p_bind_infos: &[vk::BindAccelerationStructureMemoryInfoNV],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_build_acceleration_structure_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::AccelerationStructureInfoNV,
        _instance_data: vk::Buffer,
        _instance_offset: vk::DeviceSize,
        _update: bool,
        _dst: vk::AccelerationStructureNV,
        _src: vk::AccelerationStructureNV,
        _scratch: vk::Buffer,
        _scratch_offset: vk::DeviceSize,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_acceleration_structure_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _dst: vk::AccelerationStructureNV,
        _src: vk::AccelerationStructureNV,
        _mode: vk::CopyAccelerationStructureModeKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_trace_rays_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _raygen_shader_binding_table_buffer: vk::Buffer,
        _raygen_shader_binding_offset: vk::DeviceSize,
        _miss_shader_binding_table_buffer: vk::Buffer,
        _miss_shader_binding_offset: vk::DeviceSize,
        _miss_shader_binding_stride: vk::DeviceSize,
        _hit_shader_binding_table_buffer: vk::Buffer,
        _hit_shader_binding_offset: vk::DeviceSize,
        _hit_shader_binding_stride: vk::DeviceSize,
        _callable_shader_binding_table_buffer: vk::Buffer,
        _callable_shader_binding_offset: vk::DeviceSize,
        _callable_shader_binding_stride: vk::DeviceSize,
        _width: u32,
        _height: u32,
        _depth: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_ray_tracing_pipelines_nv(
        &self,
        _pipeline_cache: vk::PipelineCache,
        _p_create_infos: &[vk::RayTracingPipelineCreateInfoNV],
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<Vec<vk::Pipeline>>> {
        LayerResult::Unhandled
    }
    fn get_ray_tracing_shader_group_handles_khr(
        &self,
        _pipeline: vk::Pipeline,
        _first_group: u32,
        _group_count: u32,
        _p_data: &mut [MaybeUninit<u8>],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_acceleration_structure_handle_nv(
        &self,
        _acceleration_structure: vk::AccelerationStructureNV,
        _p_data: &mut [MaybeUninit<u8>],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_write_acceleration_structures_properties_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_acceleration_structures: &[vk::AccelerationStructureNV],
        _query_type: vk::QueryType,
        _query_pool: vk::QueryPool,
        _first_query: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn compile_deferred_nv(
        &self,
        _pipeline: vk::Pipeline,
        _shader: u32,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_memory_host_pointer_properties_ext(
        &self,
        _handle_type: vk::ExternalMemoryHandleTypeFlags,
        _p_host_pointer: *const c_void,
        _p_memory_host_pointer_properties: &mut MaybeUninit<vk::MemoryHostPointerPropertiesEXT>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_write_buffer_marker_amd(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_stage: vk::PipelineStageFlags,
        _dst_buffer: vk::Buffer,
        _dst_offset: vk::DeviceSize,
        _marker: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_calibrated_timestamps_ext(
        &self,
        _p_timestamp_infos: &[vk::CalibratedTimestampInfoEXT],
        _p_timestamps: &mut [MaybeUninit<u64>],
    ) -> LayerResult<VkResult<u64>> {
        LayerResult::Unhandled
    }
    fn cmd_draw_mesh_tasks_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _task_count: u32,
        _first_task: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_mesh_tasks_indirect_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_mesh_tasks_indirect_count_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _count_buffer: vk::Buffer,
        _count_buffer_offset: vk::DeviceSize,
        _max_draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_exclusive_scissor_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_exclusive_scissor: u32,
        _p_exclusive_scissors: &[vk::Rect2D],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_checkpoint_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_checkpoint_marker: *const c_void,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_queue_checkpoint_data_nv(
        &self,
        _queue: vk::Queue,
        _p_checkpoint_data_count: &mut MaybeUninit<u32>,
        _p_checkpoint_data: Option<&mut [MaybeUninit<vk::CheckpointDataNV>]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn initialize_performance_api_intel(
        &self,
        _p_initialize_info: &vk::InitializePerformanceApiInfoINTEL,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn uninitialize_performance_api_intel(&self) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_performance_marker_intel(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_marker_info: &vk::PerformanceMarkerInfoINTEL,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_set_performance_stream_marker_intel(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_marker_info: &vk::PerformanceStreamMarkerInfoINTEL,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_set_performance_override_intel(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_override_info: &vk::PerformanceOverrideInfoINTEL,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn acquire_performance_configuration_intel(
        &self,
        _p_acquire_info: &vk::PerformanceConfigurationAcquireInfoINTEL,
    ) -> LayerResult<VkResult<vk::PerformanceConfigurationINTEL>> {
        LayerResult::Unhandled
    }
    fn release_performance_configuration_intel(
        &self,
        _configuration: vk::PerformanceConfigurationINTEL,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn queue_set_performance_configuration_intel(
        &self,
        _queue: vk::Queue,
        _configuration: vk::PerformanceConfigurationINTEL,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_performance_parameter_intel(
        &self,
        _parameter: vk::PerformanceParameterTypeINTEL,
        _p_value: &mut MaybeUninit<vk::PerformanceValueINTEL>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn set_local_dimming_amd(
        &self,
        _swap_chain: vk::SwapchainKHR,
        _local_dimming_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn acquire_full_screen_exclusive_mode_ext(
        &self,
        _swapchain: vk::SwapchainKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn release_full_screen_exclusive_mode_ext(
        &self,
        _swapchain: vk::SwapchainKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_device_group_surface_present_modes2_ext(
        &self,
        _p_surface_info: &vk::PhysicalDeviceSurfaceInfo2KHR,
    ) -> LayerResult<VkResult<vk::DeviceGroupPresentModeFlagsKHR>> {
        LayerResult::Unhandled
    }
    fn cmd_set_line_stipple_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _line_stipple_factor: u32,
        _line_stipple_pattern: u16,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn release_swapchain_images_ext(
        &self,
        _p_release_info: &vk::ReleaseSwapchainImagesInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_generated_commands_memory_requirements_nv(
        &self,
        _p_info: &vk::GeneratedCommandsMemoryRequirementsInfoNV,
        _p_memory_requirements: &mut MaybeUninit<vk::MemoryRequirements2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_preprocess_generated_commands_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_generated_commands_info: &vk::GeneratedCommandsInfoNV,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_execute_generated_commands_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _is_preprocessed: bool,
        _p_generated_commands_info: &vk::GeneratedCommandsInfoNV,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_pipeline_shader_group_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_bind_point: vk::PipelineBindPoint,
        _pipeline: vk::Pipeline,
        _group_index: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_indirect_commands_layout_nv(
        &self,
        _p_create_info: &vk::IndirectCommandsLayoutCreateInfoNV,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::IndirectCommandsLayoutNV>> {
        LayerResult::Unhandled
    }
    fn destroy_indirect_commands_layout_nv(
        &self,
        _indirect_commands_layout: vk::IndirectCommandsLayoutNV,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn export_metal_objects_ext(
        &self,
        _p_metal_objects_info: &mut MaybeUninit<vk::ExportMetalObjectsInfoEXT>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_descriptor_set_layout_size_ext(
        &self,
        _layout: vk::DescriptorSetLayout,
    ) -> LayerResult<vk::DeviceSize> {
        LayerResult::Unhandled
    }
    fn get_descriptor_set_layout_binding_offset_ext(
        &self,
        _layout: vk::DescriptorSetLayout,
        _binding: u32,
    ) -> LayerResult<vk::DeviceSize> {
        LayerResult::Unhandled
    }
    fn get_descriptor_ext(
        &self,
        _p_descriptor_info: &vk::DescriptorGetInfoEXT,
        _p_descriptor: &mut [MaybeUninit<u8>],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_descriptor_buffers_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_binding_infos: &[vk::DescriptorBufferBindingInfoEXT],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_descriptor_buffer_offsets_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_bind_point: vk::PipelineBindPoint,
        _layout: vk::PipelineLayout,
        _first_set: u32,
        _p_buffer_indices: &[u32],
        _p_offsets: &[vk::DeviceSize],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_descriptor_buffer_embedded_samplers_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_bind_point: vk::PipelineBindPoint,
        _layout: vk::PipelineLayout,
        _set: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_buffer_opaque_capture_descriptor_data_ext(
        &self,
        _p_info: &vk::BufferCaptureDescriptorDataInfoEXT,
        _p_data: *mut c_void,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_image_opaque_capture_descriptor_data_ext(
        &self,
        _p_info: &vk::ImageCaptureDescriptorDataInfoEXT,
        _p_data: *mut c_void,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_image_view_opaque_capture_descriptor_data_ext(
        &self,
        _p_info: &vk::ImageViewCaptureDescriptorDataInfoEXT,
        _p_data: *mut c_void,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_sampler_opaque_capture_descriptor_data_ext(
        &self,
        _p_info: &vk::SamplerCaptureDescriptorDataInfoEXT,
        _p_data: *mut c_void,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_acceleration_structure_opaque_capture_descriptor_data_ext(
        &self,
        _p_info: &vk::AccelerationStructureCaptureDescriptorDataInfoEXT,
        _p_data: *mut c_void,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_set_fragment_shading_rate_enum_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _shading_rate: vk::FragmentShadingRateNV,
        _combiner_ops: &[vk::FragmentShadingRateCombinerOpKHR; 2],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_image_subresource_layout2_ext(
        &self,
        _image: vk::Image,
        _p_subresource: &vk::ImageSubresource2EXT,
        _p_layout: &mut MaybeUninit<vk::SubresourceLayout2EXT>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_vertex_input_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_vertex_binding_descriptions: &[vk::VertexInputBindingDescription2EXT],
        _p_vertex_attribute_descriptions: &[vk::VertexInputAttributeDescription2EXT],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_memory_zircon_handle_fuchsia(
        &self,
        _p_get_zircon_handle_info: &vk::MemoryGetZirconHandleInfoFUCHSIA,
    ) -> LayerResult<VkResult<vk::zx_handle_t>> {
        LayerResult::Unhandled
    }
    fn get_memory_zircon_handle_properties_fuchsia(
        &self,
        _handle_type: vk::ExternalMemoryHandleTypeFlags,
        _zircon_handle: vk::zx_handle_t,
        _p_memory_zircon_handle_properties: &mut MaybeUninit<
            vk::MemoryZirconHandlePropertiesFUCHSIA,
        >,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn import_semaphore_zircon_handle_fuchsia(
        &self,
        _p_import_semaphore_zircon_handle_info: &vk::ImportSemaphoreZirconHandleInfoFUCHSIA,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_semaphore_zircon_handle_fuchsia(
        &self,
        _p_get_zircon_handle_info: &vk::SemaphoreGetZirconHandleInfoFUCHSIA,
    ) -> LayerResult<VkResult<vk::zx_handle_t>> {
        LayerResult::Unhandled
    }
    fn create_buffer_collection_fuchsia(
        &self,
        _p_create_info: &vk::BufferCollectionCreateInfoFUCHSIA,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::BufferCollectionFUCHSIA>> {
        LayerResult::Unhandled
    }
    fn set_buffer_collection_image_constraints_fuchsia(
        &self,
        _collection: vk::BufferCollectionFUCHSIA,
        _p_image_constraints_info: &vk::ImageConstraintsInfoFUCHSIA,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn set_buffer_collection_buffer_constraints_fuchsia(
        &self,
        _collection: vk::BufferCollectionFUCHSIA,
        _p_buffer_constraints_info: &vk::BufferConstraintsInfoFUCHSIA,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn destroy_buffer_collection_fuchsia(
        &self,
        _collection: vk::BufferCollectionFUCHSIA,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_buffer_collection_properties_fuchsia(
        &self,
        _collection: vk::BufferCollectionFUCHSIA,
        _p_properties: &mut MaybeUninit<vk::BufferCollectionPropertiesFUCHSIA>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_device_subpass_shading_max_workgroup_size_huawei(
        &self,
        _renderpass: vk::RenderPass,
        _p_max_workgroup_size: &mut MaybeUninit<vk::Extent2D>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_subpass_shading_huawei(&self, _command_buffer: vk::CommandBuffer) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_bind_invocation_mask_huawei(
        &self,
        _command_buffer: vk::CommandBuffer,
        _image_view: vk::ImageView,
        _image_layout: vk::ImageLayout,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_memory_remote_address_nv(
        &self,
        _p_memory_get_remote_address_info: &vk::MemoryGetRemoteAddressInfoNV,
    ) -> LayerResult<VkResult<vk::RemoteAddressNV>> {
        LayerResult::Unhandled
    }
    fn get_pipeline_properties_ext(
        &self,
        _p_pipeline_info: &vk::PipelineInfoEXT,
        _p_pipeline_properties: &mut MaybeUninit<vk::BaseOutStructure>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_set_patch_control_points_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _patch_control_points: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_logic_op_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _logic_op: vk::LogicOp,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_color_write_enable_ext<T: Iterator<Item = bool> + 'static>(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_color_write_enables: T,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_multi_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_vertex_info: &[vk::MultiDrawInfoEXT],
        _instance_count: u32,
        _first_instance: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_multi_indexed_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_index_info: &[vk::MultiDrawIndexedInfoEXT],
        _instance_count: u32,
        _first_instance: u32,
        _stride: u32,
        _p_vertex_offset: Option<&i32>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_micromap_ext(
        &self,
        _p_create_info: &vk::MicromapCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::MicromapEXT>> {
        LayerResult::Unhandled
    }
    fn destroy_micromap_ext(
        &self,
        _micromap: vk::MicromapEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_build_micromaps_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_infos: &[vk::MicromapBuildInfoEXT],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn build_micromaps_ext(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_infos: &[vk::MicromapBuildInfoEXT],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn copy_micromap_ext(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_info: &vk::CopyMicromapInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn copy_micromap_to_memory_ext(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_info: &vk::CopyMicromapToMemoryInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn copy_memory_to_micromap_ext(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_info: &vk::CopyMemoryToMicromapInfoEXT,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn write_micromaps_properties_ext(
        &self,
        _p_micromaps: &[vk::MicromapEXT],
        _query_type: vk::QueryType,
        _p_data: &mut [MaybeUninit<u8>],
        _stride: usize,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_copy_micromap_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::CopyMicromapInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_micromap_to_memory_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::CopyMicromapToMemoryInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_memory_to_micromap_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::CopyMemoryToMicromapInfoEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_write_micromaps_properties_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_micromaps: &[vk::MicromapEXT],
        _query_type: vk::QueryType,
        _query_pool: vk::QueryPool,
        _first_query: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_device_micromap_compatibility_ext(
        &self,
        _p_version_info: &vk::MicromapVersionInfoEXT,
    ) -> LayerResult<vk::AccelerationStructureCompatibilityKHR> {
        LayerResult::Unhandled
    }
    fn get_micromap_build_sizes_ext(
        &self,
        _build_type: vk::AccelerationStructureBuildTypeKHR,
        _p_build_info: &vk::MicromapBuildInfoEXT,
        _p_size_info: &mut MaybeUninit<vk::MicromapBuildSizesInfoEXT>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn set_device_memory_priority_ext(
        &self,
        _memory: vk::DeviceMemory,
        _priority: f32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_descriptor_set_layout_host_mapping_info_valve(
        &self,
        _p_binding_reference: &vk::DescriptorSetBindingReferenceVALVE,
        _p_host_mapping: &mut MaybeUninit<vk::DescriptorSetLayoutHostMappingInfoVALVE>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_descriptor_set_host_mapping_valve(
        &self,
        _descriptor_set: vk::DescriptorSet,
    ) -> LayerResult<*mut c_void> {
        LayerResult::Unhandled
    }
    fn cmd_copy_memory_indirect_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _copy_buffer_address: vk::DeviceAddress,
        _copy_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_memory_to_image_indirect_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _copy_buffer_address: vk::DeviceAddress,
        _stride: u32,
        _dst_image: vk::Image,
        _dst_image_layout: vk::ImageLayout,
        _p_image_subresources: &[vk::ImageSubresourceLayers],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_decompress_memory_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_decompress_memory_regions: &[vk::DecompressMemoryRegionNV],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_decompress_memory_indirect_count_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _indirect_commands_address: vk::DeviceAddress,
        _indirect_commands_count_address: vk::DeviceAddress,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_tessellation_domain_origin_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _domain_origin: vk::TessellationDomainOrigin,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_clamp_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_clamp_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_polygon_mode_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _polygon_mode: vk::PolygonMode,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_rasterization_samples_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _rasterization_samples: vk::SampleCountFlags,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_sample_mask_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _samples: vk::SampleCountFlags,
        _p_sample_mask: &[vk::SampleMask],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_alpha_to_coverage_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _alpha_to_coverage_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_alpha_to_one_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _alpha_to_one_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_logic_op_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _logic_op_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_color_blend_enable_ext<T: Iterator<Item = bool> + 'static>(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_attachment: u32,
        _p_color_blend_enables: T,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_color_blend_equation_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_attachment: u32,
        _p_color_blend_equations: &[vk::ColorBlendEquationEXT],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_color_write_mask_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_attachment: u32,
        _p_color_write_masks: &[vk::ColorComponentFlags],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_rasterization_stream_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _rasterization_stream: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_conservative_rasterization_mode_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _conservative_rasterization_mode: vk::ConservativeRasterizationModeEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_extra_primitive_overestimation_size_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _extra_primitive_overestimation_size: f32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_clip_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _depth_clip_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_sample_locations_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _sample_locations_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_color_blend_advanced_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_attachment: u32,
        _p_color_blend_advanced: &[vk::ColorBlendAdvancedEXT],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_provoking_vertex_mode_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _provoking_vertex_mode: vk::ProvokingVertexModeEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_line_rasterization_mode_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _line_rasterization_mode: vk::LineRasterizationModeEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_line_stipple_enable_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _stippled_line_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_depth_clip_negative_one_to_one_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _negative_one_to_one: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_viewport_w_scaling_enable_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _viewport_w_scaling_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_viewport_swizzle_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _first_viewport: u32,
        _p_viewport_swizzles: &[vk::ViewportSwizzleNV],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coverage_to_color_enable_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _coverage_to_color_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coverage_to_color_location_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _coverage_to_color_location: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coverage_modulation_mode_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _coverage_modulation_mode: vk::CoverageModulationModeNV,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coverage_modulation_table_enable_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _coverage_modulation_table_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coverage_modulation_table_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_coverage_modulation_table: &[f32],
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_shading_rate_image_enable_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _shading_rate_image_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_representative_fragment_test_enable_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _representative_fragment_test_enable: bool,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_set_coverage_reduction_mode_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _coverage_reduction_mode: vk::CoverageReductionModeNV,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_shader_module_identifier_ext(
        &self,
        _shader_module: vk::ShaderModule,
        _p_identifier: &mut MaybeUninit<vk::ShaderModuleIdentifierEXT>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_shader_module_create_info_identifier_ext(
        &self,
        _p_create_info: &vk::ShaderModuleCreateInfo,
        _p_identifier: &mut MaybeUninit<vk::ShaderModuleIdentifierEXT>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_optical_flow_session_nv(
        &self,
        _p_create_info: &vk::OpticalFlowSessionCreateInfoNV,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::OpticalFlowSessionNV>> {
        LayerResult::Unhandled
    }
    fn destroy_optical_flow_session_nv(
        &self,
        _session: vk::OpticalFlowSessionNV,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn bind_optical_flow_session_image_nv(
        &self,
        _session: vk::OpticalFlowSessionNV,
        _binding_point: vk::OpticalFlowSessionBindingPointNV,
        _view: vk::ImageView,
        _layout: vk::ImageLayout,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_optical_flow_execute_nv(
        &self,
        _command_buffer: vk::CommandBuffer,
        _session: vk::OpticalFlowSessionNV,
        _p_execute_info: &vk::OpticalFlowExecuteInfoNV,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_framebuffer_tile_properties_qcom(
        &self,
        _framebuffer: vk::Framebuffer,
        _p_properties_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::TilePropertiesQCOM>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_dynamic_rendering_tile_properties_qcom(
        &self,
        _p_rendering_info: &vk::RenderingInfo,
        _p_properties: &mut MaybeUninit<vk::TilePropertiesQCOM>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_acceleration_structure_khr(
        &self,
        _p_create_info: &vk::AccelerationStructureCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::AccelerationStructureKHR>> {
        LayerResult::Unhandled
    }
    fn destroy_acceleration_structure_khr(
        &self,
        _acceleration_structure: vk::AccelerationStructureKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn copy_acceleration_structure_khr(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_info: &vk::CopyAccelerationStructureInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn copy_acceleration_structure_to_memory_khr(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_info: &vk::CopyAccelerationStructureToMemoryInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn copy_memory_to_acceleration_structure_khr(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _p_info: &vk::CopyMemoryToAccelerationStructureInfoKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn write_acceleration_structures_properties_khr(
        &self,
        _p_acceleration_structures: &[vk::AccelerationStructureKHR],
        _query_type: vk::QueryType,
        _p_data: &mut [MaybeUninit<u8>],
        _stride: usize,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_copy_acceleration_structure_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::CopyAccelerationStructureInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_acceleration_structure_to_memory_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::CopyAccelerationStructureToMemoryInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_copy_memory_to_acceleration_structure_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_info: &vk::CopyMemoryToAccelerationStructureInfoKHR,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_acceleration_structure_device_address_khr(
        &self,
        _p_info: &vk::AccelerationStructureDeviceAddressInfoKHR,
    ) -> LayerResult<vk::DeviceAddress> {
        LayerResult::Unhandled
    }
    fn cmd_write_acceleration_structures_properties_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_acceleration_structures: &[vk::AccelerationStructureKHR],
        _query_type: vk::QueryType,
        _query_pool: vk::QueryPool,
        _first_query: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_device_acceleration_structure_compatibility_khr(
        &self,
        _p_version_info: &vk::AccelerationStructureVersionInfoKHR,
    ) -> LayerResult<vk::AccelerationStructureCompatibilityKHR> {
        LayerResult::Unhandled
    }
    fn get_acceleration_structure_build_sizes_khr(
        &self,
        _build_type: vk::AccelerationStructureBuildTypeKHR,
        _p_build_info: &vk::AccelerationStructureBuildGeometryInfoKHR,
        _p_max_primitive_counts: Option<&[u32]>,
        _p_size_info: &mut MaybeUninit<vk::AccelerationStructureBuildSizesInfoKHR>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_trace_rays_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_raygen_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _p_miss_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _p_hit_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _p_callable_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _width: u32,
        _height: u32,
        _depth: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_ray_tracing_pipelines_khr(
        &self,
        _deferred_operation: vk::DeferredOperationKHR,
        _pipeline_cache: vk::PipelineCache,
        _p_create_infos: &[vk::RayTracingPipelineCreateInfoKHR],
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<Vec<vk::Pipeline>>> {
        LayerResult::Unhandled
    }
    fn get_ray_tracing_capture_replay_shader_group_handles_khr(
        &self,
        _pipeline: vk::Pipeline,
        _first_group: u32,
        _group_count: u32,
        _p_data: &mut [MaybeUninit<u8>],
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn cmd_trace_rays_indirect_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _p_raygen_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _p_miss_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _p_hit_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _p_callable_shader_binding_table: &vk::StridedDeviceAddressRegionKHR,
        _indirect_device_address: vk::DeviceAddress,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_ray_tracing_shader_group_stack_size_khr(
        &self,
        _pipeline: vk::Pipeline,
        _group: u32,
        _group_shader: vk::ShaderGroupShaderKHR,
    ) -> LayerResult<vk::DeviceSize> {
        LayerResult::Unhandled
    }
    fn cmd_set_ray_tracing_pipeline_stack_size_khr(
        &self,
        _command_buffer: vk::CommandBuffer,
        _pipeline_stack_size: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_mesh_tasks_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _group_countx: u32,
        _group_county: u32,
        _group_countz: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_mesh_tasks_indirect_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn cmd_draw_mesh_tasks_indirect_count_ext(
        &self,
        _command_buffer: vk::CommandBuffer,
        _buffer: vk::Buffer,
        _offset: vk::DeviceSize,
        _count_buffer: vk::Buffer,
        _count_buffer_offset: vk::DeviceSize,
        _max_draw_count: u32,
        _stride: u32,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
}

pub trait InstanceHooks: Send + Sync {
    fn get_physical_device_features(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_features: &mut MaybeUninit<vk::PhysicalDeviceFeatures>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_format_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _format: vk::Format,
        _p_format_properties: &mut MaybeUninit<vk::FormatProperties>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_image_format_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _format: vk::Format,
        _type: vk::ImageType,
        _tiling: vk::ImageTiling,
        _usage: vk::ImageUsageFlags,
        _flags: vk::ImageCreateFlags,
        _p_image_format_properties: &mut MaybeUninit<vk::ImageFormatProperties>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_properties: &mut MaybeUninit<vk::PhysicalDeviceProperties>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_queue_family_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_queue_family_property_count: &mut MaybeUninit<u32>,
        _p_queue_family_properties: Option<&mut [MaybeUninit<vk::QueueFamilyProperties>]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_memory_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_memory_properties: &mut MaybeUninit<vk::PhysicalDeviceMemoryProperties>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_instance_proc_addr(&self, _p_name: &str) -> LayerResult<vk::PFN_vkVoidFunction> {
        LayerResult::Unhandled
    }
    fn create_device(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_create_info: &vk::DeviceCreateInfo,
        _layer_device_link: &VkLayerDeviceLink,
        _p_allocator: Option<&vk::AllocationCallbacks>,
        _p_device: &mut MaybeUninit<vk::Device>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_sparse_image_format_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _format: vk::Format,
        _type: vk::ImageType,
        _samples: vk::SampleCountFlags,
        _usage: vk::ImageUsageFlags,
        _tiling: vk::ImageTiling,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::SparseImageFormatProperties>]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_features2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_features: &mut MaybeUninit<vk::PhysicalDeviceFeatures2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_properties2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_properties: &mut MaybeUninit<vk::PhysicalDeviceProperties2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_format_properties2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _format: vk::Format,
        _p_format_properties: &mut MaybeUninit<vk::FormatProperties2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_image_format_properties2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_image_format_info: &vk::PhysicalDeviceImageFormatInfo2,
        _p_image_format_properties: &mut MaybeUninit<vk::ImageFormatProperties2>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_queue_family_properties2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_queue_family_property_count: &mut MaybeUninit<u32>,
        _p_queue_family_properties: Option<&mut [MaybeUninit<vk::QueueFamilyProperties2>]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_memory_properties2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_memory_properties: &mut MaybeUninit<vk::PhysicalDeviceMemoryProperties2>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_sparse_image_format_properties2(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_format_info: &vk::PhysicalDeviceSparseImageFormatInfo2,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::SparseImageFormatProperties2>]>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_external_buffer_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_external_buffer_info: &vk::PhysicalDeviceExternalBufferInfo,
        _p_external_buffer_properties: &mut MaybeUninit<vk::ExternalBufferProperties>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_external_fence_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_external_fence_info: &vk::PhysicalDeviceExternalFenceInfo,
        _p_external_fence_properties: &mut MaybeUninit<vk::ExternalFenceProperties>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_external_semaphore_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_external_semaphore_info: &vk::PhysicalDeviceExternalSemaphoreInfo,
        _p_external_semaphore_properties: &mut MaybeUninit<vk::ExternalSemaphoreProperties>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_tool_properties(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_tool_count: &mut MaybeUninit<u32>,
        _p_tool_properties: Option<&mut [MaybeUninit<vk::PhysicalDeviceToolProperties>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn destroy_surface_khr(
        &self,
        _surface: vk::SurfaceKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_support_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _surface: vk::SurfaceKHR,
    ) -> LayerResult<VkResult<bool>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_capabilities_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _surface: vk::SurfaceKHR,
        _p_surface_capabilities: &mut MaybeUninit<vk::SurfaceCapabilitiesKHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_formats_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _surface: vk::SurfaceKHR,
        _p_surface_format_count: &mut MaybeUninit<u32>,
        _p_surface_formats: Option<&mut [MaybeUninit<vk::SurfaceFormatKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_present_modes_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _surface: vk::SurfaceKHR,
    ) -> LayerResult<VkResult<Vec<vk::PresentModeKHR>>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_present_rectangles_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _surface: vk::SurfaceKHR,
        _p_rect_count: &mut MaybeUninit<u32>,
        _p_rects: Option<&mut [MaybeUninit<vk::Rect2D>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_display_properties_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::DisplayPropertiesKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_display_plane_properties_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::DisplayPlanePropertiesKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_display_plane_supported_displays_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _plane_index: u32,
    ) -> LayerResult<VkResult<Vec<vk::DisplayKHR>>> {
        LayerResult::Unhandled
    }
    fn get_display_mode_properties_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _display: vk::DisplayKHR,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::DisplayModePropertiesKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_display_mode_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _display: vk::DisplayKHR,
        _p_create_info: &vk::DisplayModeCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DisplayModeKHR>> {
        LayerResult::Unhandled
    }
    fn get_display_plane_capabilities_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _mode: vk::DisplayModeKHR,
        _plane_index: u32,
        _p_capabilities: &mut MaybeUninit<vk::DisplayPlaneCapabilitiesKHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_display_plane_surface_khr(
        &self,
        _p_create_info: &vk::DisplaySurfaceCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn create_xlib_surface_khr(
        &self,
        _p_create_info: &vk::XlibSurfaceCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_xlib_presentation_support_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _dpy: &mut MaybeUninit<vk::Display>,
        _visual_id: vk::VisualID,
    ) -> LayerResult<bool> {
        LayerResult::Unhandled
    }
    fn create_xcb_surface_khr(
        &self,
        _p_create_info: &vk::XcbSurfaceCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_xcb_presentation_support_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _connection: *mut vk::xcb_connection_t,
        _visual_id: vk::xcb_visualid_t,
    ) -> LayerResult<bool> {
        LayerResult::Unhandled
    }
    fn create_wayland_surface_khr(
        &self,
        _p_create_info: &vk::WaylandSurfaceCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_wayland_presentation_support_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _display: *mut vk::wl_display,
    ) -> LayerResult<bool> {
        LayerResult::Unhandled
    }
    fn create_android_surface_khr(
        &self,
        _p_create_info: &vk::AndroidSurfaceCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn create_win32_surface_khr(
        &self,
        _p_create_info: &vk::Win32SurfaceCreateInfoKHR,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_win32_presentation_support_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
    ) -> LayerResult<bool> {
        LayerResult::Unhandled
    }
    fn get_physical_device_video_capabilities_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_video_profile: &vk::VideoProfileInfoKHR,
        _p_capabilities: &mut MaybeUninit<vk::VideoCapabilitiesKHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_video_format_properties_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_video_format_info: &vk::PhysicalDeviceVideoFormatInfoKHR,
        _p_video_format_property_count: &mut MaybeUninit<u32>,
        _p_video_format_properties: Option<&mut [MaybeUninit<vk::VideoFormatPropertiesKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn enumerate_physical_device_queue_family_performance_query_counters_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _p_counter_count: &mut MaybeUninit<u32>,
        _p_counters: Option<&mut [MaybeUninit<vk::PerformanceCounterKHR>]>,
        _p_counter_descriptions: Option<&mut [MaybeUninit<vk::PerformanceCounterDescriptionKHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_queue_family_performance_query_passes_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_performance_query_create_info: &vk::QueryPoolPerformanceCreateInfoKHR,
    ) -> LayerResult<u32> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_capabilities2_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_surface_info: &vk::PhysicalDeviceSurfaceInfo2KHR,
        _p_surface_capabilities: &mut MaybeUninit<vk::SurfaceCapabilities2KHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_formats2_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_surface_info: &vk::PhysicalDeviceSurfaceInfo2KHR,
        _p_surface_format_count: &mut MaybeUninit<u32>,
        _p_surface_formats: Option<&mut [MaybeUninit<vk::SurfaceFormat2KHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_display_properties2_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::DisplayProperties2KHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_display_plane_properties2_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::DisplayPlaneProperties2KHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_display_mode_properties2_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _display: vk::DisplayKHR,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::DisplayModeProperties2KHR>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_display_plane_capabilities2_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_display_plane_info: &vk::DisplayPlaneInfo2KHR,
        _p_capabilities: &mut MaybeUninit<vk::DisplayPlaneCapabilities2KHR>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_fragment_shading_rates_khr(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_fragment_shading_rate_count: &mut MaybeUninit<u32>,
        _p_fragment_shading_rates: Option<
            &mut [MaybeUninit<vk::PhysicalDeviceFragmentShadingRateKHR>],
        >,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_debug_report_callback_ext(
        &self,
        _p_create_info: &vk::DebugReportCallbackCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DebugReportCallbackEXT>> {
        LayerResult::Unhandled
    }
    fn destroy_debug_report_callback_ext(
        &self,
        _callback: vk::DebugReportCallbackEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn debug_report_message_ext(
        &self,
        _flags: vk::DebugReportFlagsEXT,
        _object_type: vk::DebugReportObjectTypeEXT,
        _object: u64,
        _location: usize,
        _message_code: i32,
        _p_layer_prefix: &str,
        _p_message: &str,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn create_stream_descriptor_surface_ggp(
        &self,
        _p_create_info: &vk::StreamDescriptorSurfaceCreateInfoGGP,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_external_image_format_properties_nv(
        &self,
        _physical_device: vk::PhysicalDevice,
        _format: vk::Format,
        _type: vk::ImageType,
        _tiling: vk::ImageTiling,
        _usage: vk::ImageUsageFlags,
        _flags: vk::ImageCreateFlags,
        _external_handle_type: vk::ExternalMemoryHandleTypeFlagsNV,
        _p_external_image_format_properties: &mut MaybeUninit<vk::ExternalImageFormatPropertiesNV>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_vi_surface_nn(
        &self,
        _p_create_info: &vk::ViSurfaceCreateInfoNN,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn release_display_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _display: vk::DisplayKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn acquire_xlib_display_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _dpy: &mut MaybeUninit<vk::Display>,
        _display: vk::DisplayKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_rand_r_output_display_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _dpy: &mut MaybeUninit<vk::Display>,
        _rr_output: vk::RROutput,
    ) -> LayerResult<VkResult<vk::DisplayKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_capabilities2_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _surface: vk::SurfaceKHR,
        _p_surface_capabilities: &mut MaybeUninit<vk::SurfaceCapabilities2EXT>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn create_ios_surface_mvk(
        &self,
        _p_create_info: &vk::IOSSurfaceCreateInfoMVK,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn create_mac_os_surface_mvk(
        &self,
        _p_create_info: &vk::MacOSSurfaceCreateInfoMVK,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn create_debug_utils_messenger_ext(
        &self,
        _p_create_info: &vk::DebugUtilsMessengerCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::DebugUtilsMessengerEXT>> {
        LayerResult::Unhandled
    }
    fn destroy_debug_utils_messenger_ext(
        &self,
        _messenger: vk::DebugUtilsMessengerEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn submit_debug_utils_message_ext(
        &self,
        _message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
        _message_types: vk::DebugUtilsMessageTypeFlagsEXT,
        _p_callback_data: &vk::DebugUtilsMessengerCallbackDataEXT,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_multisample_properties_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _samples: vk::SampleCountFlags,
        _p_multisample_properties: &mut MaybeUninit<vk::MultisamplePropertiesEXT>,
    ) -> LayerResult<()> {
        LayerResult::Unhandled
    }
    fn get_physical_device_calibrateable_time_domains_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
    ) -> LayerResult<VkResult<Vec<vk::TimeDomainEXT>>> {
        LayerResult::Unhandled
    }
    fn create_image_pipe_surface_fuchsia(
        &self,
        _p_create_info: &vk::ImagePipeSurfaceCreateInfoFUCHSIA,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn create_metal_surface_ext(
        &self,
        _p_create_info: &vk::MetalSurfaceCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_cooperative_matrix_properties_nv(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_property_count: &mut MaybeUninit<u32>,
        _p_properties: Option<&mut [MaybeUninit<vk::CooperativeMatrixPropertiesNV>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_supported_framebuffer_mixed_samples_combinations_nv(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_combination_count: &mut MaybeUninit<u32>,
        _p_combinations: Option<&mut [MaybeUninit<vk::FramebufferMixedSamplesCombinationNV>]>,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_surface_present_modes2_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_surface_info: &vk::PhysicalDeviceSurfaceInfo2KHR,
    ) -> LayerResult<VkResult<Vec<vk::PresentModeKHR>>> {
        LayerResult::Unhandled
    }
    fn create_headless_surface_ext(
        &self,
        _p_create_info: &vk::HeadlessSurfaceCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn acquire_drm_display_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _drm_fd: i32,
        _display: vk::DisplayKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_drm_display_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _drm_fd: i32,
        _connector_id: u32,
    ) -> LayerResult<VkResult<vk::DisplayKHR>> {
        LayerResult::Unhandled
    }
    fn acquire_winrt_display_nv(
        &self,
        _physical_device: vk::PhysicalDevice,
        _display: vk::DisplayKHR,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
    fn get_winrt_display_nv(
        &self,
        _physical_device: vk::PhysicalDevice,
        _device_relative_id: u32,
    ) -> LayerResult<VkResult<vk::DisplayKHR>> {
        LayerResult::Unhandled
    }
    fn create_direct_fb_surface_ext(
        &self,
        _p_create_info: &vk::DirectFBSurfaceCreateInfoEXT,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_direct_fb_presentation_support_ext(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _dfb: &mut MaybeUninit<vk::IDirectFB>,
    ) -> LayerResult<bool> {
        LayerResult::Unhandled
    }
    fn create_screen_surface_qnx(
        &self,
        _p_create_info: &vk::ScreenSurfaceCreateInfoQNX,
        _p_allocator: Option<&vk::AllocationCallbacks>,
    ) -> LayerResult<VkResult<vk::SurfaceKHR>> {
        LayerResult::Unhandled
    }
    fn get_physical_device_screen_presentation_support_qnx(
        &self,
        _physical_device: vk::PhysicalDevice,
        _queue_family_index: u32,
        _window: *mut vk::_screen_window,
    ) -> LayerResult<bool> {
        LayerResult::Unhandled
    }
    fn get_physical_device_optical_flow_image_formats_nv(
        &self,
        _physical_device: vk::PhysicalDevice,
        _p_optical_flow_image_format_info: &vk::OpticalFlowImageFormatInfoNV,
        _p_format_count: &mut MaybeUninit<u32>,
        _p_image_format_properties: Option<
            &mut [MaybeUninit<vk::OpticalFlowImageFormatPropertiesNV>],
        >,
    ) -> LayerResult<VkResult<()>> {
        LayerResult::Unhandled
    }
}
