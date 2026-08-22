#include "vk_layer.h"

#include <stdio.h>

#include "wallpiper/fsutil.h"
#include "wallpiper/protocol.h"
#include "wallpiper/steam_paths.h"

void wp_write_vk_layer_manifest(void) {
  char temp_dir[512];
  if (!wp_temp_dir(temp_dir, sizeof(temp_dir))) {
    printf("failed to resolve temp dir for vk layer manifest\n");
    return;
  }
  wp_mkdir_p(temp_dir);

  char install_dir[1024];
  if (!wp_install_dir(install_dir, sizeof(install_dir))) {
    printf("failed to resolve install dir for vk layer manifest\n");
    return;
  }

  char path[600];
  int n = snprintf(path, sizeof(path), "%s/%s.json", temp_dir,
                   WP_VK_CAPTURE_LAYER_NAME);
  if (n <= 0 || (size_t)n >= sizeof(path)) {
    printf("vk layer manifest path too long\n");
    return;
  }

  FILE *f = fopen(path, "w");
  if (!f) {
    printf("failed to write vk layer manifest at %s\n", path);
    return;
  }

  fprintf(f,
          "{\n"
          "    \"file_format_version\" : \"1.0.0\",\n"
          "    \"layer\" : {\n"
          "        \"name\": \"%s\",\n"
          "        \"type\": \"GLOBAL\",\n"
          "        \"library_path\": \"%s/libVkLayer_wallpiper_capture.so\",\n"
          "        \"api_version\": \"1.1.0\",\n"
          "        \"implementation_version\": \"1\",\n"
          "        \"description\": \"Wallpiper frame capture layer\"\n"
          "    }\n"
          "}\n",
          WP_VK_CAPTURE_LAYER_NAME, install_dir);
  fclose(f);
}
