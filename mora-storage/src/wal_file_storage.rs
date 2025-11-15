use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufReader, BufWriter, Read, Write},
    path::PathBuf,
};

use log::{debug, info, warn};
use mora_core::{
    result::{MoraError, MoraResult, StorageError},
    traits::storage::Storage,
};

/// WAL-based file storage with in-memory index for fast reads.
///
/// Design Philosophy:
/// - Write-Ahead Log ensures durability via append-only writes with fsync
/// - In-memory HashMap provides O(1) reads without scanning disk
/// - Background compaction removes tombstones and reclaims space
/// - CRC32 checksums ensure data integrity
///
/// Architecture:
///                ┌──────────────────┐
///                │  WalFileStorage  │
///                └────────┬─────────┘
///                         │
///         ┌───────────────┴────────────────┐
///         ▼                                ▼
///  ┌─────────────┐                 ┌──────────────┐
///  │  In-Memory  │◄────replay──────│  WAL Files   │
///  │   Index     │                 │  (durable)   │
///  │ (fast read) │─────append─────►│              │
///  └─────────────┘                 └──────────────┘
pub struct WalFileStorage {
    /// In-memory index: container_id -> (sort_key -> item)
    /// Provides O(1) reads without disk access
    index: HashMap<String, HashMap<u128, Vec<u8>>>,

    /// Write handles: container_id -> BufWriter<File>
    /// Used for appending new records to WAL
    write_handles: HashMap<String, BufWriter<File>>,

    /// Base path for all WAL files
    wal_path: PathBuf,

    /// Statistics for monitoring and compaction decisions
    stats: WalStats,
}

#[derive(Debug, Default)]
struct WalStats {
    /// Total records written since load
    total_writes: u64,
    /// Total tombstones written since load
    total_tombstones: u64,
    /// Per-container record counts for compaction heuristics
    container_record_counts: HashMap<String, u64>,
}

impl WalStats {
    fn should_compact(&self, container_id: &str) -> bool {
        // Compact if tombstone ratio exceeds 30% and we have significant records
        if let Some(&record_count) = self.container_record_counts.get(container_id) {
            let tombstone_ratio = self.total_tombstones as f64 / record_count as f64;
            record_count > 1000 && tombstone_ratio > 0.3
        } else {
            false
        }
    }
}

pub struct WalFileStorageConfig {
    wal_path: PathBuf,
}

impl WalFileStorageConfig {
    pub fn load() -> MoraResult<Self> {
        let wal_path = std::env::var("MORA_WAL_PATH")
            .unwrap_or_else(|_| "/tmp/wals".to_string())
            .into();
        Ok(Self { wal_path })
    }
}

/// Record type discriminator
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum RecordType {
    Tombstone = 0,
    Item = 1,
}

impl RecordType {
    fn from_u8(value: u8) -> MoraResult<Self> {
        match value {
            0 => Ok(Self::Tombstone),
            1 => Ok(Self::Item),
            _ => Err(MoraError::StorageError(StorageError::CorruptedData(
                format!("Invalid record type: {}", value),
            ))),
        }
    }
}

// Record format constants
const SORT_KEY_BYTES: usize = 16;
const RECORD_TYPE_BYTES: usize = 1;
const ITEM_LENGTH_BYTES: usize = 8;
const CRC_BYTES: usize = 4;

/// WAL Record Format (with checksums):
///
/// Item Record:
///   ┌────────────┬──────────────┬──────────────┬─────────────────┬──────────┐
///   │ sort_key   │ record_type  │ item_length  │ item_data       │ crc32    │
///   │ (16 bytes) │ (1 byte)     │ (8 bytes)    │ (variable)      │ (4 bytes)│
///   └────────────┴──────────────┴──────────────┴─────────────────┴──────────┘
///
/// Tombstone Record:
///   ┌────────────┬──────────────┬──────────┐
///   │ sort_key   │ record_type  │ crc32    │
///   │ (16 bytes) │ (1 byte)     │ (4 bytes)│
///   └────────────┴──────────────┴──────────┘
///
/// The CRC32 covers everything except itself.

