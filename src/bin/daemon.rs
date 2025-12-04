use macos_resource_monitor::alert;
use macos_resource_monitor::config;
use macos_resource_monitor::logging::Logger;
use macos_resource_monitor::metrics::MetricsCollector;
use macos_resource_monitor::notify::Notifier;
use std::thread;
use std::time::Duration;

fn main() {
    // Загружаем или создаём конфиг
    let cfg = match config::load_or_create_default() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to load config: {e}");
            return;
        }
    };

    let logger = match Logger::new(&cfg.service.log_file_path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to init logger: {e}");
            return;
        }
    };

    let notifier = Notifier::new();

    let _ = logger.log_event("INFO", "MacResourceMonitor daemon started");
    notifier.info("MacResourceMonitor daemon started");

    let mut collector = MetricsCollector::new();

    loop {
        let snapshot = collector.collect();

        if let Err(e) = logger.log_snapshot(&snapshot) {
            eprintln!("Failed to write snapshot: {e}");
            notifier.error("Failed to write snapshot to log file");
        }

        let alerts = alert::check_thresholds(&snapshot, &cfg);
        for a in alerts {
            let _ = logger.log_event("ALERT", &a);
            notifier.warning(&a);
        }

        thread::sleep(Duration::from_secs(
            cfg.service.collection_interval_secs,
        ));
    }
}
