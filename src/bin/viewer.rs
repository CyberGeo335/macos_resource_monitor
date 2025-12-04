use macos_resource_monitor::config;
use macos_resource_monitor::metrics::ResourceSnapshot;
use serde::Deserialize;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Deserialize)]
struct LogEvent {
    timestamp: String,
    level: String,
    message: String,
}

fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().filter_map(Result::ok).collect())
}

fn show_last(log_path: &str, n: usize) -> std::io::Result<()> {
    let lines = read_lines(log_path)?;
    let start = lines.len().saturating_sub(n);
    for line in &lines[start..] {
        println!("{line}");
    }
    Ok(())
}

fn show_stats(log_path: &str) -> std::io::Result<()> {
    let lines = read_lines(log_path)?;

    let mut cpu_sum = 0.0f64;
    let mut mem_sum = 0.0f64;
    let mut disk_sum = 0.0f64;
    let mut net_in_max = 0.0f32;
    let mut net_out_max = 0.0f32;
    let mut count = 0u64;

    for line in lines {
        // Пытаемся парсить как снапшот
        if let Ok(snapshot) = serde_json::from_str::<ResourceSnapshot>(&line) {
            cpu_sum += snapshot.cpu_usage_percent as f64;
            mem_sum += snapshot.memory_usage_percent as f64;
            disk_sum += snapshot.disk_usage_percent as f64;
            if snapshot.net_in_kbps > net_in_max {
                net_in_max = snapshot.net_in_kbps;
            }
            if snapshot.net_out_kbps > net_out_max {
                net_out_max = snapshot.net_out_kbps;
            }
            count += 1;
        }
    }

    if count == 0 {
        println!("No metric snapshots found in log.");
        return Ok(());
    }

    println!("Statistics over {count} samples:");
    println!(
        "Average CPU usage: {:.1}%",
        cpu_sum / count as f64
    );
    println!(
        "Average memory usage: {:.1}%",
        mem_sum / count as f64
    );
    println!(
        "Average disk usage: {:.1}%",
        disk_sum / count as f64
    );
    println!("Max inbound network: {:.1} kbit/s", net_in_max);
    println!("Max outbound network: {:.1} kbit/s", net_out_max);

    Ok(())
}

fn print_usage() {
    eprintln!("Usage:");
    eprintln!("  viewer last [N]");
    eprintln!("  viewer stats");
}

fn main() {
    let cfg = match config::load_or_create_default() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            return;
        }
    };

    let log_path = cfg.service.log_file_path;

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return;
    }

    match args[1].as_str() {
        "last" => {
            let n = if args.len() >= 3 {
                args[2].parse::<usize>().unwrap_or(10)
            } else {
                10
            };
            if let Err(e) = show_last(&log_path, n) {
                eprintln!("Error: {e}");
            }
        }
        "stats" => {
            if let Err(e) = show_stats(&log_path) {
                eprintln!("Error: {e}");
            }
        }
        _ => print_usage(),
    }
}
