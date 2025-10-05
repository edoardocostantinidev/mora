use mora_core::{result::MoraError, traits::storage::Storage};
use std::collections::HashMap;

use crate::pool::{Bytes, EventId, QueueId, QueuePool};

#[derive(Default)]
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    id: String,
    queues: Vec<String>,
    buffer_size: usize,
    buffer_time: u128,
    msec_from_last_op: usize,
}

impl Channel {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn queues(&self) -> &Vec<String> {
        &self.queues
    }

    pub fn buffer_size(&self) -> usize {
        self.buffer_size
    }
    pub fn buffer_time(&self) -> u128 {
        self.buffer_time
    }

    pub fn msec_from_last_op(&self) -> usize {
        self.msec_from_last_op
    }

    pub fn update_msec_from_last_op(&mut self, msec: usize) {
        self.msec_from_last_op += msec;
    }

    pub fn reset_msec_from_last_op(&mut self) {
        self.msec_from_last_op = 0;
    }
}

impl ChannelManager {
    pub fn create_channel<T: Storage<ContainerId = QueueId, SortKey = EventId, Item = Bytes>>(
        &mut self,
        queue_pool: &QueuePool<T>,
        queues: Vec<String>,
        buffer_size: usize,
        buffer_time: u128,
    ) -> Result<Channel, MoraError> {
        let mut channel_id = uuid::Uuid::new_v4().to_string();

        while self.channels.contains_key(&channel_id) {
            channel_id = uuid::Uuid::new_v4().to_string();
        }

        for queue in &queues {
            if !queue_pool.contains_queue(queue) {
                return Err(MoraError::QueueNotFound(queue.clone()));
            }
        }

        let channel = Channel {
            id: channel_id.clone(),
            queues,
            buffer_size,
            buffer_time,
            msec_from_last_op: 0,
        };
        self.channels.insert(channel_id, channel.clone());
        Ok(channel)
    }

    pub fn get_channels(&self) -> Result<Vec<&Channel>, MoraError> {
        Ok(self.channels.values().collect())
    }

    pub fn get_mut_channels(&mut self) -> Result<Vec<&mut Channel>, MoraError> {
        Ok(self.channels.values_mut().collect())
    }

    pub fn get_channel(&self, channel_id: &String) -> Result<Option<&Channel>, MoraError> {
        Ok(self.channels.get(channel_id))
    }

    pub fn get_mut_channel(
        &mut self,
        channel_id: &String,
    ) -> Result<Option<&mut Channel>, MoraError> {
        Ok(self.channels.get_mut(channel_id))
    }

    pub fn close_channel(&mut self, channel_id: &String) {
        self.channels.remove(channel_id);
    }
}
