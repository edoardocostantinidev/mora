use log::info;
use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_file, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use mora_core::{
    result::{MoraError, MoraResult},
    traits::storage::Storage,
};
use regex::Regex;

use crate::temporal_queue::TemporalQueue;

type Bytes = Vec<u8>;
type QueueId = String;
type EventId = String;

pub struct QueuePool<T: Storage<ContainerId = QueueId, SortKey = EventId, Item = Bytes>> {
    queues: HashMap<QueueId, TemporalQueue<Bytes>>,
    storage: T,
    _capacity: usize,
}

impl<T: Storage<ContainerId = QueueId, SortKey = EventId, Item = Bytes>> QueuePool<T> {
    pub async fn new(capacity: usize) -> MoraResult<Self> {
        let storage = T::load()?;

        Ok(Self {
            queues: HashMap::default(),
            storage,
            _capacity: capacity,
        })
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

    pub fn enqueue(&mut self, id: &QueueId, timestamp: u128, value: Bytes) -> MoraResult<()> {
        self.enqueue_to_wal(id, timestamp, &value)?;
        Ok(self.get_queue_mut(id)?.enqueue(timestamp, value)?)
    }

    fn enqueue_to_wal(&mut self, id: &QueueId, timestamp: u128, value: &Bytes) -> MoraResult<()> {
        let mut buf = vec![];
        buf.extend_from_slice(&timestamp.to_le_bytes());
        buf.extend_from_slice(&value);
        buf.push(b'\n');

        let wal = self.wals.get_mut(id).expect("Wal file not found");
        wal.write_all(&buf)
            .map_err(|e| MoraError::FileError(format!("Failed to write to wal file: {e}")))?;
        wal.flush()
            .map_err(|e| MoraError::FileError(format!("Failed to flush wal file: {e}")))?;
        Ok(())
    }

    pub fn dequeue_until(&mut self, id: &QueueId, timestamp: u128) -> MoraResult<Vec<Bytes>> {
        Ok(self.get_queue_mut(id)?.dequeue_until(timestamp))
    }
}
