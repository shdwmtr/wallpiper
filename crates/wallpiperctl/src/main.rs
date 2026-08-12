use wallpiperd::{config, wallpaper};

const DAEMON_COMMANDS: &[&str] = &[
    "pause", "resume", "mute", "unmute", "volume", "debug", "nodebug", "set",
];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args: Vec<&str> = args.iter().map(String::as_str).collect();

    let Some((&cmd, rest)) = args.split_first() else {
        print_usage();
        std::process::exit(1);
    };

    let result = match cmd {
        "list-wallpapers" => list_wallpapers(rest),
        "list-properties" => list_properties(rest),
        "check-config" => {
            config::describe();
            Ok(())
        }
        _ if DAEMON_COMMANDS.contains(&cmd) => wallpiper_protocol::send_daemon_command(&args),
        _ => Err(format!("unknown command: {cmd}")),
    };

    if let Err(e) = result {
        eprintln!("wallpiperctl: {e}");
        std::process::exit(1);
    }
}

fn print_usage() {
    eprintln!(
        "usage: wallpiperctl <command> [args]\n\n\
         daemon commands (require a running wallpiperd):\n  \
         pause | resume | mute | unmute | volume <0-100> | debug | nodebug\n  \
         set <file> [location]\n  \
         set --id <workshop_id> [location]\n\n\
         standalone commands:\n  \
         list-wallpapers [-j]\n  \
         list-properties <workshop_id> [-j]\n  \
         check-config"
    );
}

fn list_wallpapers(args: &[&str]) -> Result<(), String> {
    let json = args.contains(&"-j");
    let wallpapers = wallpaper::list_wallpapers()?;

    if json {
        print_json(&wallpapers);
        return Ok(());
    }

    if wallpapers.is_empty() {
        println!("no workshop wallpapers found");
        return Ok(());
    }
    for w in wallpapers {
        println!("{}  {}  ({})", w.id, w.title, w.kind);
    }
    Ok(())
}

fn list_properties(args: &[&str]) -> Result<(), String> {
    let json = args.contains(&"-j");
    let Some(&id) = args.iter().find(|&&a| a != "-j") else {
        return Err("usage: list-properties <workshop_id> [-j]".to_string());
    };

    let (title, properties) = wallpaper::properties(id)?;

    if json {
        print_json(&serde_json::json!({
            "id": id,
            "title": title,
            "properties": properties,
        }));
        return Ok(());
    }

    println!("{title} ({id})");
    if properties.is_empty() {
        println!("  no properties");
        return Ok(());
    }
    for p in properties {
        println!("  {:<24} {:<8} {:<32} = {}", p.key, p.kind, p.text, p.value);
    }
    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(json) => println!("{json}"),
        Err(e) => eprintln!("failed to serialize json: {e}"),
    }
}
