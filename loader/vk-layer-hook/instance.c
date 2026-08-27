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

#include "wp_layer.h"

#include "logging.h"
#include "process.h"

#include <pthread.h>
#include <stdlib.h>
#include <string.h>

#include <vulkan/vk_layer.h>

typedef struct {
  VkStructureType sType;
  const void *pNext;
  uint32_t flags;
  void *dpy;
  unsigned long window;
} wp_xlib_surface_create_info_t;

#ifndef VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR
#define VK_STRUCTURE_TYPE_XLIB_SURFACE_CREATE_INFO_KHR 1000004000
#endif

typedef VkResult(VKAPI_PTR *PFN_wp_vkCreateXlibSurfaceKHR)(
    VkInstance instance, const wp_xlib_surface_create_info_t *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkSurfaceKHR *pSurface);

#define WP_MAX_TRACKED_SURFACES 16

typedef struct {
  bool in_use;
  VkSurfaceKHR surface;
  unsigned long xid;
} wp_surface_xid_entry_t;

static pthread_mutex_t g_surface_xid_mutex = PTHREAD_MUTEX_INITIALIZER;
static wp_surface_xid_entry_t g_surface_xids[WP_MAX_TRACKED_SURFACES];

void wp_surface_xid_register(VkSurfaceKHR surface, unsigned long xid) {
  pthread_mutex_lock(&g_surface_xid_mutex);
  for (size_t i = 0; i < WP_MAX_TRACKED_SURFACES; i++) {
    if (!g_surface_xids[i].in_use) {
      g_surface_xids[i].in_use = true;
      g_surface_xids[i].surface = surface;
      g_surface_xids[i].xid = xid;
      break;
    }
  }
  pthread_mutex_unlock(&g_surface_xid_mutex);
}

bool wp_surface_xid_lookup(VkSurfaceKHR surface, unsigned long *out_xid) {
  bool found = false;
  pthread_mutex_lock(&g_surface_xid_mutex);
  for (size_t i = 0; i < WP_MAX_TRACKED_SURFACES; i++) {
    if (g_surface_xids[i].in_use && g_surface_xids[i].surface == surface) {
      *out_xid = g_surface_xids[i].xid;
      found = true;
      break;
    }
  }
  pthread_mutex_unlock(&g_surface_xid_mutex);
  return found;
}

static const char *const REQUIRED_DEVICE_EXTENSIONS[] = {
    "VK_EXT_image_drm_format_modifier",
    "VK_KHR_image_format_list",
    "VK_KHR_external_semaphore",
    "VK_KHR_external_semaphore_fd",
};
#define REQUIRED_DEVICE_EXTENSION_COUNT                                        \
  (sizeof(REQUIRED_DEVICE_EXTENSIONS) / sizeof(REQUIRED_DEVICE_EXTENSIONS[0]))

static VkInstance g_instance = VK_NULL_HANDLE;
static PFN_vkGetInstanceProcAddr g_next_gipa = NULL;
static PFN_vkDestroyInstance g_next_destroy_instance = NULL;
static PFN_vkGetPhysicalDeviceMemoryProperties
    g_get_physical_device_memory_properties = NULL;
static PFN_vkGetPhysicalDeviceFormatProperties2
    g_get_physical_device_format_properties2 = NULL;
static PFN_wp_vkCreateXlibSurfaceKHR g_next_create_xlib_surface_khr = NULL;

void wp_global_instance_set(VkInstance instance,
                            PFN_vkGetInstanceProcAddr next_gipa) {
  if (g_instance != VK_NULL_HANDLE) {
    return;
  }
  g_instance = instance;
  g_next_gipa = next_gipa;
  g_next_destroy_instance =
      (PFN_vkDestroyInstance)next_gipa(instance, "vkDestroyInstance");
  g_get_physical_device_memory_properties =
      (PFN_vkGetPhysicalDeviceMemoryProperties)next_gipa(
          instance, "vkGetPhysicalDeviceMemoryProperties");
  g_get_physical_device_format_properties2 =
      (PFN_vkGetPhysicalDeviceFormatProperties2)next_gipa(
          instance, "vkGetPhysicalDeviceFormatProperties2");
  g_next_create_xlib_surface_khr = (PFN_wp_vkCreateXlibSurfaceKHR)next_gipa(
      instance, "vkCreateXlibSurfaceKHR");
}

