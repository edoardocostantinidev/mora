use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectionsInfo {
    pub clients_connected: u64,
}
