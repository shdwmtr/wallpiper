#include "wp_layer.h"

#include "logging.h"
#include "process.h"

#include <stdlib.h>
#include <string.h>

#define WP_MAX_DEVICES 4

static pthread_mutex_t g_device_table_mutex = PTHREAD_MUTEX_INITIALIZER;
static wp_device_data_t *g_device_table[WP_MAX_DEVICES];
static size_t g_device_table_count = 0;

typedef struct {
  const char *name;
  void **slot;
} wp_pfn_entry_t;

static void load_device_procs(wp_device_data_t *data,
                              PFN_vkGetDeviceProcAddr gdpa, VkDevice device) {
  wp_pfn_entry_t table[] = {
      {"vkCreateSwapchainKHR", (void **)&data->real_create_swapchain_khr},
      {"vkDestroySwapchainKHR", (void **)&data->real_destroy_swapchain_khr},
      {"vkQueuePresentKHR", (void **)&data->real_queue_present_khr},
      {"vkGetMemoryFdKHR", (void **)&data->real_get_memory_fd_khr},
      {"vkGetSemaphoreFdKHR", (void **)&data->real_get_semaphore_fd_khr},
      {"vkGetSwapchainImagesKHR",
       (void **)&data->real_get_swapchain_images_khr},
      {"vkGetImageDrmFormatModifierPropertiesEXT",
       (void **)&data->real_get_image_drm_format_modifier_properties_ext},
      {"vkGetDeviceQueue", (void **)&data->real_get_device_queue},
      {"vkGetDeviceQueue2", (void **)&data->real_get_device_queue2},
      {"vkCreateImage", (void **)&data->create_image},
      {"vkDestroyImage", (void **)&data->destroy_image},
      {"vkGetImageMemoryRequirements",
       (void **)&data->get_image_memory_requirements},
      {"vkAllocateMemory", (void **)&data->allocate_memory},
      {"vkFreeMemory", (void **)&data->free_memory},
      {"vkBindImageMemory", (void **)&data->bind_image_memory},
      {"vkGetImageSubresourceLayout",
       (void **)&data->get_image_subresource_layout},
      {"vkCreateCommandPool", (void **)&data->create_command_pool},
      {"vkDestroyCommandPool", (void **)&data->destroy_command_pool},
      {"vkAllocateCommandBuffers", (void **)&data->allocate_command_buffers},
      {"vkCreateFence", (void **)&data->create_fence},
      {"vkDestroyFence", (void **)&data->destroy_fence},
      {"vkWaitForFences", (void **)&data->wait_for_fences},
      {"vkGetFenceStatus", (void **)&data->get_fence_status},
      {"vkResetFences", (void **)&data->reset_fences},
      {"vkCreateSemaphore", (void **)&data->create_semaphore},
      {"vkDestroySemaphore", (void **)&data->destroy_semaphore},
      {"vkBeginCommandBuffer", (void **)&data->begin_command_buffer},
      {"vkEndCommandBuffer", (void **)&data->end_command_buffer},
      {"vkCmdPipelineBarrier", (void **)&data->cmd_pipeline_barrier},
      {"vkCmdCopyImage", (void **)&data->cmd_copy_image},
      {"vkResetCommandBuffer", (void **)&data->reset_command_buffer},
      {"vkQueueSubmit", (void **)&data->queue_submit},
  };
  for (size_t i = 0; i < sizeof(table) / sizeof(table[0]); i++) {
    *table[i].slot = (void *)gdpa(device, table[i].name);
  }
}

