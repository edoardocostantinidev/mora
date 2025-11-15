use thiserror::Error;

pub type MoraResult<T> = Result<T, MoraError>;

#[derive(Debug, Error)]
pub enum MoraError {
    #[error("queue already exists: `{0}`")]
    QueueAlreadyExists(String),
    #[error("dequeue operation failed: `{0}`")]
    DequeueError(String),
    #[error("enqueue operation failed: `{0}`")]
    EnqueueError(String),
    #[error("error reading config: `{0}`")]
    ConfigError(String),
    #[error("error starting api layer: `{0}`")]
    ApiError(String),
    #[error("queue not found: `{0}`")]
    QueueNotFound(String),
    #[error("generic error: `{0}`")]
    GenericError(String),
    #[error("connection error: `{0}`")]
    ConnectionError(String),
    #[error("not connected to the server")]
    NotConnected,
    #[error("queue full")]
    QueueFull,
    #[error("file error: `{0}`")]
    FileError(String),
    #[error("storage error: `{0}`")]
    StorageError(StorageError),
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("container already exists: `{0}`")]
    ContainerAlreadyExists(String),
    #[error("container creation failed: `{0}`")]
    ContainerCreationFailed(String),
    #[error("container deletion failed: `{0}`")]
    ContainerDeletionFailed(String),
    #[error("container not found: `{0}`")]
    ContainerNotFound(String),
    #[error("directory creation failed: `{0}`: `{1}`")]
    DirectoryCreationFailed(String, String),
    #[error("directory read failed: `{0}`")]
    DirectoryReadFailed(String),
    #[error("file read failed: `{0}`")]
    FileReadFailed(String),
    #[error("file write failed: `{0}`")]
    FileWriteFailed(String),
    #[error("item read failed: `{0}`")]
    ItemReadFailed(String),
    #[error("item not found: `{0}`")]
    ItemNotFound(String),
    #[error("item write failed: `{0}`")]
    ItemWriteFailed(String),
    #[error("corrupted data: `{0}`")]
    CorruptedData(String),
}
