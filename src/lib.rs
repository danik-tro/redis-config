//! Crate extends the list of possible sources provided by [config-rs](https://github.com/mehcode/config-rs).
//! Crate provides an asynchronous `RedisSource` source using the [redis-rs](https://github.com/redis-rs/redis-rs).
//! `RedisSource` supports reading configuration:
//! - from Hash using the HGETALL command,
//! - from String using the GET command,
//!
//! # Optional Features
//!
//! There are a few features defined in [redis-rs](https://github.com/redis-rs/redis-rs) that can enable additional functionality if so desired.
//! Some of them are turned on by default.
//! - ahash: enables ahash map/set support & uses ahash internally (+7-10% performance) (optional)
//! - tokio-comp: enables support for tokio runtime (enabled by default)
//! - async-std-comp: enables support for async-std runtime (optional)
//!
//! Tls features
//! - async-std-native-tls-comp: enables support for native-tls for async-std (optional)
//! - async-std-rustls-comp: enables support for rustls for async-std (optional)
//! - tokio-native-tls-comp: enables support for native-tls for tokio (optional)
//! - tokio-rustls-comp: enables support for rustls for tokio (optional)
//!
//! See the examples for general usage information.

mod errors;

pub use errors::{SourceError, SourceResult};

use std::{collections::HashMap, fmt::Debug};

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
/// use redis_config::RedisSource;
///
///
/// const REDIS_URL: &str = "redis://127.0.0.1:6379";
/// const SOURCE_KEY: &str = "application-settings";
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// struct ServerSettings {
///     ttl: i64,
///     path: String,
/// }
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
/// struct ApplicationSettings {
///     server: ServerSettings,
/// }
///
/// async fn get_config() -> Result<ApplicationSettings, config::ConfigError> {
///     let config = config::ConfigBuilder::<AsyncState>::default()
///         .add_async_source(
///             RedisSource::try_new(SOURCE_KEY, REDIS_URL)
///                 .map_err(|err| config::ConfigError::NotFound(err.to_string()))?,
///             )
///         .build()
///         .await?;
///     config.try_deserialize()
/// }
///
/// #[tokio::main]
/// async fn main() {
///     let config = get_config().await.unwrap();
/// }
/// ```
#[derive(Debug)]
pub struct RedisSource<SK> {
    /// Redis client
    client: redis::Client,
    /// The source key with the configuration
    source_key: SK,
    /// Indicate the Redis type in which configurations are stored. True for Hash, or String otherwise
    is_hash: bool,
    /// A required source will error if the key cannot be found
    required: bool,
}

impl<SK> RedisSource<SK> {
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
            is_hash: false,
            required: true,
        })
    }

    /// Indicate the Redis type in which configurations are stored. True for Hash, or String otherwise
    ///
    /// - Set hash to true, if you want to retrieve configuration from Redis with the HGETALL command.
    /// - Set hash to false, if you want to retrieve configuration from Redis with the GET command
    #[must_use]
    pub fn set_hash(mut self, value: bool) -> Self {
        self.is_hash = value;
        self
    }

    #[must_use]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl<SK: redis::ToRedisArgs + Clone + Send + Sync> RedisSource<SK> {
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

    /// Collects the configuration from Redis using the HGETALL command.
    ///
    /// # Errors
    ///
    /// The method has not optimal realization, so with non-numeric data may be failed deserialization
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
impl<SK: redis::ToRedisArgs + Clone + Send + Sync + Debug> AsyncSource for RedisSource<SK> {
    async fn collect(&self) -> Result<Map<String, config::Value>, ConfigError> {
        if self.is_hash {
            self.collect_from_hash()
                .await
                .map_err(|err| ConfigError::NotFound(err.to_string()))
        } else {
            self.collect_from_key()
                .await
                .map_err(|err| ConfigError::NotFound(err.to_string()))
        }
    }
}