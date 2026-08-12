#pragma once
#include "../x11.h"

void capture_on_put_image(unsigned long drawable, const XImageCompat* image, int src_x, int src_y, int dst_x, int dst_y, unsigned int src_width, unsigned int src_height);
void mouse_track_start(void* display, unsigned long window, int width, int height);
