//! Docs 
#![doc = include_str!("../README.md")]
//! 

mod errors;
pub mod states;

pub use errors::{SourceError, SourceResult};

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use config::{AsyncSource, ConfigError, Map};
use redis::AsyncCommands;

/// A configuration async source backed up by a Redis.
///
/// It supports retrieving configuration in JSON format from Strings and Hashes types in Redis.
///
/// # Examples
///
/// ```rust,no_run
/// use config::builder::AsyncState;
/// use redis_config::{states, RedisSource};
///
/// // hardcoded values, shouldn't be in production
/// const REDIS_URL: &str = "redis://127.0.0.1:6379";
/// const SOURCE_KEY: &str = "application-settings";
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
/// struct ServerSettings {
///     ttl: i64,
///     path: String,
///     // another settings
///     // ...
/// }
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// struct DbSettings {
///     pool_size: usize,
///     // another settings
///     // ...
/// }
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// struct ApplicationSettings {
///     // settings that will be taken from RedisSource
///     server: ServerSettings,
///     // settings that will be taken from Env
///     db: DbSettings,
/// }
///
/// async fn get_config() -> Result<ApplicationSettings, config::ConfigError> {
///     let config = config::ConfigBuilder::<AsyncState>::default()
///         .add_source(
///             config::Environment::with_prefix("APP")
///                 .separator("__")
///                 .try_parsing(true),
///         )
///         .add_async_source(
///             RedisSource::<_, states::PlainString>::try_new(SOURCE_KEY, REDIS_URL)
///                 .map_err(|err| config::ConfigError::NotFound(err.to_string()))?,
///         )
///         .build()
///         .await?;
///
///     config.try_deserialize()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let config = get_config().await.unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct RedisSource<SK, S = states::PlainString> {
    /// Redis client
    client: redis::Client,
    /// The source key with the configuration
    source_key: SK,
    /// Indicate the Redis type in which configurations are stored. True for Hash, or String otherwise
    /// A required source will error if the key cannot be found
    required: bool,
    /// Indicate the Redis type in which configurations are stored.
    ///
    /// - use `Hash` state, if you want to retrieve configuration from Redis with the HGETALL command.
    /// - use `PlainString` state, if you want to retrieve configuration from Redis with the GET command
    state: PhantomData<S>,
}

impl<SK, S> RedisSource<SK, S> {
    /// Initialize a new redis source with provided `source_key` and `connection_info` to Redis.
    ///
    /// # Errors
    ///
    /// Will return `SourceError::RedisError` if connection info is failed to parse.
    pub fn try_new<I: redis::IntoConnectionInfo>(
        source_key: SK,
        connection_info: I,
    ) -> SourceResult<Self> {
        let client = redis::Client::open(connection_info)?;
        Ok(Self {
            client,
            source_key,
            required: true,
            state: PhantomData,
        })
    }

    #[must_use]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl<SK: redis::ToRedisArgs + Clone + Send + Sync> RedisSource<SK, states::PlainString> {
    /// Collects the configuration from Redis using the GET command.
    async fn collect_from_key(&self) -> SourceResult<Map<String, config::Value>> {
        let data: Option<Vec<u8>> = self
            .client
            .get_async_connection()
            .await?
            .get(self.source_key.clone())
            .await?;

        if !self.required && data.is_none() {
            return Ok(Map::new());
        }

        let data = data.ok_or(SourceError::RedisKeyDoesNotExist)?;

        Ok(serde_json::from_slice(&data)?)
    }
}

impl<SK: redis::ToRedisArgs + Clone + Send + Sync> RedisSource<SK, states::Hash> {
    /// Collects the configuration from Redis using the HGETALL command.
    async fn collect_from_hash(&self) -> SourceResult<Map<String, config::Value>> {
        let data: Option<HashMap<String, Vec<u8>>> = self
            .client
            .get_async_connection()
            .await?
            .hgetall(self.source_key.clone())
            .await?;

        if !self.required && data.is_none() {
            return Ok(Map::new());
        }

        let data = data.ok_or(SourceError::RedisKeyDoesNotExist)?;

        data.into_iter()
            .map(|(k, v)| -> Result<(String, config::Value), SourceError> {
                Ok((k, serde_json::from_slice(&v)?))
            })
            .collect::<Result<HashMap<String, config::Value>, _>>()
    }
}

#[async_trait::async_trait]
impl<SK: redis::ToRedisArgs + Clone + Send + Sync + Debug> AsyncSource
    for RedisSource<SK, states::Hash>
{
    async fn collect(&self) -> Result<Map<String, config::Value>, ConfigError> {
        self.collect_from_hash()
            .await
            .map_err(|err| ConfigError::NotFound(err.to_string()))
    }
}

#[async_trait::async_trait]
impl<SK: redis::ToRedisArgs + Clone + Send + Sync + Debug> AsyncSource
    for RedisSource<SK, states::PlainString>
{
    async fn collect(&self) -> Result<Map<String, config::Value>, ConfigError> {
        self.collect_from_key()
            .await
            .map_err(|err| ConfigError::NotFound(err.to_string()))
    }
}