static VKAPI_ATTR VkResult VKAPI_CALL wp_CreateXlibSurfaceKHR(
    VkInstance instance, const wp_xlib_surface_create_info_t *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkSurfaceKHR *pSurface) {
  if (!g_next_create_xlib_surface_khr) {
    return VK_ERROR_INITIALIZATION_FAILED;
  }
  VkResult res = g_next_create_xlib_surface_khr(instance, pCreateInfo,
                                                pAllocator, pSurface);
  if (res == VK_SUCCESS && wp_capture_is_target_process()) {
    wp_surface_xid_register(*pSurface, pCreateInfo->window);
  }
  return res;
}

bool wp_global_instance_get_memory_properties(
    VkPhysicalDevice pd, VkPhysicalDeviceMemoryProperties *out) {
  if (!g_get_physical_device_memory_properties) {
    return false;
  }
  g_get_physical_device_memory_properties(pd, out);
  return true;
}

bool wp_global_instance_get_format_properties2(VkPhysicalDevice pd,
                                               VkFormat format,
                                               VkFormatProperties2 *out) {
  if (!g_get_physical_device_format_properties2) {
    return false;
  }
  VkFormatProperties2 props = {.sType = VK_STRUCTURE_TYPE_FORMAT_PROPERTIES_2};
  props.pNext = out->pNext;
  g_get_physical_device_format_properties2(pd, format, &props);
  *out = props;
  return true;
}

VKAPI_ATTR VkResult VKAPI_CALL wp_CreateInstance(
    const VkInstanceCreateInfo *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkInstance *pInstance) {
  VkLayerInstanceCreateInfo *layer_info =
      (VkLayerInstanceCreateInfo *)pCreateInfo->pNext;
  while (layer_info &&
         !(layer_info->sType == VK_STRUCTURE_TYPE_LOADER_INSTANCE_CREATE_INFO &&
           layer_info->function == VK_LAYER_LINK_INFO)) {
    layer_info = (VkLayerInstanceCreateInfo *)layer_info->pNext;
  }
  if (!layer_info) {
    return VK_ERROR_INITIALIZATION_FAILED;
  }

  PFN_vkGetInstanceProcAddr next_gipa =
      layer_info->u.pLayerInfo->pfnNextGetInstanceProcAddr;
  layer_info->u.pLayerInfo = layer_info->u.pLayerInfo->pNext;

  PFN_vkCreateInstance next_create_instance =
      (PFN_vkCreateInstance)next_gipa(NULL, "vkCreateInstance");
  if (!next_create_instance) {
    return VK_ERROR_INITIALIZATION_FAILED;
  }

  VkResult res = next_create_instance(pCreateInfo, pAllocator, pInstance);
  if (res != VK_SUCCESS) {
    return res;
  }

  wp_global_instance_set(*pInstance, next_gipa);
  return VK_SUCCESS;
}

VKAPI_ATTR void VKAPI_CALL wp_DestroyInstance(
    VkInstance instance, const VkAllocationCallbacks *pAllocator) {
  if (g_next_destroy_instance) {
    g_next_destroy_instance(instance, pAllocator);
  }
  if (instance == g_instance) {
    g_instance = VK_NULL_HANDLE;
    g_next_gipa = NULL;
    g_next_destroy_instance = NULL;
    g_get_physical_device_memory_properties = NULL;
    g_get_physical_device_format_properties2 = NULL;
    g_next_create_xlib_surface_khr = NULL;
  }
}

