use log::info;
use std::{
    collections::HashMap,
    fs::{create_dir_all, remove_file, File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::PathBuf,
};

use mora_core::result::{MoraError, MoraResult};
use regex::Regex;

use crate::temporal_queue::TemporalQueue;

type Bytes = Vec<u8>;
type QueueId = String;

#[derive(Debug)]
pub struct QueuePool {
    queues: HashMap<QueueId, TemporalQueue<Bytes>>,
    wals: HashMap<QueueId, File>,
    _capacity: usize,
}

impl QueuePool {
    pub fn new(capacity: usize) -> MoraResult<Self> {
        let mut pool = Self {
            queues: HashMap::default(),
            wals: HashMap::default(),
            _capacity: capacity,
        };

        pool.restore_from_wals()?;

        Ok(pool)
    }

    fn restore_from_wals(&mut self) -> MoraResult<()> {
        if !std::path::Path::new("wals").exists() {
            info!("No wals directory found, skipping restore, creating a new one");
            create_dir_all("wals").map_err(|e| {
                MoraError::GenericError(format!("Failed to create wals directory: {e}"))
            })?;
            return Ok(());
        }

        let wal_files = std::fs::read_dir("wals/")
            .map_err(|e| MoraError::GenericError(format!("Failed to read wals directory: {e}")))?;

        for wal_file in wal_files {
            match wal_file {
                Ok(wal) => {
                    self.restore_wal(wal.path())?;
                }
                Err(e) => {
                    return Err(MoraError::GenericError(e.to_string()));
                }
            }
        }

        Ok(())
    }

    fn restore_wal(&mut self, wal_file: PathBuf) -> MoraResult<()> {
        let id = wal_file
            .file_name()
            .ok_or(MoraError::GenericError(
                "Failed to get file name".to_string(),
            ))?
            .to_string_lossy()
            .split(".")
            .next()
            .ok_or(MoraError::GenericError(
                "Failed to get file name".to_string(),
            ))?
            .to_string();

        let file = OpenOptions::new()
            .create(false)
            .read(true)
            .write(false)
            .append(false)
            .truncate(false)
            .open(wal_file.clone())
            .map_err(|e| MoraError::FileError(format!("Failed to open wal file: {e}")))?;

        self.queues.insert(id.clone(), TemporalQueue::default());
        let mut bufreader = BufReader::new(file);
        let mut buf = vec![];

        while let Ok(bytes_read) = bufreader.read_until(b'\n', &mut buf) {
            if bytes_read == 0 {
                info!("Empty file, skipping");
                return Ok(());
            }

            if bytes_read < 16 {
                return Err(MoraError::GenericError(
                    "Invalid non-empty WAL file ".to_string(),
                ));
            }

            let timestamp_bytes: &[u8; 16] = buf[0..16].try_into().unwrap(); // safe to unwrap because we know the length of the buffer
            let timestamp = u128::from_le_bytes(*timestamp_bytes);
            self.queues
                .get_mut(&id)
                .expect("Queue not found")
                .enqueue(timestamp, buf[16..buf.len() - 1].to_owned())?;

            buf.clear();
        }

        drop(bufreader);

        let file = OpenOptions::new()
            .create(false)
            .read(true)
            .write(true)
            .append(true)
            .truncate(false)
            .open(wal_file)
            .map_err(|e| MoraError::FileError(format!("Failed to open wal file: {e}")))?;

        self.wals.insert(id.clone(), file);

        Ok(())
    }

    pub fn create_queue(&mut self, id: QueueId) -> MoraResult<()> {
        if self.queues.contains_key(&id) {
            return Err(MoraError::QueueAlreadyExists(id));
        }

        let path = format!("wals/{id}.wal");
        match File::create_new(path) {
            Ok(file) => {
                self.wals.insert(id.clone(), file);
                self.queues.insert(id, TemporalQueue::default());
            }
            Err(e) => {
                return Err(MoraError::GenericError(e.to_string()));
            }
        }

        Ok(())
    }

    pub fn delete_queue(&mut self, id: QueueId) -> MoraResult<QueueId> {
        remove_file(format!("wals/{id}.wal"))
            .map_err(|e| MoraError::FileError(format!("Failed to delete wal file: {e}")))?;
        self.wals
            .remove(&id)
            .ok_or(MoraError::QueueNotFound(id.to_string()))?;

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
