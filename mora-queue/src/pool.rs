use std::collections::HashMap;

use mora_core::result::{MoraError, MoraResult};
use regex::Regex;

use crate::temporal_queue::TemporalQueue;

type Bytes = Vec<u8>;
type QueueId = String;

#[derive(Debug, Clone)]
pub struct QueuePool {
    queues: HashMap<QueueId, TemporalQueue<Bytes>>,
    _capacity: usize,
}

impl QueuePool {
    pub fn new(capacity: usize) -> Self {
        Self {
            queues: HashMap::default(),
            _capacity: capacity,
        }
    }

    pub fn create_queue(&mut self, id: QueueId) -> MoraResult<()> {
        if self.queues.contains_key(&id) {
            return Err(MoraError::QueueAlreadyExists(id));
        }

        self.queues.insert(id, TemporalQueue::default());

        Ok(())
    }

    pub fn delete_queue(&mut self, id: QueueId) -> MoraResult<QueueId> {
        self.queues
            .remove(&id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))
            .map(|_| id)
    }

    pub fn get_queue(&self, id: &QueueId) -> MoraResult<&TemporalQueue<Bytes>> {
        self.queues
            .get(id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))
    }

    pub fn get_queue_mut(&mut self, id: &QueueId) -> MoraResult<&mut TemporalQueue<Bytes>> {
        self.queues
            .get_mut(id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))
    }

    pub fn get_queues(&self, pattern: Regex) -> MoraResult<Vec<(String, &TemporalQueue<Bytes>)>> {
        Ok(self
            .queues
            .keys()
            .filter(|k| pattern.is_match(k))
            .map(|k| (k.to_owned(), self.queues.get(k).unwrap()))
            .collect())
    }

    pub fn get_queues_mut(
        &mut self,
        pattern: Regex,
    ) -> MoraResult<Vec<(String, &mut TemporalQueue<Bytes>)>> {
        let mut queues = vec![];
        for (k, queue) in self.queues.iter_mut() {
            if pattern.is_match(k) {
                queues.push((k.to_owned(), queue));
            }
        }
        Ok(queues)
    }
}
