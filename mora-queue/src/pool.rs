use std::collections::HashMap;

use mora_core::result::{MoraError, MoraResult};
use regex::Regex;

use crate::temporal_queue::TemporalQueue;

type Bytes = Vec<u8>;
type QueueId<'a> = &'a str;
pub struct QueuePool<'a> {
    queues: HashMap<QueueId<'a>, TemporalQueue<Bytes>>,
    capacity: usize,
}

impl<'a> QueuePool<'a> {
    pub fn create_queue(&mut self, id: QueueId<'a>) -> MoraResult<()> {
        if self.queues.contains_key(id) {
            return Err(MoraError::QueueAlreadyExists(id.to_owned()));
        }

        self.queues.insert(id, TemporalQueue::default());
        self.capacity += 1;
        Ok(())
    }

    pub fn delete_queue(&mut self, id: QueueId<'a>) -> MoraResult<QueueId> {
        self.queues
            .remove(&id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))
            .map(|_| id)
    }

    pub fn get_queue(&self, id: QueueId<'a>) -> MoraResult<&TemporalQueue<Bytes>> {
        self.queues
            .get(&id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))
    }

    pub fn get_queue_mut(&mut self, id: QueueId<'a>) -> MoraResult<&mut TemporalQueue<Bytes>> {
        self.queues
            .get_mut(&id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))
    }

    pub fn get_queues(&self, pattern: Regex) -> MoraResult<Vec<&TemporalQueue<Bytes>>> {
        Ok(self
            .queues
            .keys()
            .filter(|k| pattern.is_match(k))
            .filter_map(|k| self.queues.get(k))
            .collect())
    }

    pub fn get_queues_mut(&mut self, pattern: Regex) -> MoraResult<Vec<&mut TemporalQueue<Bytes>>> {
        let mut queues = vec![];
        for (k, queue) in self.queues.iter_mut() {
            if pattern.is_match(k) {
                queues.push(queue);
            }
        }
        Ok(queues)
    }
}