wp_device_data_t *wp_device_data_create(VkDevice device,
                                        VkPhysicalDevice physical_device,
                                        PFN_vkGetDeviceProcAddr next_gdpa) {
  wp_device_data_t *data = calloc(1, sizeof(wp_device_data_t));
  if (!data) {
    return NULL;
  }

  if (wp_capture_is_target_process()) {
    WP_LOG("create_device_info: resolving device function pointers");
  }

  data->dispatch_key = wp_dispatch_key(device);
  data->device = device;
  data->physical_device = physical_device;
  data->next_gdpa = next_gdpa;
  load_device_procs(data, next_gdpa, device);

  if (wp_capture_is_target_process()) {
    WP_LOG("create_device_info: device function pointers resolved");
  }

  pthread_mutex_init(&data->swapchains_mutex, NULL);
  pthread_mutex_init(&data->queue_family_mutex, NULL);
  data->capture_link = wp_capture_link_create();

  pthread_mutex_lock(&g_device_table_mutex);
  bool inserted = false;
  if (g_device_table_count < WP_MAX_DEVICES) {
    g_device_table[g_device_table_count++] = data;
    inserted = true;
  }
  pthread_mutex_unlock(&g_device_table_mutex);

  if (!inserted) {
    pthread_mutex_destroy(&data->swapchains_mutex);
    pthread_mutex_destroy(&data->queue_family_mutex);
    wp_capture_link_destroy(data->capture_link);
    free(data);
    return NULL;
  }

  return data;
}

wp_device_data_t *wp_device_data_find(void *dispatch_key) {
  wp_device_data_t *result = NULL;
  pthread_mutex_lock(&g_device_table_mutex);
  for (size_t i = 0; i < g_device_table_count; i++) {
    if (g_device_table[i]->dispatch_key == dispatch_key) {
      result = g_device_table[i];
      break;
    }
  }
  pthread_mutex_unlock(&g_device_table_mutex);
  return result;
}

void wp_device_data_remove(void *dispatch_key) {
  pthread_mutex_lock(&g_device_table_mutex);
  for (size_t i = 0; i < g_device_table_count; i++) {
    if (g_device_table[i]->dispatch_key == dispatch_key) {
      wp_device_data_t *data = g_device_table[i];
      g_device_table[i] = g_device_table[g_device_table_count - 1];
      g_device_table_count--;
      pthread_mutex_unlock(&g_device_table_mutex);

      pthread_mutex_destroy(&data->swapchains_mutex);
      pthread_mutex_destroy(&data->queue_family_mutex);
      wp_capture_link_destroy(data->capture_link);
      free(data);
      return;
    }
  }
  pthread_mutex_unlock(&g_device_table_mutex);
}

bool wp_queue_family_for(wp_device_data_t *device_data, VkQueue queue,
                         uint32_t *out_family) {
  bool found = false;
  pthread_mutex_lock(&device_data->queue_family_mutex);
  for (size_t i = 0; i < device_data->queue_family_count; i++) {
    if (device_data->queue_family_queues[i] == queue) {
      *out_family = device_data->queue_family_indices[i];
      found = true;
      break;
    }
  }
  pthread_mutex_unlock(&device_data->queue_family_mutex);
  return found;
}

void wp_queue_family_set(wp_device_data_t *device_data, VkQueue queue,
                         uint32_t family) {
  pthread_mutex_lock(&device_data->queue_family_mutex);
  bool updated = false;
  for (size_t i = 0; i < device_data->queue_family_count; i++) {
    if (device_data->queue_family_queues[i] == queue) {
      device_data->queue_family_indices[i] = family;
      updated = true;
      break;
    }
  }
  if (!updated && device_data->queue_family_count < WP_MAX_QUEUES) {
    device_data->queue_family_queues[device_data->queue_family_count] = queue;
    device_data->queue_family_indices[device_data->queue_family_count] = family;
    device_data->queue_family_count++;
  }
  pthread_mutex_unlock(&device_data->queue_family_mutex);
}

bool wp_instance_find_memory_type(wp_device_data_t *device_data,
                                  uint32_t type_bits,
                                  VkMemoryPropertyFlags flags,
                                  uint32_t *out_index) {
  VkPhysicalDeviceMemoryProperties props;
  if (!wp_global_instance_get_memory_properties(device_data->physical_device,
                                                &props)) {
    return false;
  }
  for (uint32_t i = 0; i < props.memoryTypeCount; i++) {
    bool bit_set = (type_bits & (1u << i)) != 0;
    bool has_flags = (props.memoryTypes[i].propertyFlags & flags) == flags;
    if (bit_set && has_flags) {
      *out_index = i;
      return true;
    }
  }
  return false;
}

