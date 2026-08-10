use config::builder::AsyncState;
use redis::AsyncCommands;
use redis_config::RedisSource;
use std::time::Duration;

const REDIS_URL: &str = "redis://127.0.0.1:6379";

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct AppConfig {
    server_port: u16,
    database_url: String,
    max_connections: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct ServerConfig {
    host: String,
    port: u16,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct DatabaseConfig {
    url: String,
    pool_size: usize,
}

#[derive(Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct MultiSourceConfig {
    server: ServerConfig,
    database: DatabaseConfig,
}

async fn setup_test_data() -> redis::aio::MultiplexedConnection {
    let client = redis::Client::open(REDIS_URL).unwrap();
    let mut conn = client.get_multiplexed_async_connection().await.unwrap();

    // Setup string config
    let app_config = AppConfig {
        server_port: 8080,
        database_url: "postgres://localhost:5432/mydb".to_string(),
        max_connections: 100,
    };

    conn.set::<_, _, String>(
        "app-config-string",
        serde_json::to_string(&app_config).unwrap(),
    )
    .await
    .unwrap();

    // Setup hash config
    let server_config = ServerConfig {
        host: "localhost".to_string(),
        port: 8080,
    };

    let db_config = DatabaseConfig {
        url: "postgres://localhost:5432/mydb".to_string(),
        pool_size: 10,
    };

    conn.hset::<_, _, _, i32>(
        "app-config-hash",
        "server",
        serde_json::to_string(&server_config).unwrap(),
    )
    .await
    .unwrap();

    conn.hset::<_, _, _, i32>(
        "app-config-hash",
        "database",
        serde_json::to_string(&db_config).unwrap(),
    )
    .await
    .unwrap();

    #[cfg(feature = "json")]
    {
        use redis::JsonAsyncCommands;
        let multi_config = MultiSourceConfig {
            server: server_config,
            database: db_config,
        };

        let _: bool = conn
            .json_set("app-config-json", "$", &multi_config)
            .await
            .unwrap();
    }

    conn
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut _conn = setup_test_data().await;

    println!("=== Redis Config Builder Pattern Examples ===\n");

    // Example 1: Basic string configuration
    println!("1. Basic String Configuration:");
    let source = RedisSource::builder()
        .url(REDIS_URL)
        .key("app-config-string")
        .as_string()
        .build()?;

    let config = config::ConfigBuilder::<AsyncState>::default()
        .add_async_source(source)
        .build()
        .await?;

    let app_config: AppConfig = config.try_deserialize()?;
    println!("   Config loaded: {app_config:#?}\n");

    // Example 2: Hash configuration with options
    println!("2. Hash Configuration with Timeout:");
    let source = RedisSource::builder()
        .url(REDIS_URL)
        .key("app-config-hash")
        .as_hash()
        .with_timeout(Duration::from_secs(5))
        .build()?;

    let config = config::ConfigBuilder::<AsyncState>::default()
        .add_async_source(source)
        .build()
        .await?;

    let multi_config: MultiSourceConfig = config.try_deserialize()?;
    println!("   Config loaded: {multi_config:#?}\n");

    // Example 3: Optional configuration (won't fail if missing)
    println!("3. Optional Configuration (missing key):");
    let source = RedisSource::builder()
        .url(REDIS_URL)
        .key("missing-config")
        .as_string()
        .optional()
        .build()?;

    let config = config::ConfigBuilder::<AsyncState>::default()
        .add_async_source(source)
        .build()
        .await?;

    println!(
        "   Config loaded successfully (empty): {:?}\n",
        config.try_deserialize::<serde_json::Value>()
    );

    // Example 4: Environment-based configuration
    std::env::set_var("REDIS_URL", REDIS_URL);
    std::env::set_var("CONFIG_KEY", "app-config-string");

    println!("4. Environment-based Configuration:");
    let source = redis_config::RedisSourceBuilder::from_env("REDIS_URL")?
        .key_from_env("CONFIG_KEY")?
        .as_string()
        .build()?;

    let config = config::ConfigBuilder::<AsyncState>::default()
        .add_async_source(source)
        .build()
        .await?;

    let app_config: AppConfig = config.try_deserialize()?;
    println!("   Config loaded from env: {app_config:#?}\n");

    #[cfg(feature = "json")]
    {
        // Example 5: JSON configuration
        println!("5. JSON Configuration:");
        let source = RedisSource::builder()
            .url(REDIS_URL)
            .key("app-config-json")
            .as_json()
            .build()?;

        let config = config::ConfigBuilder::<AsyncState>::default()
            .add_async_source(source)
            .build()
            .await?;

        let multi_config: MultiSourceConfig = config.try_deserialize()?;
        println!("   Config loaded: {multi_config:#?}\n");
    }

    // Example 6: Multiple sources with different types
    println!("6. Multiple Sources (Layered Configuration):");

    // Base config from environment
    std::env::set_var("APP_SERVER_PORT", "3000");
    std::env::set_var("APP_MAX_CONNECTIONS", "50");

    let redis_source = RedisSource::builder()
        .url(REDIS_URL)
        .key("app-config-string")
        .as_string()
        .optional()
        .build()?;

    let config = config::ConfigBuilder::<AsyncState>::default()
        .add_source(
            config::Environment::with_prefix("APP")
                .separator("_")
                .try_parsing(true),
        )
        .add_async_source(redis_source)
        .build()
        .await?;

    let layered_config: AppConfig = config.try_deserialize()?;
    println!("   Layered config (Redis overrides ENV): {layered_config:#?}\n");

    println!("=== All examples completed successfully! ===");

    Ok(())
}