impl WalFileStorage {
    pub fn new(wal_path: PathBuf) -> Self {
        Self {
            index: HashMap::new(),
            write_handles: HashMap::new(),
            wal_path,
            stats: WalStats::default(),
        }
    }

    /// Get the file path for a container's WAL
    fn container_wal_path(&self, container_id: &str) -> PathBuf {
        self.wal_path.join(format!("{}.wal", container_id))
    }

    /// Replay a WAL file into the in-memory index
    fn replay_wal(&mut self, container_id: &str, file: &mut File) -> MoraResult<()> {
        info!("Replaying WAL for container: {}", container_id);

        let mut reader = BufReader::new(file);
        let container_index = self.index.entry(container_id.to_string()).or_default();
        let mut record_count = 0;

        loop {
            // Try to read the header
            let mut header_buf = [0u8; SORT_KEY_BYTES + RECORD_TYPE_BYTES];
            match reader.read_exact(&mut header_buf) {
                Ok(_) => {}
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                    // End of file - this is normal
                    break;
                }
                Err(e) => {
                    return Err(MoraError::StorageError(StorageError::ItemReadFailed(
                        e.to_string(),
                    )))
                }
            }

            let sort_key = u128::from_le_bytes(
                header_buf[..SORT_KEY_BYTES]
                    .try_into()
                    .map_err(|_| MoraError::StorageError(StorageError::CorruptedData(
                        "Invalid sort key".to_string()
                    )))?
            );

            let record_type = RecordType::from_u8(header_buf[SORT_KEY_BYTES])?;

            match record_type {
                RecordType::Item => {
                    // Read item length
                    let mut len_buf = [0u8; ITEM_LENGTH_BYTES];
                    reader.read_exact(&mut len_buf).map_err(|e| {
                        MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                    })?;
                    let item_length = u64::from_le_bytes(len_buf);

                    // Read item data
                    let mut item_data = vec![0u8; item_length as usize];
                    reader.read_exact(&mut item_data).map_err(|e| {
                        MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                    })?;

                    // Read and verify checksum
                    let mut crc_buf = [0u8; CRC_BYTES];
                    reader.read_exact(&mut crc_buf).map_err(|e| {
                        MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                    })?;
                    let stored_crc = u32::from_le_bytes(crc_buf);

                    // Compute expected checksum
                    let mut hasher = crc32fast::Hasher::new();
                    hasher.update(&header_buf);
                    hasher.update(&len_buf);
                    hasher.update(&item_data);
                    let computed_crc = hasher.finalize();

                    if stored_crc != computed_crc {
                        warn!(
                            "CRC mismatch for container {} key {}: expected {}, got {}",
                            container_id, sort_key, computed_crc, stored_crc
                        );
                        return Err(MoraError::StorageError(StorageError::CorruptedData(
                            format!("CRC mismatch for key {}", sort_key),
                        )));
                    }

                    container_index.insert(sort_key, item_data);
                    record_count += 1;
                }
                RecordType::Tombstone => {
                    // Read and verify checksum
                    let mut crc_buf = [0u8; CRC_BYTES];
                    reader.read_exact(&mut crc_buf).map_err(|e| {
                        MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                    })?;
                    let stored_crc = u32::from_le_bytes(crc_buf);

                    // Compute expected checksum
                    let mut hasher = crc32fast::Hasher::new();
                    hasher.update(&header_buf);
                    let computed_crc = hasher.finalize();

                    if stored_crc != computed_crc {
                        warn!(
                            "CRC mismatch for tombstone in container {} key {}",
                            container_id, sort_key
                        );
                        return Err(MoraError::StorageError(StorageError::CorruptedData(
                            format!("CRC mismatch for tombstone key {}", sort_key),
                        )));
                    }

                    container_index.remove(&sort_key);
                    record_count += 1;
                }
            }
        }

        self.stats
            .container_record_counts
            .insert(container_id.to_string(), record_count);