static VKAPI_ATTR void VKAPI_CALL wp_GetDeviceQueue(VkDevice device,
                                                    uint32_t queueFamilyIndex,
                                                    uint32_t queueIndex,
                                                    VkQueue *pQueue) {
  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(device));
  data->real_get_device_queue(device, queueFamilyIndex, queueIndex, pQueue);
  if (wp_capture_is_target_process()) {
    wp_queue_family_set(data, *pQueue, queueFamilyIndex);
  }
}

static VKAPI_ATTR void VKAPI_CALL wp_GetDeviceQueue2(
    VkDevice device, const VkDeviceQueueInfo2 *pQueueInfo, VkQueue *pQueue) {
  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(device));
  data->real_get_device_queue2(device, pQueueInfo, pQueue);
  if (wp_capture_is_target_process()) {
    wp_queue_family_set(data, *pQueue, pQueueInfo->queueFamilyIndex);
  }
}

static VKAPI_ATTR VkResult VKAPI_CALL wp_CreateSwapchainKHR(
    VkDevice device, const VkSwapchainCreateInfoKHR *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkSwapchainKHR *pSwapchain) {
  (void)pAllocator;
  WP_LOG("create_swapchain_khr: entered");
  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(device));
  VkResult res =
      data->real_create_swapchain_khr(device, pCreateInfo, NULL, pSwapchain);
  WP_LOG("create_swapchain_khr: next-in-chain returned %d", (int)res);

  if (res == VK_SUCCESS && wp_capture_is_target_process()) {
    wp_register_swapchain(data, *pSwapchain, pCreateInfo);
  }
  return res;
}

static VKAPI_ATTR void VKAPI_CALL
wp_DestroySwapchainKHR(VkDevice device, VkSwapchainKHR swapchain,
                       const VkAllocationCallbacks *pAllocator) {
  (void)pAllocator;
  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(device));
  if (wp_capture_is_target_process()) {
    wp_teardown_swapchain(data, swapchain);
  }
  data->real_destroy_swapchain_khr(device, swapchain, NULL);
}

static VKAPI_ATTR VkResult VKAPI_CALL
wp_QueuePresentKHR(VkQueue queue, const VkPresentInfoKHR *pPresentInfo) {
  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(queue));

  if (wp_capture_is_target_process()) {
    if (pPresentInfo->swapchainCount > 0) {
      VkSwapchainKHR swapchain = pPresentInfo->pSwapchains[0];
      uint32_t image_index = pPresentInfo->pImageIndices[0];
      wp_capture_and_notify(data, queue, swapchain, image_index);
    }
  }

  return data->real_queue_present_khr(queue, pPresentInfo);
}

WP_VK_EXPORT VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL
vkGetDeviceProcAddr(VkDevice device, const char *pName) {
  static const wp_fn_entry_t table[] = {
      {"vkGetDeviceProcAddr", (PFN_vkVoidFunction)vkGetDeviceProcAddr},
      {"vkDestroyDevice", (PFN_vkVoidFunction)wp_DestroyDevice},
      {"vkGetDeviceQueue", (PFN_vkVoidFunction)wp_GetDeviceQueue},
      {"vkGetDeviceQueue2", (PFN_vkVoidFunction)wp_GetDeviceQueue2},
      {"vkCreateSwapchainKHR", (PFN_vkVoidFunction)wp_CreateSwapchainKHR},
      {"vkDestroySwapchainKHR", (PFN_vkVoidFunction)wp_DestroySwapchainKHR},
      {"vkQueuePresentKHR", (PFN_vkVoidFunction)wp_QueuePresentKHR},
  };
  for (size_t i = 0; i < sizeof(table) / sizeof(table[0]); i++) {
    if (strcmp(pName, table[i].name) == 0) {
      return table[i].fn;
    }
  }

  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(device));
  if (!data || !data->next_gdpa) {
    return NULL;
  }
  return data->next_gdpa(device, pName);
}
