use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Words;
use fake::{Dummy, Fake, Faker};

use redis::AsyncCommands;
use redis_config::{states, RedisSource};

use std::fmt::Debug;

use config::builder::AsyncState;
use serde::de::DeserializeOwned;

const TEST_REDIS_URL_KEY: &str = "TEST_REDIS_URL";
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

pub fn get_redis_url() -> String {
    std::env::var(TEST_REDIS_URL_KEY).unwrap_or(DEFAULT_REDIS_URL.into())
}

#[allow(unused)]
pub async fn get_serialized_config_plain_string<
    SK: redis::ToRedisArgs + Clone + Send + Sync + Debug + 'static,
    C: DeserializeOwned,
>(
    source: redis_config::RedisSource<SK, states::PlainString>,
) -> C {
    config::ConfigBuilder::<AsyncState>::default()
        .add_async_source(source)
        .build()
        .await
        .unwrap()
        .try_deserialize()
        .unwrap()
}

#[allow(unused)]
pub async fn get_serialized_config_hash<
    SK: redis::ToRedisArgs + Clone + Send + Sync + Debug + 'static,
    C: DeserializeOwned,
>(
    source: redis_config::RedisSource<SK, states::Hash>,
) -> C {
    config::ConfigBuilder::<AsyncState>::default()
        .add_async_source(source)
        .build()
        .await
        .unwrap()
        .try_deserialize()
        .unwrap()
}

pub async fn get_connection() -> redis::aio::MultiplexedConnection {
    let redis_url = get_redis_url();
    let client = redis::Client::open(redis_url).unwrap();
    client.get_multiplexed_async_connection().await.unwrap()
}

#[allow(unused)]
pub async fn set_string_key_config<T: serde::Serialize>(
    connection: &mut redis::aio::MultiplexedConnection,
    key: &str,
    config: &T,
) {
    connection
        .set::<_, _, String>(key, serde_json::to_string(config).unwrap())
        .await
        .unwrap();
}

pub async fn cleanup_key(connection: &mut redis::aio::MultiplexedConnection, key: &str) {
    // On CI it randomly failed, so we don't check for the result
    _ = connection.del::<_, i32>(key).await;
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Dummy, PartialEq, Eq)]
struct FlatConfiguration {
    #[dummy(faker = "SafeEmail()")]
    sender_email: String,
    #[dummy(faker = "100..500")]
    invite_ttl: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Dummy)]
struct NlpProcessingConfiguration {
    #[dummy(faker = "Words(3..5)")]
    words_to_delete: Vec<String>,
    #[dummy(faker = "Words(5..10)")]
    words_to_safe: Vec<String>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Dummy)]
struct NestedConfiguration {
    #[dummy(faker = "Words(3..5)")]
    stop_words: Vec<String>,
    npl_conf: NlpProcessingConfiguration,
}

macro_rules! check_key_string_config {
    ($type:ty) => {
        let mut connection = get_connection().await;

        // setup
        let uuid_key = uuid7::uuid7().to_string();
        let configuration: $type = Faker.fake();
        set_string_key_config(&mut connection, &uuid_key, &configuration).await;

        let source = RedisSource::try_new(uuid_key.clone(), get_redis_url()).unwrap();
        let config_deserilized: $type = get_serialized_config_plain_string(source).await;

        // cleanup
        cleanup_key(&mut connection, &uuid_key).await;

        assert_eq!(config_deserilized, configuration);
    };
}

#[tokio::test]
async fn hash_flat_config() {
    let mut connection = get_connection().await;
    // setup
    let uuid_key = uuid7::uuid7().to_string();
    let configuration: FlatConfiguration = Faker.fake();

    connection
        .hset::<_, _, _, i32>(
            &uuid_key,
            "invite_ttl",
            serde_json::to_string(&configuration.invite_ttl).unwrap(),
        )
        .await
        .unwrap();

    connection
        .hset::<_, _, _, i32>(
            &uuid_key,
            "sender_email",
            serde_json::to_string(&configuration.sender_email).unwrap(),
        )
        .await
        .unwrap();

    let source =
        RedisSource::<_, states::Hash>::try_new(uuid_key.clone(), get_redis_url()).unwrap();

    let config_deserilized: FlatConfiguration = get_serialized_config_hash(source).await;

    // cleanup
    cleanup_key(&mut connection, &uuid_key).await;
    assert_eq!(config_deserilized, configuration);
}

#[tokio::test]
async fn hash_nested_config() {
    let mut connection = get_connection().await;
    // setup
    let uuid_key = uuid7::uuid7().to_string();
    let configuration: NestedConfiguration = Faker.fake();

    connection
        .hset::<_, _, _, i32>(
            &uuid_key,
            "npl_conf",
            serde_json::to_string(&configuration.npl_conf).unwrap(),
        )
        .await
        .unwrap();

    connection
        .hset::<_, _, _, i32>(
            &uuid_key,
            "stop_words",
            serde_json::to_string(&configuration.stop_words).unwrap(),
        )
        .await
        .unwrap();

    let source =
        RedisSource::<_, states::Hash>::try_new(uuid_key.clone(), get_redis_url()).unwrap();

    let config_deserilized: NestedConfiguration = get_serialized_config_hash(source).await;

    // cleanup
    cleanup_key(&mut connection, &uuid_key).await;
    assert_eq!(config_deserilized, configuration);
}

#[tokio::test]
async fn plain_string_flat_config() {
    check_key_string_config!(FlatConfiguration);
}

#[tokio::test]
async fn plain_string_nested_config() {
    check_key_string_config!(NestedConfiguration);
}