        info!(
            "Replayed {} records for container {}",
            record_count, container_id
        );
        Ok(())
    }

    /// Append a record to the WAL and sync to disk
    fn append_record(&mut self, container_id: &str, record: &[u8]) -> MoraResult<()> {
        let writer = self.write_handles.get_mut(container_id).ok_or_else(|| {
            MoraError::StorageError(StorageError::ContainerNotFound(container_id.to_string()))
        })?;

        writer.write_all(record).map_err(|e| {
            MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string()))
        })?;

        // Flush to OS buffer
        writer.flush().map_err(|e| {
            MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string()))
        })?;

        // Force to disk (durability!)
        writer.get_mut().sync_all().map_err(|e| {
            MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string()))
        })?;

        Ok(())
    }

    /// Encode an item record with checksum
    fn encode_item_record(sort_key: u128, item: &[u8]) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(
            SORT_KEY_BYTES + RECORD_TYPE_BYTES + ITEM_LENGTH_BYTES + item.len() + CRC_BYTES,
        );

        // Write header and data
        buffer.extend_from_slice(&sort_key.to_le_bytes());
        buffer.push(RecordType::Item as u8);
        buffer.extend_from_slice(&(item.len() as u64).to_le_bytes());
        buffer.extend_from_slice(item);

        // Compute and append checksum
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&buffer);
        let crc = hasher.finalize();
        buffer.extend_from_slice(&crc.to_le_bytes());

        buffer
    }

    /// Encode a tombstone record with checksum
    fn encode_tombstone_record(sort_key: u128) -> Vec<u8> {
        let mut buffer = Vec::with_capacity(SORT_KEY_BYTES + RECORD_TYPE_BYTES + CRC_BYTES);

        // Write header
        buffer.extend_from_slice(&sort_key.to_le_bytes());
        buffer.push(RecordType::Tombstone as u8);

        // Compute and append checksum
        let mut hasher = crc32fast::Hasher::new();
        hasher.update(&buffer);
        let crc = hasher.finalize();
        buffer.extend_from_slice(&crc.to_le_bytes());

        buffer
    }

    /// Compact a container's WAL by rewriting without tombstones
    pub fn compact_container(&mut self, container_id: &str) -> MoraResult<()> {
        info!("Compacting container: {}", container_id);

        // Get current data from index
        let items = self.index.get(container_id).ok_or_else(|| {
            MoraError::StorageError(StorageError::ContainerNotFound(container_id.to_string()))
        })?;

        // Write to temporary file
        let temp_path = self.wal_path.join(format!("{}.wal.tmp", container_id));
        let mut temp_file = BufWriter::new(
            File::create(&temp_path).map_err(|e| {
                MoraError::StorageError(StorageError::FileWriteFailed(e.to_string()))
            })?
        );

        // Write all current items
        for (&sort_key, item) in items.iter() {
            let record = Self::encode_item_record(sort_key, item);
            temp_file.write_all(&record).map_err(|e| {
                MoraError::StorageError(StorageError::FileWriteFailed(e.to_string()))
            })?;
        }

        // Flush and sync
        temp_file.flush().map_err(|e| {
            MoraError::StorageError(StorageError::FileWriteFailed(e.to_string()))
        })?;
        temp_file.get_mut().sync_all().map_err(|e| {
            MoraError::StorageError(StorageError::FileWriteFailed(e.to_string()))
        })?;
        drop(temp_file);

        // Close old handle
        self.write_handles.remove(container_id);

        // Atomically replace old file with new
        let wal_path = self.container_wal_path(container_id);
        std::fs::rename(&temp_path, &wal_path).map_err(|e| {
            MoraError::StorageError(StorageError::FileWriteFailed(e.to_string()))
        })?;

        // Reopen write handle
        let file = OpenOptions::new()
            .append(true)
            .open(&wal_path)
            .map_err(|e| {
                MoraError::StorageError(StorageError::FileWriteFailed(e.to_string()))
            })?;
        self.write_handles.insert(container_id.to_string(), BufWriter::new(file));

        // Reset stats
        self.stats
            .container_record_counts
            .insert(container_id.to_string(), items.len() as u64);

        info!(
            "Compaction complete for {}: {} items retained",
            container_id,
            items.len()
        );
        Ok(())
    }
}

