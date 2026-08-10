//! Docs
#![doc = include_str!("../README.md")]
//!

pub mod builder;
mod errors;
pub mod states;

pub use builder::RedisSourceBuilder;
pub use errors::{SourceError, SourceResult};

use std::{collections::HashMap, fmt::Debug, marker::PhantomData};

use config::{AsyncSource, ConfigError, Map};
use redis::AsyncCommands;

#[cfg(feature = "json")]
use redis::JsonAsyncCommands;

/// A configuration async source backed up by a Redis.
///
/// It supports retrieving configuration in JSON format from Strings and Hashes types in Redis.
///
/// # Examples
///
/// ## Using the Builder Pattern (Recommended)
///
/// ```rust,no_run
/// use config::builder::AsyncState;
/// use redis_config::RedisSource;
/// use std::time::Duration;
///
/// #[derive(Debug, serde::Serialize, serde::Deserialize)]
/// struct AppConfig {
///     server_port: u16,
///     database_url: String,
/// }
///
/// async fn get_config() -> Result<AppConfig, config::ConfigError> {
///     // String configuration with builder pattern
///     let redis_source = RedisSource::builder()
///         .url("redis://127.0.0.1:6379")
///         .key("app-config")
///         .as_string()
///         .optional()
///         .with_timeout(Duration::from_secs(5))
///         .build()
///         .map_err(|err| config::ConfigError::NotFound(err.to_string()))?;
///
///     let config = config::ConfigBuilder::<AsyncState>::default()
///         .add_async_source(redis_source)
///         .build()
///         .await?;
///
///     config.try_deserialize()
/// }
/// ```
///
/// ## Different Redis Data Types
///
/// ```rust,no_run
/// use redis_config::RedisSource;
///
/// // String data (JSON stored as Redis string)
/// let string_source = RedisSource::builder()
///     .url("redis://localhost:6379")
///     .key("config-key")
///     .as_string()
///     .build()?;
///
/// // Hash data (key-value pairs stored as Redis hash)
/// let hash_source = RedisSource::builder()
///     .url("redis://localhost:6379")
///     .key("config-hash")
///     .as_hash()
///     .build()?;
///
/// // JSON data (requires 'json' feature and RedisJSON module)
/// #[cfg(feature = "json")]
/// let json_source = RedisSource::builder()
///     .url("redis://localhost:6379")
///     .key("config-json")
///     .as_json()
///     .build()?;
/// # Ok::<(), redis_config::SourceError>(())
/// ```
///
/// ## Environment-based Configuration
///
/// ```rust,no_run
/// use redis_config::RedisSourceBuilder;
///
/// // Load connection details from environment variables
/// std::env::set_var("REDIS_URL", "redis://localhost:6379");
/// std::env::set_var("CONFIG_KEY", "my-app-config");
///
/// let source = RedisSourceBuilder::from_env("REDIS_URL")?
///     .key_from_env("CONFIG_KEY")?
///     .as_string()
///     .build()?;
/// # Ok::<(), redis_config::SourceError>(())
/// ```
///
/// ## Legacy API (still supported)
///
/// ```rust,no_run
/// use config::builder::AsyncState;
/// use redis_config::{states, RedisSource};
///
/// const REDIS_URL: &str = "redis://127.0.0.1:6379";
/// const SOURCE_KEY: &str = "application-settings";
///
/// async fn get_config_legacy() -> Result<serde_json::Value, config::ConfigError> {
///     let config = config::ConfigBuilder::<AsyncState>::default()
///         .add_async_source(
///             RedisSource::<_, states::PlainString>::try_new(SOURCE_KEY, REDIS_URL)
///                 .map_err(|err| config::ConfigError::NotFound(err.to_string()))?,
///         )
///         .build()
///         .await?;
///
///     config.try_deserialize()
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

impl RedisSource<String, states::PlainString> {
    /// Create a new builder for constructing a `RedisSource` with a fluent API.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use redis_config::RedisSource;
    /// use std::time::Duration;
    ///
    /// let source = RedisSource::builder()
    ///     .url("redis://localhost:6379")
    ///     .key("app-config")
    ///     .as_string()
    ///     .optional()
    ///     .with_timeout(Duration::from_secs(5))
    ///     .build()?;
    /// # Ok::<(), redis_config::SourceError>(())
    /// ```
    #[must_use]
    pub fn builder() -> builder::RedisSourceBuilder {
        builder::RedisSourceBuilder::new()
    }
}

impl<SK: redis::ToRedisArgs + Clone + Send + Sync> RedisSource<SK, states::PlainString> {
    /// Collects the configuration from Redis using the GET command.
    async fn collect_from_key(&self) -> SourceResult<Map<String, config::Value>> {
        let data: Option<Vec<u8>> = self
            .client
            .get_multiplexed_async_connection()
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
            .get_multiplexed_async_connection()
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

#[cfg(feature = "json")]
impl<SK: redis::ToRedisArgs + Clone + Send + Sync> RedisSource<SK, states::Json> {
    async fn collect_from_json(&self) -> SourceResult<Map<String, config::Value>> {
        let data: Option<Vec<u8>> = self
            .client
            .get_multiplexed_async_connection()
            .await?
            .json_get(self.source_key.clone(), "$")
            .await?;

        if !self.required && data.is_none() {
            return Ok(Map::new());
        }

        let data = data.ok_or(SourceError::RedisKeyDoesNotExist)?;

        let data: Vec<Map<String, config::Value>> =
            serde_json::from_slice(&data).map_err(SourceError::SerdeError)?;

        data.into_iter()
            .next()
            .ok_or(SourceError::RedisKeyDoesNotExist)
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

#[cfg(feature = "json")]
#[async_trait::async_trait]
impl<SK: redis::ToRedisArgs + Clone + Send + Sync + Debug> AsyncSource
    for RedisSource<SK, states::Json>
{
    async fn collect(&self) -> Result<Map<String, config::Value>, ConfigError> {
        self.collect_from_json()
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
