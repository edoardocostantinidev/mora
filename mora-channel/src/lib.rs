use mora_core::result::MoraError;
use std::collections::HashMap;

#[derive(Default, Debug, Clone)]
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    id: String,
    queues: Vec<String>,
    buffer_size: usize,
    buffer_time: usize,
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
    pub fn buffer_time(&self) -> usize {
        self.buffer_time
    }
}

impl ChannelManager {
    pub fn create_channel(
        &mut self,
        queues: Vec<String>,
        buffer_size: usize,
        buffer_time: usize,
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
        };
        self.channels.insert(channel_id, channel.clone());
        Ok(channel)
    }

    pub fn get_channels(&self) -> Result<Vec<&Channel>, MoraError> {
        Ok(self.channels.values().collect())
    }

    pub fn get_channel(&self, channel_id: &String) -> Result<Option<&Channel>, MoraError> {
        Ok(self.channels.get(channel_id))
    }

    pub fn close_channel(&mut self, channel_id: &String) -> Result<(), MoraError> {
        self.channels.remove(channel_id);
        Ok(())
    }
}