impl Storage for WalFileStorage {
    type ContainerId = String;
    type SortKey = u128;
    type Item = Vec<u8>;

    fn load() -> MoraResult<Self>
    where
        Self: Sized,
    {
        let config = WalFileStorageConfig::load()?;
        let mut storage = Self::new(config.wal_path.clone());

        // Create directory if it doesn't exist
        if !config.wal_path.exists() {
            std::fs::create_dir_all(&config.wal_path).map_err(|e| {
                MoraError::StorageError(StorageError::DirectoryCreationFailed(
                    config.wal_path.display().to_string(),
                    e.to_string(),
                ))
            })?;
            info!("Created WAL directory: {}", config.wal_path.display());
        } else {
            // Replay existing WAL files
            let entries = std::fs::read_dir(&config.wal_path).map_err(|e| {
                MoraError::StorageError(StorageError::DirectoryReadFailed(e.to_string()))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    MoraError::StorageError(StorageError::DirectoryReadFailed(e.to_string()))
                })?;

                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) != Some("wal") {
                    continue;
                }

                let container_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| {
                        MoraError::StorageError(StorageError::CorruptedData(
                            format!("Invalid WAL filename: {}", path.display()),
                        ))
                    })?;

                // Open for reading and replay
                let mut file = OpenOptions::new()
                    .read(true)
                    .open(&path)
                    .map_err(|e| {
                        MoraError::StorageError(StorageError::FileReadFailed(e.to_string()))
                    })?;

                storage.replay_wal(container_id, &mut file)?;

                // Reopen for appending
                let file = OpenOptions::new()
                    .append(true)
                    .open(&path)
                    .map_err(|e| {
                        MoraError::StorageError(StorageError::FileReadFailed(e.to_string()))
                    })?;

