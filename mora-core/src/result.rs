use thiserror::Error;

pub type MoraResult<T> = Result<T, MoraError>;

#[derive(Debug, Error)]
pub enum MoraError {
    #[error("queue already exists `{0}`")]
    QueueAlreadyExists(String),
    #[error("enqueue operation failed: `{0}`")]
    EnqueueError(String),
    #[error("error reading config `{0}`")]
    ConfigError(String),
    #[error("error starting api layer `{0}`")]
    ApiError(String),
}
