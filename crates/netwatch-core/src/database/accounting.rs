use rusqlite::{Connection, Result as SqliteResult, params};

pub struct AccountingRepository;

impl AccountingRepository {
    /// update the daily usage for a device and also update the last counters in the devices table
    pub fn record_usage(
        conn: &Connection,
        device_id: i64,
        date: &str, // YYYY-MM-DD
        rx_delta: u64,
        tx_delta: u64,
        new_rx_total: u64,
        new_tx_total: u64,
    ) -> SqliteResult<()> {
        conn.execute(
            "INSERT INTO device_usage_daily (device_id, date, rx_bytes, tx_bytes)
             VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(device_id, date) DO UPDATE SET
                rx_bytes = rx_bytes + excluded.rx_bytes,
                tx_bytes = tx_bytes + excluded.tx_bytes",
            params![device_id, date, rx_delta as i64, tx_delta as i64],
        )?;

        Self::update_last_counters(conn, device_id, new_rx_total, new_tx_total)?;
        Ok(())
    }

    pub fn update_last_counters(
        conn: &Connection,
        device_id: i64,
        new_rx: u64,
        new_tx: u64,
    ) -> SqliteResult<()> {
        conn.execute(
            "UPDATE devices 
             SET last_rx_bytes = ?1, last_tx_bytes = ?2, last_counter_update = CURRENT_TIMESTAMP
             WHERE id = ?3",
            params![new_rx as i64, new_tx as i64, device_id],
        )?;
        Ok(())
    }

    pub fn get_device_usage_between(
        conn: &Connection,
        device_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> SqliteResult<(u64, u64)> {
        let mut stmt = conn.prepare(
            "SELECT SUM(rx_bytes), SUM(tx_bytes) 
             FROM device_usage_daily 
             WHERE device_id = ?1 AND date >= ?2 AND date <= ?3",
        )?;

        stmt.query_row(params![device_id, start_date, end_date], |row| {
            let rx: Option<i64> = row.get(0)?;
            let tx: Option<i64> = row.get(1)?;
            Ok((rx.unwrap_or(0) as u64, tx.unwrap_or(0) as u64))
        })
    }

    pub fn get_user_usage_between(
        conn: &Connection,
        user_id: i64,
        start_date: &str,
        end_date: &str,
    ) -> SqliteResult<(u64, u64)> {
        let mut stmt = conn.prepare(
            "SELECT SUM(rx_bytes), SUM(tx_bytes) 
             FROM device_usage_daily 
             JOIN devices ON devices.id = device_usage_daily.device_id
             WHERE devices.user_id = ?1 AND date >= ?2 AND date <= ?3",
        )?;

        stmt.query_row(params![user_id, start_date, end_date], |row| {
            let rx: Option<i64> = row.get(0)?;
            let tx: Option<i64> = row.get(1)?;
            Ok((rx.unwrap_or(0) as u64, tx.unwrap_or(0) as u64))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::devices::DeviceRepository;
    use crate::database::setup_test_db;

    #[test]
    fn records_daily_usage_and_updates_counters() {
        let conn = setup_test_db();
        let device = DeviceRepository::upsert(&conn, "AA:BB:CC", None, None).unwrap();

        AccountingRepository::record_usage(&conn, device.id, "2026-08-17", 500, 100, 1500, 300)
            .unwrap();

        let stats = AccountingRepository::get_device_usage_between(
            &conn,
            device.id,
            "2026-08-17",
            "2026-08-17",
        )
        .unwrap();

        assert_eq!(stats.0, 500);
        assert_eq!(stats.1, 100);

        let updated = DeviceRepository::get_by_id(&conn, device.id).unwrap();
        assert_eq!(updated.last_rx_bytes, 1500);
        assert_eq!(updated.last_tx_bytes, 300);
    }
}
