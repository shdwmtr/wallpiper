use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

fn send_with_retry(args: &[&str]) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        match wallpiper_protocol::send_daemon_command(args) {
            Err(e) if e.contains("not running") && Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(20));
            }
            other => return other,
        }
    }
}

#[test]
fn round_trips_ok_and_err_responses() {
    let dir = format!("/tmp/wallpiper-protocol-test-{}", std::process::id());
    std::fs::create_dir_all(&dir).unwrap();
    std::env::set_var("WALLPIPER_RUNTIME_DIR", &dir);

    let seen = Arc::new(Mutex::new(Vec::new()));
    let seen_clone = seen.clone();
    wallpiper_protocol::spawn_daemon_ctl_listener(move |args| {
        seen_clone.lock().unwrap().push(args.join(" "));
        if args.first() == Some(&"fail") {
            Err("boom".to_string())
        } else {
            Ok(())
        }
    });

    assert!(send_with_retry(&["pause"]).is_ok());
    assert_eq!(send_with_retry(&["fail", "now"]), Err("boom".to_string()));

    assert_eq!(
        *seen.lock().unwrap(),
        vec!["pause".to_string(), "fail now".to_string()]
    );

    std::fs::remove_dir_all(&dir).ok();
}
