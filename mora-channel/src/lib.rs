use std::collections::HashMap;

use mora_core::result::MoraError;
use regex::Regex;

#[derive(Default, Debug, Clone)]
pub struct ChannelManager {
    channels: HashMap<String, Channel>,
}

#[derive(Debug, Clone)]
pub struct Channel {
    id: String,
    queue_filter: Regex,
}
impl Channel {
    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn queue_filter(&self) -> &Regex {
        &self.queue_filter
    }
}

impl ChannelManager {
    pub fn create_channel(&mut self, queue_filter: Regex) -> Result<Channel, MoraError> {
        let mut channel_id = uuid::Uuid::new_v4().to_string();

        while self.channels.contains_key(&channel_id) {
            channel_id = uuid::Uuid::new_v4().to_string();
        }

        let channel = Channel {
            id: channel_id.clone(),
            queue_filter,
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
