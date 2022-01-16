use std::time::Duration;
use sqlx::postgres::PgPoolOptions;
use crate::data::db::{PSQL};
use crate::data::kv::{Redis};
use crate::error::Error;

pub struct Source {
    pub db: PSQL,
    pub kv: Redis
}

impl Source {
    pub async fn new(db_uri: &str, kv_uri: &str) -> Result<Self, Error> {
        let db_pool = PgPoolOptions::new().connect_timeout(Duration::from_secs(1))
            .connect(db_uri).await?;
        let client = redis::Client::open(kv_uri).unwrap();
        let kv_conn_manager = redis::aio::ConnectionManager::new(client).await.unwrap();
        let psql = PSQL { pool: db_pool };
        let redis = Redis { conn_manager: kv_conn_manager };
        Ok(Self {
            db: psql,
            kv: redis
        })
    }
}