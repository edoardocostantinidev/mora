use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStatusData {
    pub version: String,
    pub current_time_in_ns: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ClusterStatus {
    Online(ClusterStatusData),
    Degraded(String),
    #[default]
    Offline,
}
