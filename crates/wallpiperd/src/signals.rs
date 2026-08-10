pub fn reap_children_forever() {
    let mut signals = signal_hook::iterator::Signals::new([signal_hook::consts::SIGCHLD])
        .expect("failed to register SIGCHLD handler");
    std::thread::spawn(move || {
        for _ in signals.forever() {
            loop {
                let mut status: libc::c_int = 0;
                let pid = unsafe { libc::waitpid(-1, &mut status, libc::WNOHANG) };
                if pid <= 0 {
                    break;
                }
            }
        }
    });
}

pub fn install_shutdown_handler(on_shutdown: impl Fn() + Send + 'static) {
    let mut signals = signal_hook::iterator::Signals::new([
        signal_hook::consts::SIGINT,
        signal_hook::consts::SIGTERM,
    ])
    .expect("failed to register signal handler");
    std::thread::spawn(move || {
        if signals.forever().next().is_some() {
            on_shutdown();
            std::process::exit(0);
        }
    });
}
