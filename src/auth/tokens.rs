use jsonwebtoken::Algorithm::ED448;
use jsonwebtoken::{encode, Header, EncodingKey};
use jsonwebtoken::errors::ErrorKind;
use rand::RngCore;
use rand::rngs::OsRng;
use serde::Serialize;
use ring::aead;
use ring::aead::{Aad, AES_256_GCM, BoundKey, Nonce};
use crate::data::key;
use crate::data::source::Source;


use crate::error::Error;

pub struct Tokens {
    pub access_token: String,
}

#[derive(Serialize)]
struct AccessToken {
    pub sub: String,
}

async fn symmetric_crypt_key(dsrc: &Source, mut data: Vec<u8>) -> Result<String, Error> {
    let key = key::get_refresh_symmetric(&dsrc).await?;
    symmetric_crypt(&key, data)
}

fn symmetric_crypt(key: &str, mut data: Vec<u8>) -> Result<String, Error> {
    let key_bytes = base64::decode_config(key, base64::URL_SAFE_NO_PAD)?;
    let unbound_key = aead::UnboundKey::new(&AES_256_GCM, &key_bytes)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let mut symmetric_bytes = nonce_bytes.clone().to_vec();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let encrypter = aead::LessSafeKey::new(unbound_key);
    encrypter.seal_in_place_append_tag(nonce, Aad::empty(), &mut data);
    symmetric_bytes.extend(data);

    Ok(base64::encode_config(symmetric_bytes, base64::URL_SAFE_NO_PAD))
}

pub async fn refresh_all_tokens(dsrc: &Source) -> Result<Tokens, Error> {

    Ok(Tokens { access_token: "".to_string() })
}

pub async fn new_token_family(dsrc: &Source) -> Result<Tokens, Error> {
    let private_key = key::get_token_private(dsrc).await?;

    let at = AccessToken {
        sub: "ab".to_string()
    };

    let enc = encode_token(private_key.as_bytes(), &at)?;

    Ok(Tokens { access_token: enc })
}

pub fn encode_token<T: Serialize>(private_key: &[u8], claims: &T) -> Result<String, Error>{
    let header = Header::new(ED448);
    let encoding_key = EncodingKey::from_ed_pem(private_key)?;
    Ok(encode(&header, claims, &encoding_key)?)
}

#[cfg(test)]
mod tests {
    use crate::auth::keyutil::new_symmetric_keypair;
    use super::*;

    #[test]
    fn test_token() {
        let at = AccessToken {
            sub: "ab".to_string()
        };
        let ed25519 = "-----BEGIN PRIVATE KEY-----\n\
            MC4CAQAwBQYDK2VwBCIEIHZ4+VqCXpwjjlv439/zsrKHcWJej0ZgJt4XaJ7Lxd8/\n\
            -----END PRIVATE KEY-----".to_string();
        let ed448 = "-----BEGIN PRIVATE KEY-----\n\
            MEcCAQAwBQYDK2VxBDsEOV8K6nOltf9IEE+xHw7HY9bwrPyjEu3+RHYMMEgS6QTJ\n\
            w1dLURYlIrYYxX9N52B5n/U2aF1owL0xDg==\n\
            -----END PRIVATE KEY-----".to_string();
        encode_token(ed448.as_bytes(), &at).unwrap();
        encode_token(ed25519.as_bytes(), &at).unwrap();
    }

    #[test]
    fn test_encrypt() {
        let key = new_symmetric_keypair();
        let x = symmetric_crypt(&key.private, "hello".to_owned().into_bytes()).unwrap();
        println!("{}", x);
    }
}