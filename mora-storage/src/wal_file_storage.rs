use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    hash::Hash,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use log::info;
use mora_core::{
    result::{MoraError, MoraResult},
    traits::storage::Storage,
};

pub struct WalFileStorage<ContainerId: Eq + Hash> {
    wals: HashMap<ContainerId, File>,
}

impl<ContainerId: Eq + Hash> WalFileStorage<ContainerId> {
    pub fn new() -> Self {
        Self {
            wals: HashMap::new(),
        }
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
}

impl<ContainerId: Eq + Hash> Storage for WalFileStorage<ContainerId> {
    type ContainerId = ContainerId;

    type SortKey = String;

    type Item = Vec<u8>;

    fn load() -> MoraResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn create_container(&mut self, container_id: Self::ContainerId) -> MoraResult<()> {
        todo!()
    }

    fn delete_container(&mut self, container_id: Self::ContainerId) -> MoraResult<()> {
        todo!()
    }

    fn list_containers(&self) -> MoraResult<Vec<Self::ContainerId>> {
        todo!()
    }

    fn get_item(
        &self,
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
    ) -> MoraResult<Option<Self::Item>> {
        todo!()
    }

    fn get_items_range(
        &self,
        container_id: Self::ContainerId,
        start_key: Self::SortKey,
        end_key: Self::SortKey,
    ) -> MoraResult<Vec<Self::Item>> {
        todo!()
    }

    fn get_n_items(
        &self,
        container_id: Self::ContainerId,
        n: usize,
    ) -> MoraResult<Vec<Self::Item>> {
        todo!()
    }

    fn delete_item(
        &mut self,
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
    ) -> MoraResult<()> {
        todo!()
    }

    fn delete_items_range(
        &mut self,
        container_id: Self::ContainerId,
        start_key: Self::SortKey,
        end_key: Self::SortKey,
    ) -> MoraResult<()> {
        todo!()
    }

    fn store_item(
        &mut self,
        container_id: Self::ContainerId,
        sort_key: Self::SortKey,
        item: Self::Item,
    ) -> MoraResult<()> {
        todo!()
    }
}
