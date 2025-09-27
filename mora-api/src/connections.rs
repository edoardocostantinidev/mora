use std::collections::HashMap;

use serde::Serialize;

pub type ConnectionId = String;

const CONNECTION_TIMEOUT_IN_MSEC: i64 = 5000;

#[derive(Debug, Clone, Default)]
pub struct Connections {
    clients_connected: HashMap<ConnectionId, Client>,
}

impl Connections {
    pub fn clients_connected(&self) -> usize {
        self.clients_connected.len()
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct Client {
    ip: String,
    last_activity: i64,
}

impl Connections {
    pub fn add_client(&mut self, id: String, ip: String) {
        self.clients_connected.insert(
            format!("{id}-{ip}"),
            Client {
                ip,
                last_activity: chrono::Utc::now().timestamp_millis(),
            },
        );
    }

    pub fn purge_old_connections(&mut self) {
        self.clients_connected.retain(|_, client| {
            client.last_activity
                > chrono::Utc::now().timestamp_millis() - CONNECTION_TIMEOUT_IN_MSEC
        });
    }
}
