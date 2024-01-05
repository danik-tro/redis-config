mod common;

use common::get_manager;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Words;
use fake::{Dummy, Fake, Faker};

use redis::AsyncCommands;
use redis_config::{states, RedisSource};

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

#[tokio::test]
async fn flat_config() {
    let mut manager = get_manager().await;
    // setup
    let uuid_key = uuid7::uuid7().to_string();
    let configuration: FlatConfiguration = Faker.fake();

    manager
        .hset::<_, _, _, i32>(
            &uuid_key,
            "invite_ttl",
            serde_json::to_string(&configuration.invite_ttl).unwrap(),
        )
        .await
        .unwrap();

    manager
        .hset::<_, _, _, i32>(
            &uuid_key,
            "sender_email",
            serde_json::to_string(&configuration.sender_email).unwrap(),
        )
        .await
        .unwrap();

    let source =
        RedisSource::<_, states::Hash>::try_new(uuid_key.clone(), common::get_redis_url()).unwrap();

    let config_deserilized: FlatConfiguration = common::get_serialized_config_hash(source).await;

    // cleanup
    common::cleanup_key(&mut manager, &uuid_key).await;
    assert_eq!(config_deserilized, configuration);
}

#[tokio::test]
async fn nested_config() {
    let mut manager = get_manager().await;
    // setup
    let uuid_key = uuid7::uuid7().to_string();
    let configuration: NestedConfiguration = Faker.fake();

    manager
        .hset::<_, _, _, i32>(
            &uuid_key,
            "npl_conf",
            serde_json::to_string(&configuration.npl_conf).unwrap(),
        )
        .await
        .unwrap();

    manager
        .hset::<_, _, _, i32>(
            &uuid_key,
            "stop_words",
            serde_json::to_string(&configuration.stop_words).unwrap(),
        )
        .await
        .unwrap();

    let source =
        RedisSource::<_, states::Hash>::try_new(uuid_key.clone(), common::get_redis_url()).unwrap();

    let config_deserilized: NestedConfiguration = common::get_serialized_config_hash(source).await;

    // cleanup
    common::cleanup_key(&mut manager, &uuid_key).await;
    assert_eq!(config_deserilized, configuration);
}
