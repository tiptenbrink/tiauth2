use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as serde_from_j_str, to_string as serde_to_j_str};
use crate::error::Error;
use async_trait::async_trait;
use redis::{FromRedisValue, Value};
use serde::de::DeserializeOwned;

pub struct Redis {
    pub(crate) conn_manager: ConnectionManager
}

#[async_trait]
pub trait KeyValue {
    async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error>;

    async fn store_json<T: Serialize + Sync>(&self, key: &str, json: &T, expire: usize) -> Result<(), Error>;
}

#[async_trait]
impl KeyValue for Redis {
    async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, Error> {
        let val: Value = redis::cmd("JSON.GET").arg(key).query_async(&mut self.conn_manager.clone()).await?;
        match val {
            Value::Nil => Ok(None),
            _ => {
                let json_str = String::from_redis_value(&val)?;
                Ok(Some(serde_from_j_str::<T>(&json_str)?))
            }
        }
    }

    async fn store_json<T: Serialize + Sync>(&self, key: &str, json: &T, expire: usize) -> Result<(), Error> {
        let json_str = serde_to_j_str(json)?;

        let _: () = redis::pipe()
            .expire(key, expire).ignore()
            .cmd("JSON.SET").arg(key).arg(".").arg(&json_str)
            .query_async(&mut self.conn_manager.clone()).await?;
        Ok(())
    }
}