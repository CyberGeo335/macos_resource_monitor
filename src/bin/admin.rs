use std::env;
use std::path::PathBuf;
use std::process::Command;

fn plist_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME not set");
    PathBuf::from(home)
        .join("Library")
        .join("LaunchAgents")
        .join("com.example.macresourcemonitor.plist")
}

fn run_command(cmd: &mut Command) -> bool {
    match cmd.status() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            eprintln!("Command failed with status: {status}");
            false
        }
        Err(e) => {
            eprintln!("Failed to run command: {e}");
            false
        }
    }
}

fn cmd_start() {
    let plist = plist_path();
    println!("Starting service with plist: {}", plist.display());
    let mut cmd = Command::new("launchctl");
    cmd.arg("load").arg("-w").arg(plist);
    let ok = run_command(&mut cmd);
    if ok {
        println!("Service started");
    }
}

fn cmd_stop() {
    let plist = plist_path();
    println!("Stopping service with plist: {}", plist.display());
    let mut cmd = Command::new("launchctl");
    cmd.arg("unload").arg("-w").arg(plist);
    let ok = run_command(&mut cmd);
    if ok {
        println!("Service stopped");
    }
}

fn cmd_restart() {
    cmd_stop();
    cmd_start();
}

fn cmd_status() {
    let mut cmd = Command::new("launchctl");
    cmd.arg("list").arg("com.example.macresourcemonitor");
    let ok = run_command(&mut cmd);
    if ok {
        println!("If the service is listed above, it is running.");
    } else {
        println!("Service is probably not running.");
    }
}

fn print_usage() {
    eprintln!("Usage: admin <start|stop|restart|status>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "start" => cmd_start(),
        "stop" => cmd_stop(),
        "restart" => cmd_restart(),
        "status" => cmd_status(),
        _ => print_usage(),
    }
}
