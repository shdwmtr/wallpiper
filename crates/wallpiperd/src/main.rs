fn main() {
    if std::env::args().nth(1).is_some() {
        eprintln!("wallpiperd takes no arguments; use `wallpiperctl` to control a running daemon");
        std::process::exit(1);
    }

    wallpiperd::run();
}
