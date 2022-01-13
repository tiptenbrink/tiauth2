use std::time::{SystemTime, UNIX_EPOCH};
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};
use hex::{decode};

pub fn random_time_hash_hex(extra_seed: Option<Vec<u8>>) -> String {
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
        .as_secs().to_be_bytes().to_vec();
    let mut random_rng = OsRng;
    let mut random_vec: Vec<u8> = vec![0; 8];
    random_rng.fill_bytes(&mut random_vec);

    let extra_seed = extra_seed.unwrap_or(vec![]);

    let combined = [timestamp, extra_seed, random_vec].concat();
    let mut hash = Sha256::new();
    hash.update(combined);
    let random = hash.finalize().to_vec();
    hex::encode(random)
}