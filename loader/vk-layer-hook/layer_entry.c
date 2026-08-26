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

#include <string.h>

#define WP_LAYER_NAME "VK_LAYER_wallpiper_capture"
#define WP_LAYER_DESCRIPTION "Wallpiper frame capture layer"

static void fill_layer_properties(VkLayerProperties *props) {
  memset(props, 0, sizeof(*props));
  strncpy(props->layerName, WP_LAYER_NAME, VK_MAX_EXTENSION_NAME_SIZE - 1);
  props->specVersion = VK_API_VERSION_1_1;
  props->implementationVersion = 1;
  strncpy(props->description, WP_LAYER_DESCRIPTION,
          VK_MAX_DESCRIPTION_SIZE - 1);
}

static VkResult enumerate_one_layer(uint32_t *pPropertyCount,
                                    VkLayerProperties *pProperties) {
  if (!pProperties) {
    *pPropertyCount = 1;
    return VK_SUCCESS;
  }
  if (*pPropertyCount < 1) {
    *pPropertyCount = 0;
    return VK_INCOMPLETE;
  }
  fill_layer_properties(&pProperties[0]);
  *pPropertyCount = 1;
  return VK_SUCCESS;
}

WP_VK_EXPORT VKAPI_ATTR VkResult VKAPI_CALL vkEnumerateInstanceLayerProperties(
    uint32_t *pPropertyCount, VkLayerProperties *pProperties) {
  return enumerate_one_layer(pPropertyCount, pProperties);
}

WP_VK_EXPORT VKAPI_ATTR VkResult VKAPI_CALL
vkEnumerateInstanceExtensionProperties(const char *pLayerName,
                                       uint32_t *pPropertyCount,
                                       VkExtensionProperties *pProperties) {
  (void)pProperties;
  if (!pLayerName || strcmp(pLayerName, WP_LAYER_NAME) != 0) {
    return VK_ERROR_LAYER_NOT_PRESENT;
  }
  *pPropertyCount = 0;
  return VK_SUCCESS;
}

WP_VK_EXPORT VKAPI_ATTR VkResult VKAPI_CALL vkEnumerateDeviceLayerProperties(
    VkPhysicalDevice physicalDevice, uint32_t *pPropertyCount,
    VkLayerProperties *pProperties) {
  (void)physicalDevice;
  return enumerate_one_layer(pPropertyCount, pProperties);
}

WP_VK_EXPORT VKAPI_ATTR VkResult VKAPI_CALL
vkEnumerateDeviceExtensionProperties(VkPhysicalDevice physicalDevice,
                                     const char *pLayerName,
                                     uint32_t *pPropertyCount,
                                     VkExtensionProperties *pProperties) {
  (void)physicalDevice;
  (void)pProperties;
  if (!pLayerName || strcmp(pLayerName, WP_LAYER_NAME) != 0) {
    return VK_ERROR_LAYER_NOT_PRESENT;
  }
  *pPropertyCount = 0;
  return VK_SUCCESS;
}
