# redis-config

[![Crates.io](https://img.shields.io/crates/v/redis_config.svg)](https://crates.io/crates/redis_config)
![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Crates.io](https://img.shields.io/crates/d/redis_config.svg)](https://crates.io/crates/redis_config)
[![Docs.rs](https://docs.rs/redis_config/badge.svg)](https://docs.rs/redis_config)
[![CI](https://github.com/danik-tro/redis-config/workflows/CI/badge.svg)](https://github.com/danik-tro/redis-config/actions)

> Implementation of Redis source as Async source for config-rs crate.

## Usage

```toml
# Cargo.toml

[dependencies]
redis_config = "*"
```

### Feature flags

- `tokio-comp` - enables support for tokio runtime (enabled by default)
- `async-std-comp` - enables support for async-std runtime (optional)
- `ahash` - enables ahash map/set support & uses ahash internally (+7-10% performance) (optional)

### Examples

```rust
 use config::builder::AsyncState;
 use redis::AsyncCommands;
 use redis_config::{states, RedisSource};

 // hardcoded values, shouldn't be in production
 const REDIS_URL: &str = "redis://127.0.0.1:6379";
 const SOURCE_KEY: &str = "application-settings";

 #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq, Clone)]
 struct ServerSettings {
     ttl: i64,
     path: String,
     // another settings
     // ...
 }

 #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
 struct DbSettings {
     pool_size: usize,
     // another settings
     // ...
 }

 #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
 struct ApplicationSettings {
     // settings that will be taken from RedisSource
     server: ServerSettings,
     // settings that will be taken from Env
     db: DbSettings,
 }

 async fn get_config() -> Result<ApplicationSettings, config::ConfigError> {
     let config = config::ConfigBuilder::<AsyncState>::default()
         .add_source(
             config::Environment::with_prefix("APP")
                 .separator("__")
                 .try_parsing(true),
         )
         .add_async_source(
             RedisSource::<_, states::PlainString>::try_new(SOURCE_KEY, REDIS_URL)
                 .map_err(|err| config::ConfigError::NotFound(err.to_string()))?,
         )
         .build()
         .await?;

     config.try_deserialize()
 }

 #[tokio::main]
 async fn main() {
     let config = get_config().await.unwrap();
 }
```

### More

See the [documentation](https://docs.rs/redis_config) for more usage information.

## License

`redis_config` is primarily distributed under the terms of both the MIT license.

See LICENSE for details.
