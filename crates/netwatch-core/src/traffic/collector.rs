use super::counters::TrafficCounters;
use crate::error::TrafficError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceTraffic {
    pub mac_address: String,
    pub counters: TrafficCounters,
}

pub trait Collector {
    fn collect(&self) -> Result<Vec<DeviceTraffic>, TrafficError>;
}
