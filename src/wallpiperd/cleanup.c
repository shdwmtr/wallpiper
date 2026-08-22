#include "cleanup.h"

#include "portal.h"
#include "process.h"

#include <stdio.h>

void wp_cleanup(void) {
  printf("wallpiper-daemon shutting down, cleaning up spawned processes\n");

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
