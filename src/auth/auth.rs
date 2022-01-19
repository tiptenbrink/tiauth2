use rand::RngCore;
use rand::rngs::OsRng;
use ring::aead;
use ring::aead::{Aad, AES_256_GCM, Nonce};
use crate::error::Error;

pub fn symmetric_crypt(key: &[u8], mut data: Vec<u8>) -> Result<Vec<u8>, Error> {
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

pub fn symmetric_decrypt(key: &[u8], encrypted: Vec<u8>) -> Result<Vec<u8>, Error> {
    // encrypted: nonce (12 bytes) + ciphertext + tag appended
    let unbound_key = aead::UnboundKey::new(&AES_256_GCM, key)?;

    if encrypted.len() < 12 {
        return Err(Error::BadCryptInput)
    }
    let (nonce_slice, crypt) = encrypted.split_at(12);
    let nonce_sized = <[u8; 12]>::try_from(nonce_slice.clone()).map_err(|_e| Error::BadCryptInput)?;
    let nonce = Nonce::assume_unique_for_key(nonce_sized);
    let decrypter = aead::LessSafeKey::new(unbound_key);
    let mut crypt = crypt.to_vec();
    let out = decrypter.open_in_place(nonce, Aad::empty(), &mut crypt)?;
    Ok(out.to_vec())
}