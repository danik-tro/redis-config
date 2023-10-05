/// Represents all possible errors that can occur during working with the source.  
#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    /// Configuration can't be read from the Redis due to the redis error
    #[error("RedisError: {0}")]
    RedisError(
        #[from]
        #[source]
        redis::RedisError,
    ),
    /// Configuration can't be serialized from the Redis payload due to invalid data
    #[error("SerdeError: {0}")]
    SerdeError(
        #[from]
        #[source]
        serde_json::error::Error,
    ),
    /// The provided source key is empty.
    #[error("Redis key doesn't exist.")]
    RedisKeyDoesNotExist,
    /// Environment variables not set to initialize redis client from the env
    #[error("Environment variable {0} should be set")]
    EnvVariableNotSet(String),
}

pub type SourceResult<T> = Result<T, SourceError>;
