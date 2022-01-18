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
    let key_bytes = base64::decode_config(key, base64::URL_SAFE_NO_PAD)?;
    let crypt_bytes = symmetric_crypt(key_bytes.as_slice(), data)?;
    Ok(base64::encode_config(crypt_bytes, base64::URL_SAFE_NO_PAD))
}

fn symmetric_crypt(key: &[u8], mut data: Vec<u8>) -> Result<Vec<u8>, Error> {
    let unbound_key = aead::UnboundKey::new(&AES_256_GCM, key)?;
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let mut symmetric_bytes = nonce_bytes.clone().to_vec();
    let nonce = Nonce::assume_unique_for_key(nonce_bytes);
    let encrypter = aead::LessSafeKey::new(unbound_key);
    encrypter.seal_in_place_append_tag(nonce, Aad::empty(), &mut data)?;
    symmetric_bytes.extend(data);

    // out: nonce (12 bytes) + ciphertext + tag appended
    Ok(symmetric_bytes)
}

fn symmetric_decrypt(key: &[u8], encrypted: Vec<u8>) -> Result<Vec<u8>, Error> {
    // encrypted: nonce (12 bytes) + ciphertext + tag appended
    let unbound_key = aead::UnboundKey::new(&AES_256_GCM, key)?;

    if encrypted.len() < 12 {
        return Err(Error::BadCryptInput)
    }
    let (nonce_slice, crypt) = encrypted.split_at(12);
    let nonce_sized = <[u8; 12]>::try_from(nonce_slice.clone()).map_err(|e| Error::BadCryptInput)?;
    let nonce = Nonce::assume_unique_for_key(nonce_sized);
    let decrypter = aead::LessSafeKey::new(unbound_key);
    let mut crypt = crypt.to_vec();
    let out = decrypter.open_in_place(nonce, Aad::empty(), &mut crypt)?;
    Ok(out.to_vec())
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
        let input = "hello".to_owned();
        let key = new_symmetric_keypair();
        let key_bytes = base64::decode_config(&key.private, base64::URL_SAFE_NO_PAD).unwrap();
        let x = symmetric_crypt(key_bytes.as_slice(), (&input).as_bytes().to_vec()).unwrap();
        //println!("{}", base64::encode_config(&x, base64::URL_SAFE_NO_PAD));
        let z = symmetric_decrypt(key_bytes.as_slice(), x).unwrap();
        let u = String::from_utf8(z).unwrap();
        assert_eq!(input, u)
    }
}