use sea_query::{Value, Values};
use serde::Deserialize;
use crate::data::db::{Database, Row};
use crate::data::source::Source;
use crate::error::Error;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct SavedRefreshToken {
    pub id: i32,
    pub family_id: String,
    pub access_value: String,
    pub id_token_value: String,
    pub iat: i32,
    pub exp: i32,
    pub nonce: String,
}

impl<'a> Row for &'a SavedRefreshToken {
    fn keys(&self, include_id: bool) -> &str {
        if include_id {
            "id, family_id, access_value, id_token_value, iat, exp, nonce"
        } else {
            "family_id, access_value, id_token_value, iat, exp, nonce"
        }
    }

    fn vals(&self, include_id: bool) -> &str {
        if include_id {
            "$1, $2, $3, $4, $5, $6, $7"
        } else {
            "$1, $2, $3, $4, $5, $6"
        }
    }

    fn set(&self) -> &str {
        "id = $1, family_id = $2, access_value = $3, id_token_value = $4, iat =$5, exp = $6, nonce = $7"
    }

    fn values(&self, include_id: bool) -> Values {
        let mut val_vec = vec![
            Value::from(self.family_id.clone()),
            Value::from(self.access_value.clone()),
            Value::from(self.id_token_value.clone()),
            Value::from(self.iat),
            Value::from(self.exp),
            Value::from(self.nonce.clone()),
        ];

        if include_id {
            val_vec.insert(0, Value::from(self.id));
        }

        Values(val_vec)
    }
}

pub async fn refresh_save(dsrc: &Source, row: &SavedRefreshToken) -> Result<i32, Error> {
    dsrc.db.insert_return_id("refreshtokens", row).await
}

pub async fn get_refresh_by_id(dsrc: &Source, id: i32) -> Result<SavedRefreshToken, Error> {
    dsrc.db.retrieve_by_id("refreshtokens", id).await?.ok_or(Error::NoRow)
}

pub async fn refresh_transaction(dsrc: &Source, id_delete: i32, new_refresh: &SavedRefreshToken) -> Result<i32, Error> {
    // CURRENTLY DOES NOT FAIL IF IT DOES NOT EXIST
    // TODO add check in query delete if it did delete
    dsrc.db.delete_insert_return_id_transaction("refreshtokens", id_delete, new_refresh).await
}

pub async fn delete_family(dsrc: &Source, family_id: &str) -> Result<(), Error> {
    dsrc.db.delete_by_column::<>("refreshtokens", "family_id", family_id).await
}