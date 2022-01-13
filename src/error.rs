#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("kv error: {0}")]
    KvError(#[from] redis::RedisError),

    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error)
}