                storage.write_handles.insert(
                    container_id.to_string(),
                    BufWriter::new(file),
                );
            }
        }

        Ok(storage)
    }

    fn create_container(&mut self, container_id: &Self::ContainerId) -> MoraResult<()> {
        if self.index.contains_key(container_id) {
            return Err(MoraError::StorageError(
                StorageError::ContainerAlreadyExists(container_id.clone()),
            ));
        }

        let wal_path = self.container_wal_path(container_id);
        if wal_path.exists() {
            return Err(MoraError::StorageError(
                StorageError::ContainerAlreadyExists(container_id.clone()),
            ));
        }

        // Create empty WAL file
        let file = File::create(&wal_path).map_err(|e| {
            MoraError::StorageError(StorageError::ContainerCreationFailed(e.to_string()))
        })?;

        // Initialize structures
        self.index.insert(container_id.clone(), HashMap::new());
        self.write_handles.insert(container_id.clone(), BufWriter::new(file));
        self.stats.container_record_counts.insert(container_id.clone(), 0);

        debug!("Created container: {}", container_id);
        Ok(())
    }

    fn delete_container(&mut self, container_id: &Self::ContainerId) -> MoraResult<()> {
        // Remove from memory
        self.index.remove(container_id);
        self.stats.container_record_counts.remove(container_id);

        // Close and remove file
        if let Some(mut writer) = self.write_handles.remove(container_id) {
            writer.flush().map_err(|e| {
                MoraError::StorageError(StorageError::ContainerDeletionFailed(e.to_string()))
            })?;
        }

        let wal_path = self.container_wal_path(container_id);
        std::fs::remove_file(&wal_path).map_err(|e| {
            MoraError::StorageError(StorageError::ContainerDeletionFailed(e.to_string()))
        })?;

        debug!("Deleted container: {}", container_id);
        Ok(())
    }

    fn list_containers(&self) -> MoraResult<Vec<Self::ContainerId>> {
        Ok(self.index.keys().cloned().collect())
    }

    fn delete_item(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_key: &Self::SortKey,
    ) -> MoraResult<()> {
        // Remove from index
        let container_index = self.index.get_mut(container_id).ok_or_else(|| {
            MoraError::StorageError(StorageError::ContainerNotFound(container_id.clone()))
        })?;
        container_index.remove(item_sort_key);

        // Append tombstone to WAL
        let record = Self::encode_tombstone_record(*item_sort_key);
        self.append_record(container_id, &record)?;

        self.stats.total_tombstones += 1;
        if let Some(count) = self.stats.container_record_counts.get_mut(container_id) {
            *count += 1;
        }

        // Check if compaction is needed
        if self.stats.should_compact(container_id) {
            debug!("Triggering compaction for container: {}", container_id);
            self.compact_container(container_id)?;
        }

        Ok(())
    }

    fn store_item(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_key: &Self::SortKey,
        item: &Self::Item,
    ) -> MoraResult<()> {
        // Update index
        let container_index = self.index.get_mut(container_id).ok_or_else(|| {
            MoraError::StorageError(StorageError::ContainerNotFound(container_id.clone()))
        })?;
        container_index.insert(*item_sort_key, item.clone());

        // Append to WAL
        let record = Self::encode_item_record(*item_sort_key, item);
        self.append_record(container_id, &record)?;

        self.stats.total_writes += 1;
        if let Some(count) = self.stats.container_record_counts.get_mut(container_id) {
            *count += 1;
        }

        Ok(())
    }

    fn store_items(
        &mut self,
        container_id: &Self::ContainerId,
        items: &[(Self::SortKey, Self::Item)],
    ) -> MoraResult<()> {
        if items.is_empty() {
            return Ok(());
        }

        // Update index
        let container_index = self.index.get_mut(container_id).ok_or_else(|| {
            MoraError::StorageError(StorageError::ContainerNotFound(container_id.clone()))
        })?;

        // Build batch of records
        let mut batch = Vec::new();
        for (sort_key, item) in items {
            container_index.insert(*sort_key, item.clone());
            let record = Self::encode_item_record(*sort_key, item);
            batch.extend_from_slice(&record);
        }

        // Single append + sync for entire batch
        self.append_record(container_id, &batch)?;

        self.stats.total_writes += items.len() as u64;
        if let Some(count) = self.stats.container_record_counts.get_mut(container_id) {
            *count += items.len() as u64;
        }

        Ok(())
    }

    fn get_all_items(
        &mut self,
        container_id: &Self::ContainerId,
    ) -> MoraResult<HashMap<Self::SortKey, Self::Item>> {
        // O(1) lookup from in-memory index!
        self.index
            .get(container_id)
            .cloned()
            .ok_or_else(|| {
                MoraError::StorageError(StorageError::ContainerNotFound(container_id.clone()))
            })
    }

    fn delete_items(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_keys: &[Self::SortKey],
    ) -> MoraResult<()> {
        if item_sort_keys.is_empty() {
            return Ok(());
        }

        // Update index
        let container_index = self.index.get_mut(container_id).ok_or_else(|| {
            MoraError::StorageError(StorageError::ContainerNotFound(container_id.clone()))
        })?;

        // Build batch of tombstones
        let mut batch = Vec::new();
        for sort_key in item_sort_keys {
            container_index.remove(sort_key);
            let record = Self::encode_tombstone_record(*sort_key);
            batch.extend_from_slice(&record);
        }

        // Single append + sync for entire batch
        self.append_record(container_id, &batch)?;

        self.stats.total_tombstones += item_sort_keys.len() as u64;
        if let Some(count) = self.stats.container_record_counts.get_mut(container_id) {
            *count += item_sort_keys.len() as u64;
        }

        // Check if compaction is needed
        if self.stats.should_compact(container_id) {
            debug!("Triggering compaction for container: {}", container_id);
            self.compact_container(container_id)?;
        }

        Ok(())
    }
}
