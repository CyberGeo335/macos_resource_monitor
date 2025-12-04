use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, Networks, System};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub cpu_usage_percent: f32,
    pub memory_used_mb: u64,
    pub memory_usage_percent: f32,
    pub disk_usage_percent: f32,
    pub net_in_kbps: f32,
    pub net_out_kbps: f32,
    pub total_net_in_bytes: u64,
    pub total_net_out_bytes: u64,
}

pub struct MetricsCollector {
    sys: System,
    disks: Disks,
    networks: Networks,
    last_total_in: u64,
    last_total_out: u64,
    last_timestamp: Option<DateTime<Utc>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        // Загружаем всё, что может понадобиться
        let mut sys = System::new_all();
        sys.refresh_all();

        // Список дисков и сетевых интерфейсов
        let disks = Disks::new_with_refreshed_list();
        let networks = Networks::new_with_refreshed_list();

        let (in_bytes, out_bytes) = Self::read_network_totals(&networks);

        MetricsCollector {
            sys,
            disks,
            networks,
            last_total_in: in_bytes,
            last_total_out: out_bytes,
            last_timestamp: None,
        }
    }

    fn read_network_totals(networks: &Networks) -> (u64, u64) {
        let mut total_in = 0u64;
        let mut total_out = 0u64;

        // Networks по сути HashMap<String, NetworkData>
        for (_name, data) in networks.iter() {
            total_in = total_in.saturating_add(data.total_received());
            total_out = total_out.saturating_add(data.total_transmitted());
        }

        (total_in, total_out)
    }

    pub fn collect(&mut self) -> ResourceSnapshot {
        // Обновляем CPU/память/диски/сеть
        self.sys.refresh_cpu();      // в 0.30.13 есть
        self.sys.refresh_memory();

        self.disks.refresh();        // без параметров
        self.networks.refresh();     // без параметров

        let now = Utc::now();

        // CPU: среднее по всем ядрам
        let mut cpu_total = 0f32;
        let cpus = self.sys.cpus();
        for cpu in cpus {
            cpu_total += cpu.cpu_usage();
        }
        let cpu_usage_percent = if cpus.is_empty() {
            0.0
        } else {
            cpu_total / cpus.len() as f32
        };

        // Память
        let total_mem = self.sys.total_memory();
        let used_mem = self.sys.used_memory();
        let memory_usage_percent = if total_mem == 0 {
            0.0
        } else {
            (used_mem as f32 / total_mem as f32) * 100.0
        };

        // sysinfo 0.30 возвращает память в KiB, поэтому MB = KiB / 1024
        let memory_used_mb = used_mem / 1024;

        // Диски (суммарно по всем)
        let mut total_space = 0u64;
        let mut available_space = 0u64;
        for disk in self.disks.list() {
            total_space = total_space.saturating_add(disk.total_space());
            available_space = available_space.saturating_add(disk.available_space());
        }
        let used_space = total_space.saturating_sub(available_space);
        let disk_usage_percent = if total_space == 0 {
            0.0
        } else {
            (used_space as f32 / total_space as f32) * 100.0
        };

        // Сеть
        let (total_in, total_out) = Self::read_network_totals(&self.networks);

        let (net_in_kbps, net_out_kbps) = if let Some(last_ts) = self.last_timestamp {
            let dt_ms = now.signed_duration_since(last_ts).num_milliseconds();
            if dt_ms > 0 {
                let delta_secs = dt_ms as f32 / 1000.0;
                let delta_in = total_in.saturating_sub(self.last_total_in);
                let delta_out = total_out.saturating_sub(self.last_total_out);

                // кбит/с: bytes * 8 / 1024 / сек
                (
                    (delta_in as f32 * 8.0 / 1024.0) / delta_secs,
                    (delta_out as f32 * 8.0 / 1024.0) / delta_secs,
                )
            } else {
                (0.0, 0.0)
            }
        } else {
            // первый замер — скорости ещё не считаем
            (0.0, 0.0)
        };

        self.last_total_in = total_in;
        self.last_total_out = total_out;
        self.last_timestamp = Some(now);

        ResourceSnapshot {
            timestamp: now,
            cpu_usage_percent,
            memory_used_mb,
            memory_usage_percent,
            disk_usage_percent,
            net_in_kbps,
            net_out_kbps,
            total_net_in_bytes: total_in,
            total_net_out_bytes: total_out,
        }
    }
}
