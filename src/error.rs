#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("db error: {0}")]
    SqlxError(#[from] sqlx::Error)
}

// impl From<sqlx::Error> for Error {
//     fn from(e: sqlx::Error) -> Self {
//         todo!()
//     }
// }