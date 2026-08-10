use redis_config::RedisSource;
use std::time::Duration;

#[test]
fn test_builder_compilation() {
    // Basic builder usage
    let _source = RedisSource::builder()
        .url("redis://localhost:6379")
        .key("test-key")
        .as_string()
        .build();

    // With options
    let _source = RedisSource::builder()
        .url("redis://localhost:6379")  
        .key("test-key")
        .as_hash()
        .optional()
        .with_timeout(Duration::from_secs(5))
        .with_connection_pool_size(10)
        .build();

    // From environment
    let _source = redis_config::RedisSourceBuilder::from_env("REDIS_URL")
        .and_then(|builder| builder.key_from_env("REDIS_KEY"))
        .map(|builder| builder.as_string().build());

    #[cfg(feature = "json")]
    {
        let _source = RedisSource::builder()
            .url("redis://localhost:6379")
            .key("test-json")
            .as_json()
            .build();
    }
}