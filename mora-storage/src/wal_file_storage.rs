use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufWriter, Read, Seek, Write},
    path::Path,
};

use mora_core::{
    result::{MoraError, MoraResult, StorageError},
    traits::storage::Storage,
};

pub struct WalFileStorage {
    wals: HashMap<String, BufWriter<File>>,
    wal_path: String,
}

pub struct WalFileStorageConfig {
    wal_path: String,
}

impl WalFileStorageConfig {
    pub fn load() -> MoraResult<Self> {
        let wal_path = std::env::var("MORA_WAL_PATH").unwrap_or_else(|_| "/tmp/wals".to_string());
        Ok(Self { wal_path })
    }
}

enum ItemDescriptor {
    Tombstone = 0,
    Item = 1,
}

impl Into<u8> for ItemDescriptor {
    fn into(self) -> u8 {
        self as u8
    }
}

impl From<&[u8]> for ItemDescriptor {
    fn from(value: &[u8]) -> Self {
        match value[0] {
            0 => Self::Tombstone,
            1 => Self::Item,
            _ => panic!("invalid item descriptor"),
        }
    }
}

impl WalFileStorage {
    pub fn new(wal_path: String) -> Self {
        Self {
            wals: HashMap::new(),
            wal_path,
        }
    }
}

const SORT_KEY_BYTES: usize = 16;
const ITEM_DESCRIPTOR_BYTES: usize = 1;
const ITEM_LENGTH_BYTES: usize = 8;

/// WAL file storage design notes
///
/// File layout per container:
///
///   wal_path/<container_id>.wal
///        ┌──────────────────────────────────────────────────────────────┐
///        │ Record 1 │ Record 2 │ ... │ Record N                         │
///        └──────────────────────────────────────────────────────────────┘
///
/// Each record is a framed entry containing the sort key and payload:
/// Item:
///        ┌────────────┬───────────────────────┬──────────────────┬─────────────────┐
///        │ key (16B)  │ item_descriptor (1B)  │ item_length (4B) │ item (variable) │
///        └────────────┴───────────────────────┴──────────────────┴─────────────────┘
///
/// Tombstone:
///        ┌────────────┬──────────────────────┐
///        │ key (16B)  │ item_descriptor (1B) │
///        └────────────┴──────────────────────┘
impl Storage for WalFileStorage {
    type ContainerId = String;

    type SortKey = u128;

    type Item = Vec<u8>;

    // load()
    // Initialize storage from disk.
    //
    //   Startup -> load()
    //                │
    //                ▼
    //        scan wal_path for files
    //                │
    //                ▼
    //      open file handles -> populate self.wals -> Ok(self)
    fn load() -> MoraResult<Self>
    where
        Self: Sized,
    {
        let config = WalFileStorageConfig::load()?;
        let mut storage = Self::new(config.wal_path.to_owned());

        if !Path::new(&config.wal_path).exists() {
            std::fs::create_dir_all(&config.wal_path).map_err(|e| {
                MoraError::StorageError(StorageError::DirectoryCreationFailed(
                    config.wal_path.to_string(),
                    e.to_string(),
                ))
            })?;
        } else {
            // List all wal files in the wal_path directory
            let mut wal_files = std::fs::read_dir(config.wal_path).map_err(|e| {
                MoraError::StorageError(StorageError::DirectoryReadFailed(e.to_string()))
            })?;

            while let Some(Ok(wal_file)) = wal_files.next() {
                let file_handle = OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(wal_file.path())
                    .map_err(|e| {
                        MoraError::StorageError(StorageError::FileReadFailed(e.to_string()))
                    })?;
                storage.wals.insert(
                    wal_file
                        .file_name()
                        .to_string_lossy()
                        .to_string()
                        .replace(".wal", ""),
                    BufWriter::new(file_handle),
                );
            }
        }

        Ok(storage)
    }

