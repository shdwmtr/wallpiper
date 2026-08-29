/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2026 Ethan Alexander
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy
 * of this software and associated documentation files (the "Software"), to deal
 * in the Software without restriction, including without limitation the rights
 * to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
 * copies of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in
 * all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
 * FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
 * AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
 * LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
 * SOFTWARE.
 */

#pragma once

#include <pthread.h>
#include <stdbool.h>
#include <stdint.h>

#include <vulkan/vulkan.h>

#include "config.h"
#include "wallpiper/capture_socket.h"

#define WP_MAX_SWAPCHAINS 16
#define WP_MAX_QUEUES 16

#define WP_VK_EXPORT __attribute__((visibility("default")))

typedef enum {
  WP_SLOT_NEVER_WRITTEN,
  WP_SLOT_RELEASED_TO_FOREIGN,
} wp_slot_ownership_t;

typedef struct {
  bool in_use;
  VkImage image;
  VkDeviceMemory memory;
  VkCommandPool pool;
  VkCommandBuffer cmd;
  VkFence fence;
  bool fence_pending;
  VkSemaphore ready_semaphore;
  wp_slot_ownership_t ownership;
  bool buf_sent;
  uint32_t stride;
  uint64_t modifier;
} wp_capture_slot_t;

typedef struct {
  bool in_use;
  VkSwapchainKHR swapchain;
  VkFormat format;
  VkExtent2D extent;
  VkImage images[16];
  uint32_t image_count;
  uint64_t candidate_modifiers[32];
  uint32_t candidate_modifier_count;
  wp_capture_slot_t slots[WP_CAPTURE_SLOT_COUNT];
  size_t next_slot;
  uint64_t present_count;
  bool has_last_present_at;
  struct timespec last_present_at;
  uint32_t wire_channel;
} wp_swapchain_state_t;

typedef struct {
  void *dispatch_key;
  VkDevice device;
  VkPhysicalDevice physical_device;
  PFN_vkGetDeviceProcAddr next_gdpa;

  PFN_vkCreateSwapchainKHR real_create_swapchain_khr;
  PFN_vkDestroySwapchainKHR real_destroy_swapchain_khr;
  PFN_vkQueuePresentKHR real_queue_present_khr;
  PFN_vkGetMemoryFdKHR real_get_memory_fd_khr;
  PFN_vkGetSemaphoreFdKHR real_get_semaphore_fd_khr;
  PFN_vkGetSwapchainImagesKHR real_get_swapchain_images_khr;
  PFN_vkGetImageDrmFormatModifierPropertiesEXT
      real_get_image_drm_format_modifier_properties_ext;
  PFN_vkGetDeviceQueue real_get_device_queue;
  PFN_vkGetDeviceQueue2 real_get_device_queue2;

  PFN_vkCreateImage create_image;
  PFN_vkDestroyImage destroy_image;
  PFN_vkGetImageMemoryRequirements get_image_memory_requirements;
  PFN_vkAllocateMemory allocate_memory;
  PFN_vkFreeMemory free_memory;
  PFN_vkBindImageMemory bind_image_memory;
  PFN_vkGetImageSubresourceLayout get_image_subresource_layout;
  PFN_vkCreateCommandPool create_command_pool;
  PFN_vkDestroyCommandPool destroy_command_pool;
  PFN_vkAllocateCommandBuffers allocate_command_buffers;
  PFN_vkCreateFence create_fence;
  PFN_vkDestroyFence destroy_fence;
  PFN_vkWaitForFences wait_for_fences;
  PFN_vkGetFenceStatus get_fence_status;
  PFN_vkResetFences reset_fences;
  PFN_vkCreateSemaphore create_semaphore;
  PFN_vkDestroySemaphore destroy_semaphore;
  PFN_vkBeginCommandBuffer begin_command_buffer;
  PFN_vkEndCommandBuffer end_command_buffer;
  PFN_vkCmdPipelineBarrier cmd_pipeline_barrier;
  PFN_vkCmdCopyImage cmd_copy_image;
  PFN_vkResetCommandBuffer reset_command_buffer;
  PFN_vkQueueSubmit queue_submit;

  pthread_mutex_t swapchains_mutex;
  wp_swapchain_state_t swapchains[WP_MAX_SWAPCHAINS];

  pthread_mutex_t queue_family_mutex;
  VkQueue queue_family_queues[WP_MAX_QUEUES];
  uint32_t queue_family_indices[WP_MAX_QUEUES];
  size_t queue_family_count;

  wp_capture_link_t *capture_link;
} wp_device_data_t;

typedef struct {
  const char *name;
  PFN_vkVoidFunction fn;
} wp_fn_entry_t;

wp_device_data_t *wp_device_data_create(VkDevice device,
                                        VkPhysicalDevice physical_device,
                                        PFN_vkGetDeviceProcAddr next_gdpa);
wp_device_data_t *wp_device_data_find(void *dispatch_key);
void wp_device_data_remove(void *dispatch_key);

static inline void *wp_dispatch_key(const void *dispatchable_handle) {
  return *(void *const *)dispatchable_handle;
}

bool wp_instance_find_memory_type(wp_device_data_t *device_data,
                                  uint32_t type_bits,
                                  VkMemoryPropertyFlags flags,
                                  uint32_t *out_index);

void wp_global_instance_set(VkInstance instance,
                            PFN_vkGetInstanceProcAddr next_gipa);
bool wp_global_instance_get_memory_properties(
    VkPhysicalDevice pd, VkPhysicalDeviceMemoryProperties *out);
bool wp_global_instance_get_format_properties2(VkPhysicalDevice pd,
                                               VkFormat format,
                                               VkFormatProperties2 *out);

void wp_surface_xid_register(VkSurfaceKHR surface, unsigned long xid);
bool wp_surface_xid_lookup(VkSurfaceKHR surface, unsigned long *out_xid);

VKAPI_ATTR VkResult VKAPI_CALL wp_CreateInstance(
    const VkInstanceCreateInfo *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkInstance *pInstance);
VKAPI_ATTR void VKAPI_CALL wp_DestroyInstance(
    VkInstance instance, const VkAllocationCallbacks *pAllocator);
VKAPI_ATTR VkResult VKAPI_CALL wp_CreateDevice(
    VkPhysicalDevice physicalDevice, const VkDeviceCreateInfo *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkDevice *pDevice);
VKAPI_ATTR void VKAPI_CALL
wp_DestroyDevice(VkDevice device, const VkAllocationCallbacks *pAllocator);

VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL
vkGetInstanceProcAddr(VkInstance instance, const char *pName);
VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL vkGetDeviceProcAddr(VkDevice device,
                                                             const char *pName);

void wp_register_swapchain(wp_device_data_t *device_data,
                           VkSwapchainKHR swapchain,
                           const VkSwapchainCreateInfoKHR *create_info);
void wp_teardown_swapchain(wp_device_data_t *device_data,
                           VkSwapchainKHR swapchain);
void wp_capture_and_notify(wp_device_data_t *device_data, VkQueue queue,
                           VkSwapchainKHR swapchain, uint32_t image_index);

bool wp_queue_family_for(wp_device_data_t *device_data, VkQueue queue,
                         uint32_t *out_family);
void wp_queue_family_set(wp_device_data_t *device_data, VkQueue queue,
                         uint32_t family);
