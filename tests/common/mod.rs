use std::{fmt::Debug, sync::Arc};

use async_once_cell::OnceCell;
use config::builder::AsyncState;
use redis::AsyncCommands;
use redis_config::states;
use serde::de::DeserializeOwned;

const TEST_REDIS_URL_KEY: &str = "TEST_REDIS_URL";
pub const DEFAULT_REDIS_URL: &str = "redis://127.0.0.1:6379";

static REDIS_MANAGER: OnceCell<Arc<redis::aio::ConnectionManager>> = OnceCell::new();

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

pub async fn get_manager() -> redis::aio::ConnectionManager {
    let manager = REDIS_MANAGER
        .get_or_init(async {
            let redis_url = get_redis_url();
            let client = redis::Client::open(redis_url).unwrap();
            let manager = redis::aio::ConnectionManager::new(client).await.unwrap();

            Arc::new(manager)
        })
        .await;

    manager.as_ref().clone()
}

#[allow(unused)]
pub async fn set_string_key_config<T: serde::Serialize>(
    manager: &mut redis::aio::ConnectionManager,
    key: &str,
    config: &T,
) {
    manager
        .set::<_, _, String>(key, serde_json::to_string(config).unwrap())
        .await
        .unwrap();
}

pub async fn cleanup_key(manager: &mut redis::aio::ConnectionManager, key: &str) {
    // On CI it randomly failed, so we don't check for the result
    _ = manager.del::<_, i32>(key).await;
}
