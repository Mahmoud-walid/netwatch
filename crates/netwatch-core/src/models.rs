use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Device {
    pub id: i64,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub hostname: Option<String>,
    pub display_name: Option<String>,
    pub user_id: Option<i64>,
    pub is_online: bool,
    pub first_seen: String,
    pub last_seen: String,
    pub last_rx_bytes: u64,
    pub last_tx_bytes: u64,
}
