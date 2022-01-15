use sea_query::{Value, Values};
use crate::data::db::{Database, Row};
use crate::data::source::Source;
use crate::error::Error;
use crate::auth::keyutil::new_curve25519_keypair;

#[derive(sqlx::FromRow, Debug, Clone)]
pub struct Key {
    pub id: i32,
    pub algorithm: String,
    pub public: String,
    pub private: String,
    pub public_format: String,
    pub public_encoding: String,
    pub private_format: String,
    pub private_encoding: String,
}

impl<'a> Row for &'a Key {
    fn keys(&self, include_id: bool) -> &str {
        if include_id {
            "id, algorithm, public, private, public_format, public_encoding, private_format, private_encoding"
        } else {
            "algorithm, public, private, public_format, public_encoding, private_format, private_encoding"
        }
    }

    fn vals(&self, include_id: bool) -> &str {
        if include_id {
            "$1, $2, $3, $4, $5, $6, $7, $8"
        } else {
            "$1, $2, $3, $4, $5, $6, $7"
        }
    }

    fn set(&self) -> &str {
        "id = $1, algorithm = $2, public = $3, private = $4, public_format = $5, public_encoding = $6, private_format = $7, private_encoding = $8"
    }

    fn values(&self, include_id: bool) -> Values {
        let mut val_vec = vec![
            Value::from(self.algorithm.clone()),
            Value::from(self.public.clone()),
            Value::from(self.private.clone()),
            Value::from(self.public_format.clone()),
            Value::from(self.public_encoding.clone()),
            Value::from(self.private_format.clone()),
            Value::from(self.private_encoding.clone()),
        ];

        if include_id {
            val_vec.insert(0, Value::from(self.id));
        }

        Values(val_vec)
    }
}

async fn get_key_row(dsrc: &Source, id: i32) -> Result<Option<Key>, Error> {
    dsrc.db.retrieve_by_id::<Key>("keys", id).await
}

pub async fn get_opaque_private(dsrc: &Source) -> Result<String, Error> {
    Ok(get_opaque_key(dsrc).await?.private)
}

async fn get_opaque_key(dsrc: &Source) -> Result<Key, Error> {
    let key_row = get_key_row(dsrc, 0).await?;
    let empty = key_row.is_none();
    let key = key_row.unwrap_or_else(|| {
        let key_row = new_curve25519_keypair();
        key_row
    });
    if empty {
        upsert_key_row(dsrc, &key).await?;
    }

    Ok(key)
}

pub async fn upsert_key_row(dsrc: &Source, row: &Key) -> Result<(), Error> {
    dsrc.db.upsert_by_id("keys", row).await
}