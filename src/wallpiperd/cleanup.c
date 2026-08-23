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

#include "cleanup.h"

#include "portal.h"
#include "process.h"

#include <stdio.h>

void wp_cleanup(void) {
  printf("wallpiperd shutting down, cleaning up spawned processes\n");

  bool detached = wp_portal_detach_display();
  printf("display detach handshake -> %s\n",
         detached ? "ok" : "failed or timed out, proceeding anyway");

  wp_pid_list_t pids;
  wp_find_renderer_pids(&pids);

  wp_pid_list_t webwallpaper_pids;
  wp_find_webwallpaper_pids(&webwallpaper_pids);
  for (size_t i = 0; i < webwallpaper_pids.count &&
                     pids.count < sizeof(pids.pids) / sizeof(pids.pids[0]);
       i++) {
    pids.pids[pids.count++] = webwallpaper_pids.pids[i];
  }

  wp_kill_pids_gracefully(pids.pids, pids.count);

  int display_pid;
  if (wp_portal_take_display_pid(&display_pid)) {
    wp_kill_pids_gracefully(&display_pid, 1);
  }
}
