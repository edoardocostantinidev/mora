use std::collections::HashMap;

use mora_core::{
    result::{MoraError, MoraResult},
    traits::storage::Storage,
};
use regex::Regex;

use crate::temporal_queue::TemporalQueue;

type Bytes = Vec<u8>;
type QueueId = String;
type EventId = [u8; 16];

pub struct QueuePool<T: Storage<ContainerId = QueueId, SortKey = EventId, Item = Bytes>> {
    queues: HashMap<QueueId, TemporalQueue<Bytes>>,
    storage: T,
}

impl<T: Storage<ContainerId = QueueId, SortKey = EventId, Item = Bytes>> QueuePool<T> {
    pub async fn new() -> MoraResult<Self> {
        let storage = T::load()?;
        let mut pool = Self {
            queues: HashMap::default(),
            storage,
        };

        let containers = pool.storage.list_containers()?;
        for container in containers {
            pool.queues
                .insert(container.to_owned(), TemporalQueue::default());
            for (key, item) in pool.storage.get_all_items(&container)? {
                pool.get_queue_mut(&container)?
                    .enqueue(u128::from_le_bytes(key), item)?;
            }
        }

        Ok(pool)
    }

    pub fn create_queue(&mut self, id: QueueId) -> MoraResult<()> {
        if self.queues.contains_key(&id) {
            return Err(MoraError::QueueAlreadyExists(id));
        }

        self.storage.create_container(&id)?;
        self.queues.insert(id, TemporalQueue::default());

        Ok(())
    }

    pub fn delete_queue(&mut self, id: QueueId) -> MoraResult<QueueId> {
        self.storage.delete_container(&id)?;

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

    pub fn enqueue(&mut self, id: &QueueId, timestamp: u128, value: Bytes) -> MoraResult<()> {
        self.storage
            .store_item(&id, &timestamp.to_le_bytes(), &value)?;
        self.get_queue_mut(id)?.enqueue(timestamp, value)?;
        Ok(())
    }

    pub fn dequeue_until(
        &mut self,
        id: &QueueId,
        timestamp: u128,
        delete: bool,
    ) -> MoraResult<Vec<Bytes>> {
        Ok(self.get_queue_mut(id)?.dequeue_until(timestamp, delete))
    }
}
