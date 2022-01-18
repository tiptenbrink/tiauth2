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

    #[error("unspecified ring error")]
    RingUnspecified(#[from] ring::error::Unspecified),

    #[error("jwt error: {0}")]
    JwtError(#[from] jsonwebtoken::errors::Error),

    #[error("opaque error: {0}")]
    OpaqueError(#[from] opaquebind::Error),

    #[error("db error: {0}")]
    DbError(#[from] sqlx::Error),

    #[error("kv error: {0}")]
    KvError(#[from] redis::RedisError),

    #[error("serde error: {0}")]
    SerdeError(#[from] serde_json::Error),

    #[error("decode error base64: {0}")]
    DecodeError(#[from] base64::DecodeError),

    #[error("crypt error")]
    BadCryptInput
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