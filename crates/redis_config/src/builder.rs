use std::{marker::PhantomData, time::Duration};

use crate::{states, RedisSource, SourceError, SourceResult};

#[derive(Debug)]
pub struct NoState;

#[derive(Debug)]
pub struct NoUrl;

#[derive(Debug)]
pub struct NoKey;

#[derive(Debug)]
pub struct RedisSourceBuilder<S = NoState, U = NoUrl, K = NoKey> {
    url: Option<String>,
    key: Option<String>,
    required: bool,
    timeout: Option<Duration>,
    connection_pool_size: Option<u32>,
    _state: PhantomData<S>,
    _url: PhantomData<U>,
    _key: PhantomData<K>,
}

impl Default for RedisSourceBuilder {
    fn default() -> Self {
        Self {
            url: None,
            key: None,
            required: true,
            timeout: None,
            connection_pool_size: None,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        }
    }
}

impl RedisSourceBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn url<T: Into<String>>(self, url: T) -> RedisSourceBuilder<NoState, String, NoKey> {
        RedisSourceBuilder {
            url: Some(url.into()),
            key: self.key,
            required: self.required,
            timeout: self.timeout,
            connection_pool_size: self.connection_pool_size,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        }
    }

    /// Create a new builder from environment variable containing Redis URL.
    ///
    /// # Errors
    ///
    /// Returns `SourceError::EnvVariableNotSet` if the environment variable is not set.
    pub fn from_env(url_env: &str) -> SourceResult<RedisSourceBuilder<NoState, String, NoKey>> {
        let url = std::env::var(url_env)
            .map_err(|_| SourceError::EnvVariableNotSet(url_env.to_string()))?;

        Ok(RedisSourceBuilder {
            url: Some(url),
            key: None,
            required: true,
            timeout: None,
            connection_pool_size: None,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        })
    }
}

impl<S, K> RedisSourceBuilder<S, String, K> {
    pub fn key<T: Into<String>>(self, key: T) -> RedisSourceBuilder<S, String, String> {
        RedisSourceBuilder {
            url: self.url,
            key: Some(key.into()),
            required: self.required,
            timeout: self.timeout,
            connection_pool_size: self.connection_pool_size,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        }
    }

    /// Set the Redis key from an environment variable.
    ///
    /// # Errors
    ///
    /// Returns `SourceError::EnvVariableNotSet` if the environment variable is not set.
    pub fn key_from_env(
        self,
        key_env: &str,
    ) -> SourceResult<RedisSourceBuilder<S, String, String>> {
        let key = std::env::var(key_env)
            .map_err(|_| SourceError::EnvVariableNotSet(key_env.to_string()))?;

        Ok(RedisSourceBuilder {
            url: self.url,
            key: Some(key),
            required: self.required,
            timeout: self.timeout,
            connection_pool_size: self.connection_pool_size,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        })
    }
}

impl<U, K> RedisSourceBuilder<NoState, U, K> {
    #[must_use]
    pub fn as_string(self) -> RedisSourceBuilder<states::PlainString, U, K> {
        RedisSourceBuilder {
            url: self.url,
            key: self.key,
            required: self.required,
            timeout: self.timeout,
            connection_pool_size: self.connection_pool_size,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        }
    }

    #[must_use]
    pub fn as_hash(self) -> RedisSourceBuilder<states::Hash, U, K> {
        RedisSourceBuilder {
            url: self.url,
            key: self.key,
            required: self.required,
            timeout: self.timeout,
            connection_pool_size: self.connection_pool_size,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        }
    }

    #[cfg(feature = "json")]
    #[must_use]
    pub fn as_json(self) -> RedisSourceBuilder<states::Json, U, K> {
        RedisSourceBuilder {
            url: self.url,
            key: self.key,
            required: self.required,
            timeout: self.timeout,
            connection_pool_size: self.connection_pool_size,
            _state: PhantomData,
            _url: PhantomData,
            _key: PhantomData,
        }
    }
}

impl<S, U, K> RedisSourceBuilder<S, U, K> {
    #[must_use]
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    #[must_use]
    pub fn optional(self) -> Self {
        self.required(false)
    }

    #[must_use]
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    #[must_use]
    pub fn with_connection_pool_size(mut self, size: u32) -> Self {
        self.connection_pool_size = Some(size);
        self
    }
}

impl<S> RedisSourceBuilder<S, String, String> {
    /// Build the `RedisSource` from the configured options.
    ///
    /// # Errors
    ///
    /// Returns `SourceError::EnvVariableNotSet` if URL or key are missing.
    /// Returns `SourceError::RedisError` if Redis connection fails.
    pub fn build(self) -> SourceResult<RedisSource<String, S>> {
        let url = self
            .url
            .ok_or_else(|| SourceError::EnvVariableNotSet("Redis URL is required".to_string()))?;

        let key = self
            .key
            .ok_or_else(|| SourceError::EnvVariableNotSet("Redis key is required".to_string()))?;

        let client = redis::Client::open(url)?;

        Ok(RedisSource {
            client,
            source_key: key,
            required: self.required,
            state: PhantomData,
        })
    }
}
