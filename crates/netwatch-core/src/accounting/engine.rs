use crate::database::{accounting::AccountingRepository, devices::DeviceRepository};
use crate::error::NetWatchError;
use crate::traffic::{DeviceTraffic, TrafficCounters};
use chrono::Local;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct LiveBandwidth {
    pub rx_bps: u64,
    pub tx_bps: u64,
}

pub struct AccountingEngine {
    last_poll: Option<Instant>,
}

impl AccountingEngine {
    pub fn new() -> Self {
        Self { last_poll: None }
    }

    pub fn process_poll(
        &mut self,
        conn: &mut rusqlite::Connection,
        traffics: &[DeviceTraffic],
    ) -> Result<HashMap<String, LiveBandwidth>, NetWatchError> {
        let now = Instant::now();
        let elapsed_secs = if let Some(last) = self.last_poll {
            now.duration_since(last).as_secs_f64()
        } else {
            0.0
        };
        self.last_poll = Some(now);

        let tx = conn
            .transaction()
            .map_err(crate::error::DatabaseError::Query)?;
        let today = Local::now().format("%Y-%m-%d").to_string(); // Timezone Aware!

        let mut live_bandwidth = HashMap::new();

        for dt in traffics {
            let device = DeviceRepository::upsert(&tx, &dt.mac_address, None, None, None)
                .map_err(crate::error::DatabaseError::Query)?;

            let previous = TrafficCounters {
                rx_bytes: device.last_rx_bytes,
                tx_bytes: device.last_tx_bytes,
            };

            let delta = dt.counters.calculate_delta(&previous);

            if elapsed_secs > 0.0 {
                live_bandwidth.insert(
                    dt.mac_address.clone(),
                    LiveBandwidth {
                        rx_bps: (delta.rx_bytes as f64 / elapsed_secs) as u64,
                        tx_bps: (delta.tx_bytes as f64 / elapsed_secs) as u64,
                    },
                );
            } else {
                live_bandwidth.insert(dt.mac_address.clone(), LiveBandwidth::default());
            }

            if delta.rx_bytes > 0 || delta.tx_bytes > 0 {
                AccountingRepository::record_usage(
                    &tx,
                    device.id,
                    &today,
                    delta.rx_bytes,
                    delta.tx_bytes,
                    dt.counters.rx_bytes,
                    dt.counters.tx_bytes,
                )
                .map_err(crate::error::DatabaseError::Query)?;
            } else if dt.counters != previous {
                AccountingRepository::update_last_counters(
                    &tx,
                    device.id,
                    dt.counters.rx_bytes,
                    dt.counters.tx_bytes,
                )
                .map_err(crate::error::DatabaseError::Query)?;
            }
        }

        tx.commit().map_err(crate::error::DatabaseError::Query)?;
        Ok(live_bandwidth)
    }
}
