#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TrafficCounters {
    pub rx_bytes: u64, // Download
    pub tx_bytes: u64, // Upload
}

impl TrafficCounters {
    pub fn calculate_delta(&self, previous: &TrafficCounters) -> TrafficCounters {
        let rx_delta = if self.rx_bytes >= previous.rx_bytes {
            self.rx_bytes - previous.rx_bytes
        } else {
            self.rx_bytes
        };

        let tx_delta = if self.tx_bytes >= previous.tx_bytes {
            self.tx_bytes - previous.tx_bytes
        } else {
            self.tx_bytes
        };

        TrafficCounters {
            rx_bytes: rx_delta,
            tx_bytes: tx_delta,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculates_normal_delta() {
        let old = TrafficCounters {
            rx_bytes: 100,
            tx_bytes: 50,
        };
        let new = TrafficCounters {
            rx_bytes: 150,
            tx_bytes: 75,
        };
        let delta = new.calculate_delta(&old);

        assert_eq!(delta.rx_bytes, 50);
        assert_eq!(delta.tx_bytes, 25);
    }

    #[test]
    fn handles_counter_reset() {
        let old = TrafficCounters {
            rx_bytes: 1000,
            tx_bytes: 500,
        };
        let new = TrafficCounters {
            rx_bytes: 150,
            tx_bytes: 75,
        };
        let delta = new.calculate_delta(&old);

        assert_eq!(delta.rx_bytes, 150);
        assert_eq!(delta.tx_bytes, 75);
    }
}
