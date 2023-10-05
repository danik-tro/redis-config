mod common;

use common::get_manager;
use fake::faker::internet::en::SafeEmail;
use fake::faker::lorem::en::Words;
use fake::{Dummy, Fake, Faker};

use redis_config::RedisSource;

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
        let mut manager = get_manager().await;

        // setup
        let uuid_key = uuid7::uuid7().to_string();
        let configuration: $type = Faker.fake();
        common::set_string_key_config(&mut manager, &uuid_key, &configuration).await;

        let source = RedisSource::try_new(uuid_key.clone(), common::get_redis_url()).unwrap();
        let config_deserilized: $type = common::get_serialized_config(source).await;

        // cleanup
        common::cleanup_key(&mut manager, &uuid_key).await;

        assert_eq!(config_deserilized, configuration);
    };
}

#[tokio::test]
async fn flat_config() {
    check_key_string_config!(FlatConfiguration);
}

#[tokio::test]
async fn nested_config() {
    check_key_string_config!(NestedConfiguration);
}
