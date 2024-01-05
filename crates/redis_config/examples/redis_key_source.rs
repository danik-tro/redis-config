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
    // The config should be initialized before using the RedisSource
    // can be initialized using redis-cli or another possible way
    let client = redis::Client::open(REDIS_URL).unwrap();

    let conf_settings = ApplicationSettings {
        server: ServerSettings {
            ttl: 1500,
            path: "/path/mock".into(),
        },
        db: DbSettings { pool_size: 5 },
    };

    // set env variable
    std::env::set_var("APP__DB__POOL_SIZE", "5");

    let mut settings_to_serialize = std::collections::HashMap::new();
    settings_to_serialize.insert("server".to_string(), conf_settings.server.clone());

    let mut conn = client.get_async_connection().await.unwrap();
    conn.set::<_, _, String>(
        SOURCE_KEY,
        serde_json::to_string(&settings_to_serialize).unwrap(),
    )
    .await
    .unwrap();

    let config = get_config().await.unwrap();
    assert_eq!(config, conf_settings);

    println!("Config from REDIS: \n{config:?}");
}