VKAPI_ATTR VkResult VKAPI_CALL wp_CreateDevice(
    VkPhysicalDevice physicalDevice, const VkDeviceCreateInfo *pCreateInfo,
    const VkAllocationCallbacks *pAllocator, VkDevice *pDevice) {
  VkLayerDeviceCreateInfo *layer_info =
      (VkLayerDeviceCreateInfo *)pCreateInfo->pNext;
  while (layer_info &&
         !(layer_info->sType == VK_STRUCTURE_TYPE_LOADER_DEVICE_CREATE_INFO &&
           layer_info->function == VK_LAYER_LINK_INFO)) {
    layer_info = (VkLayerDeviceCreateInfo *)layer_info->pNext;
  }
  if (!layer_info) {
    return VK_ERROR_INITIALIZATION_FAILED;
  }

  PFN_vkGetInstanceProcAddr next_gipa =
      layer_info->u.pLayerInfo->pfnNextGetInstanceProcAddr;
  PFN_vkGetDeviceProcAddr next_gdpa =
      layer_info->u.pLayerInfo->pfnNextGetDeviceProcAddr;
  layer_info->u.pLayerInfo = layer_info->u.pLayerInfo->pNext;

  PFN_vkCreateDevice next_create_device =
      (PFN_vkCreateDevice)next_gipa(g_instance, "vkCreateDevice");
  if (!next_create_device) {
    return VK_ERROR_INITIALIZATION_FAILED;
  }

  VkDeviceCreateInfo modified_info = *pCreateInfo;
  const char **extension_ptrs = NULL;

  if (wp_capture_is_target_process()) {
    size_t base_count = pCreateInfo->enabledExtensionCount;
    size_t max_count = base_count + REQUIRED_DEVICE_EXTENSION_COUNT;
    extension_ptrs = malloc(max_count * sizeof(const char *));
    if (extension_ptrs) {
      size_t n = 0;
      for (size_t i = 0; i < base_count; i++) {
        extension_ptrs[n++] = pCreateInfo->ppEnabledExtensionNames[i];
      }
      for (size_t r = 0; r < REQUIRED_DEVICE_EXTENSION_COUNT; r++) {
        bool present = false;
        for (size_t i = 0; i < base_count; i++) {
          if (strcmp(pCreateInfo->ppEnabledExtensionNames[i],
                     REQUIRED_DEVICE_EXTENSIONS[r]) == 0) {
            present = true;
            break;
          }
        }
        if (!present) {
          extension_ptrs[n++] = REQUIRED_DEVICE_EXTENSIONS[r];
        }
      }
      modified_info.enabledExtensionCount = (uint32_t)n;
      modified_info.ppEnabledExtensionNames = extension_ptrs;
    }
  }

  VkResult res =
      next_create_device(physicalDevice, &modified_info, pAllocator, pDevice);
  free(extension_ptrs);

  WP_LOG(
      "create_device: injected required extensions, next-in-chain returned %d",
      (int)res);

  if (res != VK_SUCCESS) {
    return res;
  }

  wp_device_data_t *data =
      wp_device_data_create(*pDevice, physicalDevice, next_gdpa);
  if (!data) {
    return VK_ERROR_OUT_OF_HOST_MEMORY;
  }

  return VK_SUCCESS;
}

VKAPI_ATTR void VKAPI_CALL
wp_DestroyDevice(VkDevice device, const VkAllocationCallbacks *pAllocator) {
  wp_device_data_t *data = wp_device_data_find(wp_dispatch_key(device));
  PFN_vkDestroyDevice real_destroy = NULL;
  if (data && data->next_gdpa) {
    real_destroy =
        (PFN_vkDestroyDevice)data->next_gdpa(device, "vkDestroyDevice");
  }
  wp_device_data_remove(wp_dispatch_key(device));
  if (real_destroy) {
    real_destroy(device, pAllocator);
  }
}

typedef struct {
  const char *name;
  PFN_vkVoidFunction fn;
} wp_instance_fn_entry_t;

WP_VK_EXPORT VKAPI_ATTR PFN_vkVoidFunction VKAPI_CALL
vkGetInstanceProcAddr(VkInstance instance, const char *pName) {
  static const wp_instance_fn_entry_t table[] = {
      {"vkGetInstanceProcAddr", (PFN_vkVoidFunction)vkGetInstanceProcAddr},
      {"vkCreateInstance", (PFN_vkVoidFunction)wp_CreateInstance},
      {"vkDestroyInstance", (PFN_vkVoidFunction)wp_DestroyInstance},
      {"vkCreateDevice", (PFN_vkVoidFunction)wp_CreateDevice},
      {"vkGetDeviceProcAddr", (PFN_vkVoidFunction)vkGetDeviceProcAddr},
      {"vkCreateXlibSurfaceKHR", (PFN_vkVoidFunction)wp_CreateXlibSurfaceKHR},
  };
  for (size_t i = 0; i < sizeof(table) / sizeof(table[0]); i++) {
    if (strcmp(pName, table[i].name) == 0) {
      return table[i].fn;
    }
  }

  if (!g_next_gipa) {
    return NULL;
  }
  return g_next_gipa(instance, pName);
}
