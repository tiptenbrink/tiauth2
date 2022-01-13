use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use serde_json::{from_str as serde_json_from_str, to_string as serde_json_to_str};
use crate::error::Error;
use async_trait::async_trait;
use serde::de::DeserializeOwned;

pub struct Redis {
    pub(crate) conn_manager: ConnectionManager
}

#[async_trait]
pub trait KeyValue {
    async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error>;

    async fn store_json<T: Serialize + Sync>(&self, key: &str, json: &T, expire: usize) -> Result<(), Error>;
}

#[async_trait]
impl KeyValue for Redis {
    async fn get_json<T: DeserializeOwned>(&self, key: &str) -> Result<T, Error> {
        let json_str: String = redis::cmd("JSON.GET").arg(key).query_async(&mut self.conn_manager.clone()).await?;
        Ok(serde_json_from_str::<T>(&json_str)?)
    }

    async fn store_json<T: Serialize + Sync>(&self, key: &str, json: &T, expire: usize) -> Result<(), Error> {
        let json_str = serde_json_to_str(json)?;

        let _: () = redis::pipe()
            .expire(key, expire).ignore()
            .cmd("JSON.SET").arg(key).arg(".").arg(&json_str)
            .query_async(&mut self.conn_manager.clone()).await?;
        Ok(())
    }
}