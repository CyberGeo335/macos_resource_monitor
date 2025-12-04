use crate::config::Config;
use crate::metrics::ResourceSnapshot;

pub fn check_thresholds(snapshot: &ResourceSnapshot, cfg: &Config) -> Vec<String> {
    let mut alerts = Vec::new();

    if snapshot.cpu_usage_percent > cfg.thresholds.cpu_usage_percent {
        alerts.push(format!(
            "CPU usage {:.1}% exceeded threshold {:.1}%",
            snapshot.cpu_usage_percent, cfg.thresholds.cpu_usage_percent
        ));
    }

    if snapshot.memory_usage_percent > cfg.thresholds.memory_usage_percent {
        alerts.push(format!(
            "Memory usage {:.1}% exceeded threshold {:.1}%",
            snapshot.memory_usage_percent, cfg.thresholds.memory_usage_percent
        ));
    }

    if snapshot.disk_usage_percent > cfg.thresholds.disk_usage_percent {
        alerts.push(format!(
            "Disk usage {:.1}% exceeded threshold {:.1}%",
            snapshot.disk_usage_percent, cfg.thresholds.disk_usage_percent
        ));
    }

    if snapshot.net_in_kbps > cfg.thresholds.net_in_kbps {
        alerts.push(format!(
            "Inbound network {:.1} kbit/s exceeded threshold {:.1} kbit/s",
            snapshot.net_in_kbps, cfg.thresholds.net_in_kbps
        ));
    }

    if snapshot.net_out_kbps > cfg.thresholds.net_out_kbps {
        alerts.push(format!(
            "Outbound network {:.1} kbit/s exceeded threshold {:.1} kbit/s",
            snapshot.net_out_kbps, cfg.thresholds.net_out_kbps
        ));
    }

    alerts
}
