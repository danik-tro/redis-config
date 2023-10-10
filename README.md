# redis-config

![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)
[![Crates.io](https://img.shields.io/crates/d/redis_config.svg)](https://crates.io/crates/redis_config)
[![Docs.rs](https://docs.rs/redis_config/badge.svg)](https://docs.rs/redis_config)

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
 use redis_config::RedisSource;

 #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
 struct ServerSettings {
     ttl: i64,
     path: String,
 }

 #[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
 struct ApplicationSettings {
     server: ServerSettings,
 }

 async fn get_config() -> Result<ApplicationSettings, config::ConfigError> {
     let config = config::ConfigBuilder::<AsyncState>::default()
         .add_async_source(
             RedisSource::try_new(SOURCE_KEY, REDIS_URL)
                 .map_err(|err| config::ConfigError::NotFound(err.to_string()))?,
             )
         .build()
         .await?;
     config.try_deserialize()
 }

 let config = get_config().await.unwrap();
```

### More

See the [documentation](https://docs.rs/redis_config) for more usage information.

## License

`redis_config` is primarily distributed under the terms of both the MIT license.

See LICENSE-MIT for details.