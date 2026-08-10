use std::fs::File;
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use crate::config::{wine_prefix, wineserver_bin};

pub fn ensure_persistent_wineserver() {
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
        Ok(child) => println!(
            "persistent wineserver ensured for {prefix} pid={}",
            child.id()
        ),
        Err(e) => println!("failed to start persistent wineserver ({bin}): {e}"),
    }
}
