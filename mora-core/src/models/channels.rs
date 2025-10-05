use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BufferOptions {
    pub time: u128,
    pub size: usize,
}

#[derive(Deserialize)]
pub struct CreateChannelRequest {
    pub queues: Vec<String>,
    pub buffer_options: BufferOptions,
}

#[derive(Serialize)]
pub struct CreateChannelResponse {
    pub channel_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct ListChannelsResponse {
    pub channels: Vec<Channel>,
}

pub type GetChannelResponse = Channel;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Channel {
    pub channel_id: String,
    pub queues: Vec<String>,
    pub buffer_options: BufferOptions,
    pub msec_from_last_op: usize,
}

#[derive(Serialize)]
pub struct Event {
    pub timestamp: u128,
    pub queue_name: String,
    pub data: String,
}

#[derive(Serialize)]
pub struct GetChannelEventsResponse {
    pub events: Vec<Event>,
}
