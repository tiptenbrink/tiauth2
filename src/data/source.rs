use sqlx::postgres::PgPoolOptions;
use crate::data::db::{PSQL};
use crate::error::Error;

pub struct Source {
    pub db: PSQL
}

impl Source {
    pub async fn new(uri: &str) -> Result<Self, Error> {
        let pool = PgPoolOptions::new().connect(uri).await?;
        let psql = PSQL { pool };
        Ok(Self {
            db: psql
        })
    }
}