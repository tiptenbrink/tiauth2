use sea_query::{Value, Values};
use sqlx::{Encode, FromRow, Postgres, Type};
use sqlx::postgres::PgRow;
use sqlx::query::Query;
use crate::data::db;
use crate::data::db::{Database, Row};
use crate::data::source::Source;
use crate::error::Error;

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub usp_hex: String,
    pub id: i32,
    pub password_file: String
}

impl Row for User {
    fn keys(&self, include_id: bool) -> &str {
        if include_id {
            "id, usp_hex, password_file"
        } else {
            "usp_hex, password_file"
        }
    }

    fn vals(&self, include_id: bool) -> &str {
        if include_id {
            "$1, $2, $3"
        } else {
            "$1, $2"
        }
    }

    fn set(&self) -> &str {
        "id = $1, usp_hex = $2, password_file = $3"
    }

    fn values(&self, include_id: bool) -> Values {
        let mut val_vec = vec![
            Value::from(self.usp_hex.clone()),
            Value::from(self.password_file.clone()),
        ];

        if include_id {
            val_vec.insert(0, Value::from(self.id));
        }

        Values(val_vec)
    }
}

pub async fn get_user_by_id(dsrc: &Source, id: i32) -> Result<User, Error> {
    dsrc.db.retrieve_by_id::<User>("users", id).await
}

pub async fn get_user_by_usph(dsrc: &Source, usp_hex: &str) -> Result<User, Error> {
    let val = Values(vec![Value::from(usp_hex)]);
    dsrc.db.retrieve_by_unique::<User>("users", "usp_hex", val).await
}

pub async fn upsert_user_row(dsrc: &Source, row: User) -> Result<(), Error> {
    dsrc.db.upsert_by_id("users", row).await
}

pub async fn new_user(dsrc: &Source, row: User) -> Result<i32, Error> {
    dsrc.db.insert_return_id("users", row).await
}