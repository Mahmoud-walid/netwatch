use std::fs;
use std::path::Path;

use super::collector::{Collector, DeviceTraffic};
use super::counters::TrafficCounters;
use crate::error::TrafficError;

pub struct LocalCollector;

impl Collector for LocalCollector {
    fn collect(&self) -> Result<Vec<DeviceTraffic>, TrafficError> {
        let mut results = Vec::new();
        let net_dir = Path::new("/sys/class/net");

        if !net_dir.exists() {
            return Err(TrafficError::Collector(
                "sysfs network directory /sys/class/net not found".into(),
            ));
        }

        for entry in fs::read_dir(net_dir)? {
            let entry = entry?;
            let iface_name = entry.file_name().to_string_lossy().to_string();

            if iface_name == "lo" {
                continue;
            }

            let mac_path = entry.path().join("address");
            let rx_path = entry.path().join("statistics/rx_bytes");
            let tx_path = entry.path().join("statistics/tx_bytes");

            if !mac_path.exists() || !rx_path.exists() || !tx_path.exists() {
                continue;
            }

            let mac = fs::read_to_string(mac_path)?.trim().to_uppercase();
            if mac.is_empty() || mac == "00:00:00:00:00:00" {
                continue;
            }

            let rx_bytes: u64 = fs::read_to_string(rx_path)?.trim().parse().unwrap_or(0);
            let tx_bytes: u64 = fs::read_to_string(tx_path)?.trim().parse().unwrap_or(0);

            results.push(DeviceTraffic {
                mac_address: mac,
                counters: TrafficCounters { rx_bytes, tx_bytes },
            });
        }

        Ok(results)
    }
}
