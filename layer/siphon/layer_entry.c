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
