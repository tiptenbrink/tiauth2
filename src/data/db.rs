use std::iter::Map;
use std::ops::Deref;
use sqlx::{Encode, FromRow, Pool, Postgres, Type};
use async_trait::async_trait;
use sqlx::postgres::{PgPoolOptions, PgRow};
use crate::error::Error;

pub trait VV:
    Encode<'static, Postgres> + Type<Postgres>
{

}

pub trait Row:
    'static
    + Send
{
    type RowModel;

    fn id(&self) -> i32;

    fn keys(&self) -> String;
    fn vals(&self) -> String;
    fn vals_vec(&self) -> Vec<Box<dyn VV>>;
    fn set(&self) -> String;

    fn bind<T: Encode<'static, Postgres> + Type<Postgres>>(&self, v: Self::RowModel) -> T;
}

#[async_trait]
pub trait Database {
    async fn retrieve_by_id<T>(&self, table: &str, id: i32) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin;

    async fn upsert_by_id<R: Row>(&self, table: &str, row: R) -> Result<(), Error>;
}

pub struct PSQL {
    pub(crate) pool: Pool<Postgres>
}

#[async_trait]
impl Database for PSQL {
    async fn retrieve_by_id<T>(&self, table: &str, id: i32) -> Result<T, Error>
    where
        T: for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let query = format!("SELECT * FROM {table} WHERE id = $1", table=table);
        let row: T = sqlx::query_as(
            &query
        )
            .bind(id)
            .fetch_one(&self.pool).await?;
        Ok(row)
    }

    async fn upsert_by_id<R: Row>(&self, table: &str, row: R) -> Result<(), Error>{
        let query = format!("INSERT INTO {table} ({keys}) VALUES ({vals}) ON CONFLICT (id)\
            DO UPDATE SET {set}", table=table, keys=row.keys(), vals=row.vals(), set=row.set());
        let mut q = sqlx::query(
            &query
        );
        for p in row.vals_vec() {
            q = q.bind(p.deref());
        };
        Ok(q.execute(&self.pool).await.and(Ok(()))?)
    }
}