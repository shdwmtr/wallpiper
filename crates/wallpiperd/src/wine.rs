use std::fs::File;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use crate::config::{wine_prefix, wineserver_bin, wineserver_pidfile};

fn pid_alive(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}

fn kill_previous_wineserver() {
    let pidfile = wineserver_pidfile();
    let Ok(contents) = std::fs::read_to_string(&pidfile) else {
        return;
    };
    let Ok(pid) = contents.trim().parse::<i32>() else {
        let _ = std::fs::remove_file(&pidfile);
        return;
    };
    if !pid_alive(pid) {
        let _ = std::fs::remove_file(&pidfile);
        return;
    }

    println!("stopping wineserver from a previous wallpiperd run pid={pid}");
    unsafe {
        libc::kill(pid, libc::SIGTERM);
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    while pid_alive(pid) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(100));
    }
    if pid_alive(pid) {
        println!("wineserver pid={pid} did not exit, sending SIGKILL");
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
    let _ = std::fs::remove_file(&pidfile);
}

pub fn ensure_persistent_wineserver() {
    kill_previous_wineserver();

    let bin = wineserver_bin();
    let prefix = wine_prefix();
    let Ok(logfile) = File::create("/tmp/wallpiperd-wineserver.log") else {
        println!("failed to create wineserver logfile");
        return;
    };
    let Ok(logfile_err) = logfile.try_clone() else {
        return;
    };
    match Command::new(&bin)
        .arg("-p")
        .env("WINEPREFIX", &prefix)
        .stdout(Stdio::from(logfile))
        .stderr(Stdio::from(logfile_err))
        .process_group(0)
        .spawn()
    {
        Ok(child) => {
            let pid = child.id();
            println!("persistent wineserver ensured for {prefix} pid={pid}");
            if let Err(e) = std::fs::write(wineserver_pidfile(), pid.to_string()) {
                println!("failed to record wineserver pidfile: {e}");
            }
        }
        Err(e) => println!("failed to start persistent wineserver ({bin}): {e}"),
    }
}
