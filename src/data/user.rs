use sqlx::{Encode, FromRow, Postgres, Type};
use sqlx::postgres::PgRow;
use sqlx::query::Query;
use crate::data::db;
use crate::data::db::{Database, Row, VV};
use crate::data::source::Source;
use crate::error::Error;

#[derive(sqlx::FromRow, Debug)]
pub struct User { usp_hex: String, id: i32, password_file: String }

#[derive(Clone)]
enum UserEnum {
    Id(i32),
    UspHex(String),
    PasswordFile(String)
}


impl Row for User {
    type RowModel = UserEnum;

    fn id(&self) -> i32 {
        self.id
    }

    fn keys(&self) -> String {
        format!("id, usp_hex, password_file")
    }

    fn vals(&self) -> String {
        format!("$1, $2, $3")
    }

    fn vals_vec(&self) -> Vec<Box<dyn VV>> {
        vec![Box::new(self.id)]
    }

    fn set(&self) -> String {
        format!("id = $1, usp_hex = $2, password_file = $3")
    }
}

pub async fn get_user_by_id(dsrc: Source, id: i32) -> Result<User, Error> {
    dsrc.db.retrieve_by_id::<User>("users", id).await
}

pub async fn upsert_user_row(dsrc: Source, row: User) -> Result<(), Error> {
    dsrc.db.upsert_by_id("users", row).await
}