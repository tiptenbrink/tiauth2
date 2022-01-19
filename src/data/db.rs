use sqlx::{Executor, FromRow, Pool, Postgres, Transaction};
use async_trait::async_trait;
use sqlx::postgres::{PgRow};
use crate::error::Error;
use sea_query::{Value, Values};
sea_query::sea_query_driver_postgres!();
use sea_query_driver_postgres::{bind_query, bind_query_as};

#[async_trait]
pub trait Database {
    async fn retrieve_by_id<T>(&self, table: &str, id: i32) -> Result<Option<T>, Error>
        where
            T: for<'r> FromRow<'r, PgRow> + Send + Unpin;

    async fn retrieve_by_unique<T>(&self, table: &str, unique_column: &str, value: Values) -> Result<Option<T>, Error>
        where
            T: for<'r> FromRow<'r, PgRow> + Send + Unpin;

    async fn upsert_by_id<R: Row + Send>(&self, table: &str, row: R) -> Result<(), Error>;

    async fn insert_return_id<T: Row + Send>(&self, table: &str, row: T) -> Result<i32, Error>;

    async fn delete_by_id_required(&self, table: &str, id: i32) -> Result<(), Error>;

    async fn delete_by_column<V: Send>(&self, table: &str, column: &str, column_val: V) -> Result<(), Error>
        where
            sea_query::Value: From<V>;

    async fn delete_insert_return_id_transaction<T: Row + Send>(&self, table: &str, id_delete: i32, row: T) -> Result<i32, Error>;
}

pub struct PSQL {
    pub(crate) pool: Pool<Postgres>
}

// TODO make a macro for this
pub trait Row {
    fn keys(&self, include_id: bool) -> &str;
    fn vals(&self, include_id: bool) -> &str;
    fn set(&self) -> &str;
    fn values(&self, include_id: bool) -> Values;
}

#[async_trait]
impl Database for PSQL {
    async fn retrieve_by_id<T>(&self, table: &str, id: i32) -> Result<Option<T>, Error>
        where
            T: for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let query = format!("SELECT * FROM {table} WHERE id = $1", table=table);
        let row: Option<T> = sqlx::query_as(
            &query
        )
            .bind(id)
            .fetch_optional(&self.pool).await?;
        Ok(row)
    }

    async fn retrieve_by_unique<T>(&self, table: &str, unique_column: &str, value: Values) -> Result<Option<T>, Error>
        where
            T: for<'r> FromRow<'r, PgRow> + Send + Unpin
    {
        let query = format!("SELECT * FROM {table} WHERE {column} = $1", table=table, column=unique_column);
        let row: Option<T> = bind_query_as(sqlx::query_as(&query), &value).fetch_optional(&self.pool).await?;
        Ok(row)
    }

    async fn upsert_by_id<T: Row + Send>(&self, table: &str, row: T) -> Result<(), Error> {
        let query = format!("INSERT INTO {table} ({keys}) VALUES ({vals}) ON CONFLICT (id)\
            DO UPDATE SET {set}", table=table, keys=row.keys(true), vals=row.vals(true), set=row.set());
        let values = row.values(true).to_owned();
        bind_query(sqlx::query(&query), &values).execute(&self.pool).await?;
        Ok(())
    }

    async fn insert_return_id<T: Row + Send>(&self, table: &str, row: T) -> Result<i32, Error> {
        psql_insert_return_id::<&Pool<Postgres>, T>(&self.pool, table, row).await
    }

    /// Returns error if row does not exist
    async fn delete_by_id_required(&self, table: &str, id: i32) -> Result<(), Error> {
        psql_delete_by_id_required::<&Pool<Postgres>>(&self.pool, table, id).await
    }

    /// Don't allow users to send "column" as it is not injection-protected
    /// If there are no rows, there is no error
    async fn delete_by_column<V: Send>(&self, table: &str, column: &str, column_val: V) -> Result<(), Error>
        where
            sea_query::Value: From<V>
    {
        let query = format!("DELETE FROM {table} WHERE {column} = $1", table=table, column=column);
        let values = Values(vec![Value::from(column_val)]);
        bind_query(sqlx::query(&query), &values).execute(&self.pool).await?;
        Ok(())
    }

    async fn delete_insert_return_id_transaction<T: Row + Send>(&self, table: &str, id_delete: i32, row: T) -> Result<i32, Error> {
        let mut tx = self.pool.begin().await?;
        psql_delete_by_id_required::<&mut Transaction<Postgres>>(&mut tx, table, id_delete).await?;
        let id = psql_insert_return_id::<&mut Transaction<Postgres>, T>(&mut tx, table, row).await?;
        tx.commit().await?;
        Ok(id)
    }
}

async fn psql_delete_by_id_required<'a, E>(exec: E, table: &str, id: i32) -> Result<(), Error>
    where
        E: Executor<'a, Database=Postgres>
{
    let query = format!("WITH deleted as (DELETE FROM {table} WHERE id = $1 returning 1)
                                SELECT count(*) from deleted;", table=table);
    let count: (i64,) = sqlx::query_as(
        &query
    )
        .bind(id).fetch_one(exec)
        .await?;
    if count.0 == 0 {
        return Err(Error::NoRow)
    }
    Ok(())
}

async fn psql_insert_return_id<'a, E, T: Row + Send>(exec: E, table: &str, row: T) -> Result<i32, Error>
    where
        E: Executor<'a, Database=Postgres>
{
    let query = format!("INSERT INTO {table} ({keys}) VALUES ({vals}) RETURNING (id)",
                        table=table, keys=row.keys(false), vals=row.vals(false));
    let values = row.values(false).to_owned();
    let id: (i32,) = bind_query_as(sqlx::query_as(&query), &values).fetch_one(exec).await?;
    Ok(id.0)
}