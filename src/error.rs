#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("required exists")]
    RequiredExists,

    #[error("expired auth_id or flow_id")]
    FlowExpired,

    #[error("incorrect username for login finish")]
    IncorrectFinishUsername,

    #[error("opaque error: {0}")]
    OpaqueError(#[from] opaquebind::Error),

    #[error("db error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("kv error: {0}")]
    KvError(#[from] redis::RedisError),

    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error)
}