    // create_container(&container_id)
    // Create an empty WAL file for the container if it doesn't exist.
    //
    //   create_container(id)
    //        │
    //        ▼
    //   wal_path directory exists? ── yes ─▶ go through the flow
    //        │ no
    //        ▼
    //   create directory -> create file -> insert handle in self.wals -> Ok(())
    //        │
    //        ▼
    //   wal_path/id.wal exists? ── yes ─▶ Err(MoraError::StorageError(ContainerAlreadyExists(id)))
    //        │ no
    //        ▼
    //   create file -> insert handle in self.wals -> Ok(())
    fn create_container(&mut self, container_id: &Self::ContainerId) -> MoraResult<()> {
        if self.wals.contains_key(container_id) {
            return Err(MoraError::StorageError(
                StorageError::ContainerAlreadyExists(container_id.to_string()),
            ));
        }

        if Path::new(&format!("{}/{}", self.wal_path, container_id)).exists() {
            return Err(MoraError::StorageError(
                StorageError::ContainerAlreadyExists(container_id.to_string()),
            ));
        }

        let file = File::create(format!("{}/{}", self.wal_path, container_id)).map_err(|e| {
            MoraError::StorageError(StorageError::ContainerCreationFailed(e.to_string()))
        })?;

        self.wals.insert(container_id.clone(), BufWriter::new(file));
        Ok(())
    }

    // delete_container(&container_id)
    // Remove container WAL (logically: close handle, delete file).
    //
    //   delete_container(id)
    //        │
    //        ▼
    //   close handle (if open)
    //        │
    //        ▼
    //   fs remove wal_path/id.wal -> Ok(())
    fn delete_container(&mut self, container_id: &Self::ContainerId) -> MoraResult<()> {
        let file = self
            .wals
            .get_mut(container_id)
            .ok_or(MoraError::StorageError(StorageError::ContainerNotFound(
                container_id.to_string(),
            )))?;
        file.flush().map_err(|e| {
            MoraError::StorageError(StorageError::ContainerDeletionFailed(e.to_string()))
        })?;

        std::fs::remove_file(format!("{}/{}", self.wal_path, container_id)).map_err(|e| {
            MoraError::StorageError(StorageError::ContainerDeletionFailed(e.to_string()))
        })?;

        self.wals.remove(container_id);
        Ok(())
    }

    // list_containers()
    // List known containers (by scanning directory or tracking opened handles).
    //
    //   list_containers()
    //        │
    //        ▼
    //   scan wal_path/*.wal -> collect ids -> return slice/collection
    fn list_containers(&self) -> MoraResult<Vec<Self::ContainerId>> {
        Ok(self.wals.keys().cloned().collect())
    }

    // delete_item(&container_id, &sort_key)
    // Append a tombstone record for the key.
    //
    //   delete_item(id, k)
    //        │
    //        ▼
    //   seek end -> write [key_len|key|item_len=0|<tombstone-flag>]
    //        │
    //        ▼
    //   flush -> Ok(())
    fn delete_item(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_key: &Self::SortKey,
    ) -> MoraResult<()> {
        let file_buffer = self
            .wals
            .get_mut(container_id)
            .ok_or(MoraError::StorageError(StorageError::ContainerNotFound(
                container_id.to_string(),
            )))?
            .get_mut();

        let mut buffer = Vec::with_capacity(SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES);
        insert_delete_item_op_to_buffer(&mut buffer, *item_sort_key);
        file_buffer
            .write_all(&buffer)
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        file_buffer
            .flush()
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        Ok(())
    }

    // store_item(&container_id, &sort_key, &item)
    // Enqueue by appending a framed record at EOF.
    //
    //   store_item(id, k, v)
    //        │
    //        ▼
    //   ensure wal file exists -> open handle
    //        │
    //        ▼
    //   append [key_len|key|item_len|item]
    //        │
    //        ▼
    //   flush/fsync -> Ok(())
    fn store_item(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_key: &Self::SortKey,
        item: &Self::Item,
    ) -> MoraResult<()> {
        let file_buffer = self
            .wals
            .get_mut(container_id)
            .ok_or(MoraError::StorageError(StorageError::ContainerNotFound(
                container_id.to_string(),
            )))?
            .get_mut();

        let mut buffer = Vec::new();
        insert_add_item_op_to_buffer(&mut buffer, *item_sort_key, item);

        file_buffer
            .seek(std::io::SeekFrom::End(0))
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        file_buffer
            .write_all(&buffer)
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        file_buffer
            .flush()
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        Ok(())
    }

