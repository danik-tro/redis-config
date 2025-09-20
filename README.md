# redis-config
[![Crates.io](https://img.shields.io/crates/v/redis_config.svg)](https://crates.io/crates/redis_config)
![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Crates.io](https://img.shields.io/crates/d/redis_config.svg)](https://crates.io/crates/redis_config)
[![Docs.rs](https://docs.rs/redis_config/badge.svg)](https://docs.rs/redis_config)
[![CI](https://github.com/danik-tro/redis-config/workflows/CI/badge.svg)](https://github.com/danik-tro/redis-config/actions)
[![codecov](https://codecov.io/gh/danik-tro/redis-config/graph/badge.svg?token=yDK7m7Qvvt)](https://codecov.io/gh/danik-tro/redis-config)

> Implementation of Redis source as Async source for config-rs crate.

`redis-config` extends the list of possible sources provided by [config-rs](https://github.com/mehcode/config-rs) and provides an asynchronous `RedisSource` source using the [redis-rs](https://github.com/redis-rs/redis-rs).

`RedisSource` supports reading configuration:
 - from Hash using the HGETALL command,
 - from String using the GET command,
 - from JSON using the JSON.GET [key] $ command,

##### Features

There are a few features defined in [redis-rs](https://github.com/redis-rs/redis-rs) that can enable additional functionality if so desired.
 Some of them are turned on by default.
 - ahash: enables ahash map/set support & uses ahash internally (+7-10% performance) (optional)
 - tokio-comp: enables support for tokio runtime (enabled by default)
 - json: enables support for JSON using the JSON.GET [key] $ command (optional)

 ##### Tls features
 - tokio-native-tls-comp: enables support for native-tls for tokio (optional)
 - tokio-rustls-comp: enables support for rustls for tokio (optional).


See the examples for general usage information.

## Usage

### Dependencies

```toml
# Cargo.toml

[dependencies]
config = "0.15.6"
redis_config = { version = "*", features = ["tokio-comp"]}
tokio = { version = "1", features = ["rt", "macros", "rt-multi-thread"] }
serde = { version = "1.0", features = ["derive"]}
```

### Usage example

```rust
 use config::builder::AsyncState;
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
