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
const ITEM_LENGTH_BYTES: usize = 4;

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

    type SortKey = [u8; SORT_KEY_BYTES];

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

        file_buffer
            .write_all(
                &[
                    item_sort_key.to_vec(),
                    vec![ItemDescriptor::Tombstone as u8],
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<u8>>(),
            )
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

        let to_be_written = [
            item_sort_key.to_vec(),
            vec![ItemDescriptor::Item as u8],
            item.len().to_le_bytes()[..ITEM_LENGTH_BYTES].to_vec(),
            item.to_vec(),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>();

        file_buffer
            .seek(std::io::SeekFrom::End(0))
            .map_err(|e| MoraError::StorageError(StorageError::ItemWriteFailed(e.to_string())))?;

        file_buffer
            .write_all(&to_be_written)
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
        let mut buffer = [0_u8; SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES + ITEM_LENGTH_BYTES];
        while let Ok(_) = file_buffer.read_exact(&mut buffer) {
            // read header frame into separate variables
            let sort_key = &buffer[..SORT_KEY_BYTES];
            let item_descriptor_bytes =
                &buffer[SORT_KEY_BYTES..SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES];

            match Into::<ItemDescriptor>::into(item_descriptor_bytes) {
                ItemDescriptor::Item => {
                    let item_length_bytes = &buffer[SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES
                        ..SORT_KEY_BYTES + ITEM_DESCRIPTOR_BYTES + ITEM_LENGTH_BYTES];
                    let item_length =
                        u32::from_le_bytes(item_length_bytes.try_into().map_err(|_| {
                            MoraError::StorageError(StorageError::ItemReadFailed(
                                "invalid item length".to_string(),
                            ))
                        })?);
                    // read item into separate buffer
                    let mut item_buffer = vec![0_u8; item_length as usize];
                    file_buffer.read_exact(&mut item_buffer).map_err(|e| {
                        MoraError::StorageError(StorageError::ItemReadFailed(e.to_string()))
                    })?;
                    let mut sort_key_bytes = [0; SORT_KEY_BYTES];
                    sort_key_bytes.copy_from_slice(&sort_key);

                    items.insert(sort_key_bytes, item_buffer);
                }
                ItemDescriptor::Tombstone => {
                    items.remove(sort_key);
                }
            }
        }

        Ok(items)
    }
}
