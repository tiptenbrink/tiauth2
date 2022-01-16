#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("required exists")]
    RequiredExists,

    #[error("bad flow: {0}")]
    BadFlow(BadFlow),

    #[error("missing field token_request")]
    MissingFieldTokenRequest,

    #[error("incorrect username for login finish")]
    IncorrectFinishUsername,

    #[error("incorrect field: {0}")]
    IncorrectField(String),

    #[error("field encoding failure: {0}")]
    BadFieldEncoding(String),

    #[error("opaque error: {0}")]
    OpaqueError(#[from] opaquebind::Error),

    #[error("db error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("kv error: {0}")]
    KvError(#[from] redis::RedisError),

    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error)
}

#[derive(Debug, thiserror::Error)]
pub enum BadFlow {
    #[error("expired auth_id")]
    ExpiredAuthId,

    #[error("expired auth_id")]
    ExpiredFlowId,

    #[error("expired code")]
    ExpiredCode,

    #[error("bad challenge")]
    BadChallenge
}