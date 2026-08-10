use crate::portal;
use crate::process::{find_renderer_pids, find_webwallpaper_pids, kill_pids_gracefully};

pub fn cleanup() {
    println!("wallpiperd shutting down, cleaning up spawned processes");

    let detached = portal::detach_display();
    println!(
        "display detach handshake -> {}",
        if detached {
            "ok"
        } else {
            "failed or timed out, proceeding anyway"
        }
    );

    let mut pids = find_renderer_pids();
    pids.extend(find_webwallpaper_pids());
    kill_pids_gracefully(&pids);

    if let Some(pid) = portal::take_display_pid() {
        kill_pids_gracefully(&[pid]);
    }
}