    fn get_all_items(
        &mut self,
        container_id: &Self::ContainerId,
    ) -> MoraResult<HashMap<Self::SortKey, Self::Item>> {
        let file_buffer = self
            .wals
            .get_mut(container_id)
            .ok_or(MoraError::StorageError(StorageError::ContainerNotFound(
                container_id.to_string(),
            )))?
            .get_mut();

        let mut items = HashMap::new();
        let mut buffer = [0_u8; SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES];
        let mut offset = 0;
        while let Ok(_) = file_buffer
            .seek(std::io::SeekFrom::Start(offset))
            .and_then(|_| file_buffer.read_exact(&mut buffer))
        {
            // read header frame into separate variables
            let sort_key = &buffer[..SORT_KEY_BYTES];
            let mut sort_key_bytes_buf = [0; SORT_KEY_BYTES];
            sort_key_bytes_buf.copy_from_slice(&sort_key);
            let sort_key_u128 = u128::from_le_bytes(sort_key_bytes_buf);
            dbg!("sort_key_u128: {:?}", sort_key_u128);

            let item_descriptor_bytes =
                &buffer[SORT_KEY_BYTES..SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES];
            dbg!("item_descriptor_bytes: {:?}", item_descriptor_bytes);
            offset += buffer.len() as u64;

            match Into::<ItemDescriptor>::into(item_descriptor_bytes) {
                ItemDescriptor::Item => {
                    let mut item_length_buffer = [0_u8; ITEM_LENGTH_BYTES];
                    file_buffer
                        .seek(std::io::SeekFrom::Start(offset))
                        .and_then(|_| file_buffer.read_exact(&mut item_length_buffer))
                        .map_err(|e| {
                            MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                        })?;
                    offset += item_length_buffer.len() as u64;
                    let item_length =
                        u64::from_le_bytes(item_length_buffer.try_into().map_err(|_| {
                            MoraError::StorageError(StorageError::ItemReadFailed(
                                "invalid item length".to_string(),
                            ))
                        })?);
                    dbg!("item_length: {:?}", item_length);
                    // read item into separate buffer
                    let mut item_buffer = vec![0_u8; item_length as usize];
                    file_buffer
                        .seek(std::io::SeekFrom::Start(offset))
                        .and_then(|_| file_buffer.read_exact(&mut item_buffer))
                        .map_err(|e| {
                            MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                        })?;
                    offset += item_buffer.len() as u64;
                    dbg!("item_buffer: {:?}", &item_buffer);
                    items.insert(sort_key_u128, item_buffer);
                }
                ItemDescriptor::Tombstone => {
                    items.remove(&sort_key_u128);
                }
            }
        }

        Ok(items)
    }

    fn delete_items(
        &mut self,
        container_id: &Self::ContainerId,
        item_sort_keys: &[Self::SortKey],
    ) -> MoraResult<()> {
        let file_buffer = self
            .wals
            .get_mut(container_id)
            .ok_or(MoraError::StorageError(StorageError::ContainerNotFound(
                container_id.to_string(),
            )))?
            .get_mut();

        let mut buffer = Vec::new();
        item_sort_keys
            .iter()
            .for_each(|key| insert_delete_item_op_to_buffer(&mut buffer, *key));

        file_buffer
            .write_all(&buffer)
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        file_buffer
            .flush()
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        Ok(())
    }
}

fn insert_delete_item_op_to_buffer(buffer: &mut Vec<u8>, key: u128) {
    buffer.extend_from_slice(&key.to_le_bytes());
    buffer.push(ItemDescriptor::Tombstone as u8);
}

fn insert_add_item_op_to_buffer(buffer: &mut Vec<u8>, key: u128, item: &[u8]) {
    buffer.extend_from_slice(&key.to_le_bytes());
    buffer.push(ItemDescriptor::Item.into());
    buffer.extend_from_slice(&item.len().to_le_bytes());
    buffer.extend_from_slice(item);
}
