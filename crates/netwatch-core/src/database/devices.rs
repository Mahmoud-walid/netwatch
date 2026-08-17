use crate::models::Device;
use rusqlite::{Connection, Result as SqliteResult, params};

pub struct DeviceRepository;

impl DeviceRepository {
    fn row_to_device(row: &rusqlite::Row) -> SqliteResult<Device> {
        let rx: Option<i64> = row.get(9).ok();
        let tx: Option<i64> = row.get(10).ok();

        Ok(Device {
            id: row.get(0)?,
            mac_address: row.get(1)?,
            ip_address: row.get(2)?,
            hostname: row.get(3)?,
            display_name: row.get(4)?,
            user_id: row.get(5)?,
            is_online: row.get(6)?,
            first_seen: row.get(7)?,
            last_seen: row.get(8)?,
            last_rx_bytes: rx.unwrap_or(0) as u64,
            last_tx_bytes: tx.unwrap_or(0) as u64,
        })
    }

    pub fn upsert(
        conn: &Connection,
        mac_address: &str,
        ip_address: Option<&str>,
        hostname: Option<&str>,
    ) -> SqliteResult<Device> {
        conn.execute(
            "INSERT INTO devices (mac_address, ip_address, hostname, is_online, last_seen)
             VALUES (?1, ?2, ?3, 1, CURRENT_TIMESTAMP)
             ON CONFLICT(mac_address) DO UPDATE SET
                ip_address = excluded.ip_address,
                hostname = excluded.hostname,
                is_online = 1,
                last_seen = CURRENT_TIMESTAMP",
            params![mac_address, ip_address, hostname],
        )?;

        Self::get_by_mac(conn, mac_address)
    }

    pub fn get_by_mac(conn: &Connection, mac_address: &str) -> SqliteResult<Device> {
        conn.query_row(
            "SELECT id, mac_address, ip_address, hostname, display_name, user_id, is_online, first_seen, last_seen, last_rx_bytes, last_tx_bytes 
             FROM devices WHERE mac_address = ?1",
            params![mac_address],
            Self::row_to_device,
        )
    }

    pub fn get_by_id(conn: &Connection, id: i64) -> SqliteResult<Device> {
        conn.query_row(
            "SELECT id, mac_address, ip_address, hostname, display_name, user_id, is_online, first_seen, last_seen, last_rx_bytes, last_tx_bytes 
             FROM devices WHERE id = ?1",
            params![id],
            Self::row_to_device,
        )
    }

    pub fn get_all(conn: &Connection) -> SqliteResult<Vec<Device>> {
        let mut stmt = conn.prepare(
            "SELECT id, mac_address, ip_address, hostname, display_name, user_id, is_online, first_seen, last_seen, last_rx_bytes, last_tx_bytes 
             FROM devices ORDER BY last_seen DESC",
        )?;

        let iter = stmt.query_map([], Self::row_to_device)?;
        let mut devices = Vec::new();
        for device in iter {
            devices.push(device?);
        }
        Ok(devices)
    }

    pub fn set_online_status(
        conn: &Connection,
        mac_address: &str,
        is_online: bool,
    ) -> SqliteResult<()> {
        conn.execute(
            "UPDATE devices SET is_online = ?1 WHERE mac_address = ?2",
            params![is_online, mac_address],
        )?;
        Ok(())
    }

    pub fn mark_all_offline(conn: &Connection) -> SqliteResult<()> {
        conn.execute("UPDATE devices SET is_online = 0", [])?;
        Ok(())
    }

    pub fn assign_user(
        conn: &Connection,
        device_id: i64,
        user_id: Option<i64>,
    ) -> SqliteResult<()> {
        conn.execute(
            "UPDATE devices SET user_id = ?1 WHERE id = ?2",
            params![user_id, device_id],
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::setup_test_db;
    use crate::database::users::UserRepository;

    #[test]
    fn upserts_new_device() {
        let conn = setup_test_db();
        let device = DeviceRepository::upsert(
            &conn,
            "AA:BB:CC:DD:EE:FF",
            Some("192.168.1.10"),
            Some("MyPhone"),
        )
        .unwrap();

        assert_eq!(device.mac_address, "AA:BB:CC:DD:EE:FF");
        assert_eq!(device.ip_address.as_deref(), Some("192.168.1.10"));
        assert_eq!(device.hostname.as_deref(), Some("MyPhone"));
        assert!(device.is_online);
    }

    #[test]
    fn upsert_updates_existing_device() {
        let conn = setup_test_db();
        let first = DeviceRepository::upsert(
            &conn,
            "AA:BB:CC:DD:EE:FF",
            Some("192.168.1.10"),
            Some("MyPhone"),
        )
        .unwrap();

        let second = DeviceRepository::upsert(
            &conn,
            "AA:BB:CC:DD:EE:FF",
            Some("192.168.1.15"),
            Some("MyPhone2"),
        )
        .unwrap();

        assert_eq!(first.id, second.id);
        assert_eq!(second.ip_address.as_deref(), Some("192.168.1.15"));
        assert_eq!(second.hostname.as_deref(), Some("MyPhone2"));
        assert!(second.is_online);
    }

    #[test]
    fn assigns_user_to_device() {
        let conn = setup_test_db();
        let user = UserRepository::create(&conn, "Mahmoud").unwrap();
        let device = DeviceRepository::upsert(&conn, "AA:BB:CC:DD:EE:FF", None, None).unwrap();

        DeviceRepository::assign_user(&conn, device.id, Some(user.id)).unwrap();

        let updated = DeviceRepository::get_by_id(&conn, device.id).unwrap();
        assert_eq!(updated.user_id, Some(user.id));
    }
}
