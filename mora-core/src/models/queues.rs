use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct ListQueuesResponse {
    pub queues: Vec<GetQueueResponse>,
}

pub type GetQueueResponse = Queue;

#[derive(Debug, Deserialize)]
pub struct PostQueueRequest {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Queue {
    pub id: String,
    pub pending_events_count: u128,
}
