use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use base64::DecodeError;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use once_cell::sync::Lazy;
use rand::{RngCore};
use serde::Serialize;
use serde::de::DeserializeOwned;
use crate::error::Error;


pub fn utc_timestamp() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        .as_secs()
}


pub fn random_time_hash_hex(extra_seed: Option<&[u8]>) -> String {
    let timestamp = utc_timestamp().to_be_bytes().to_vec();
    let mut random_rng = OsRng;
    let mut random_vec: Vec<u8> = vec![0; 8];
    random_rng.fill_bytes(&mut random_vec);

    let extra_seed = extra_seed.map_or_else(|| vec![], |e| e.to_vec());

    let combined = [timestamp, extra_seed, random_vec].concat();
    let mut hash = Sha256::new();
    hash.update(combined);
    let random = hash.finalize().to_vec();
    hex::encode(random)
}

static URLSAFE: Lazy<Mutex<HashSet<u8>>> = Lazy::new(|| {
    let chr_bytes = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_".as_bytes().iter().cloned();
    let s = HashSet::from_iter(chr_bytes);
    Mutex::new(s)
});

pub fn usp_hex(utf_str: &str) -> String {
    let mut anp_base6url_str = "".to_owned();
    let enc_str = utf_str.to_owned().into_bytes();
    for bt in enc_str {
        if URLSAFE.lock().unwrap().contains(&bt) {
            anp_base6url_str = format!("{}{}", anp_base6url_str, String::from_utf8(vec![bt]).unwrap())
        } else {
            anp_base6url_str = format!("{}~{}", anp_base6url_str, hex::encode([bt]))
        }
    }
    anp_base6url_str
}

pub fn enc_b64url<T: AsRef<[u8]>>(to_enc: T) -> String {
    base64::encode_config(to_enc, base64::URL_SAFE_NO_PAD)
}

pub fn dec_b64url<T: AsRef<[u8]>>(to_dec: T) -> Result<Vec<u8>, DecodeError> {
    base64::decode_config(to_dec, base64::URL_SAFE_NO_PAD)
}

pub fn enc_struct<T>(strct: &T) -> Result<String, Error>
    where
        T: ?Sized + Serialize
{
    let strct_bytes = serde_json::to_vec(strct)?;
    Ok(enc_b64url(&strct_bytes))
}

pub fn dec_struct<'a, T: AsRef<[u8]>, V>(strct_str: T) -> Result<V, Error>
    where
        V: DeserializeOwned
{
    let strct_bytes = dec_b64url(strct_str)?;
    Ok(serde_json::from_slice::<V>(&strct_bytes)?)
}

pub fn rng_urlsafe(n_bytes: usize) -> String {
    let mut rng_bytes = vec![0; n_bytes];
    OsRng.fill_bytes(&mut rng_bytes);
    base64::encode_config(&rng_bytes, base64::URL_SAFE_NO_PAD)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usp_hex() {
        assert_eq!(usp_hex("ka25kja5kasdf;lkja@@@!!!😂s"),
        "ka25kja5kasdf~3blkja~40~40~40~21~21~21~f0~9f~98~82s".to_string());
    }
}