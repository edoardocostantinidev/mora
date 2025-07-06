use log::info;
use mora_core::result::MoraError;
use std::collections::HashMap;

#[derive(Debug)]
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

impl ChannelManager {
    pub fn new() -> Self {
        Self {
            channels: HashMap::<_, _>::default(),
        }
    }
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

    pub fn queues(&mut self) -> &Vec<String> {
        self.msec_from_last_op = 0;
        info!("Zeroed msec from last op for {}", self.id);
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
}

impl ChannelManager {
    pub fn create_channel(
        &mut self,
        queues: Vec<String>,
        buffer_size: usize,
        buffer_time: u128,
    ) -> Result<Channel, MoraError> {
        let mut channel_id = uuid::Uuid::new_v4().to_string();

        while self.channels.contains_key(&channel_id) {
            channel_id = uuid::Uuid::new_v4().to_string();
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
