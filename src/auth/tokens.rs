use jsonwebtoken::Algorithm::ED448;
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use crate::data::key;
use crate::data::source::Source;
use crate::auth::auth::{symmetric_crypt, symmetric_decrypt};
use crate::config::{AUD, ISS};
use crate::data::refresh::{delete_family, get_refresh_by_id, refresh_save, refresh_transaction, SavedRefreshToken};
use crate::error::Error;
use crate::utility::{dec_b64url, enc_b64url, enc_struct, rng_urlsafe, utc_timestamp};

const ID_EXP: u64 = 10 * 60 * 60;
pub const ACCESS_EXP: u64 = 1 * 60 * 60;
const REFRESH_EXP: i32 = 1 * 60 * 60;

const GRACE_PERIOD: i32 = 3 * 60;

#[derive(Serialize, Deserialize)]
struct RefreshToken {
    pub id: i32,
    pub family_id: String,
    pub nonce: String
}

#[derive(Serialize, Deserialize)]
struct AccessTokenUntimed {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub scope: String
}

#[derive(Serialize, Deserialize)]
struct IdTokenUntimed {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub auth_time: u64,
    pub nonce: String
}

#[derive(Serialize, Deserialize)]
struct AccessToken {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub scope: String,
    pub iat: u64,
    pub exp: u64
}

#[derive(Serialize, Deserialize)]
struct IdToken {
    pub sub: String,
    pub iss: String,
    pub aud: Vec<String>,
    pub auth_time: u64,
    pub nonce: String,
    pub iat: u64,
    pub exp: u64
}

pub struct Tokens {
    pub access_token: String,
    pub id_token: String,
    pub refresh_token: String,
    pub returned_scope: String
}

async fn get_private_key_bytes(dsrc: &Source) -> Result<Vec<u8>, Error> {
    let key = key::get_token_private(&dsrc).await?;
    Ok(key.into_bytes())
}

async fn get_symmetric_key_bytes(dsrc: &Source) -> Result<Vec<u8>, Error> {
    let key = key::get_refresh_symmetric(&dsrc).await?;
    Ok(dec_b64url(key)?)
}

fn decrypt_refresh_token(symmetric_key: &[u8], refresh_token: String) -> Result<RefreshToken, Error> {
    let refresh_bytes = dec_b64url(refresh_token)?;
    let refresh = symmetric_decrypt(symmetric_key, refresh_bytes)?;
    Ok(serde_json::from_slice(&refresh)?)
}

fn encrypt_refresh_token(symmetric_key: &[u8], refresh_token: RefreshToken) -> Result<String, Error> {
    let refresh_bytes = serde_json::to_vec(&refresh_token)?;
    let refresh = symmetric_crypt(symmetric_key, refresh_bytes)?;
    Ok(enc_b64url(&refresh))
}

fn get_finish_tokens_from_save(saved_refresh: &SavedRefreshToken, utc_now: u64) -> Result<(AccessToken, IdToken), Error> {
    let at_bytes = dec_b64url(saved_refresh.access_value.clone())?;
    let it_bytes = dec_b64url(saved_refresh.id_token_value.clone())?;

    let at: AccessTokenUntimed = serde_json::from_slice(&at_bytes)?;
    let it: IdTokenUntimed = serde_json::from_slice(&it_bytes)?;

    get_finish_tokens(at, it, utc_now)
}

fn get_finish_tokens(at: AccessTokenUntimed, it: IdTokenUntimed, utc_now: u64) -> Result<(AccessToken, IdToken), Error> {
    let at = AccessToken { iat: utc_now, exp: utc_now + ACCESS_EXP,
        sub: at.sub,
        iss: at.iss,
        aud: at.aud,
        scope: at.scope
    };

    let it = IdToken { iat: utc_now, exp: utc_now + ACCESS_EXP,
        sub: it.sub,
        iss: it.iss,
        aud: it.aud,
        auth_time: it.auth_time,
        nonce: it.nonce
    };

    Ok((at, it))
}

fn new_refresh(old_refresh: SavedRefreshToken, utc_now: u64) -> Result<(SavedRefreshToken, String), Error> {
    let nonce = rng_urlsafe(16);

    Ok((SavedRefreshToken { nonce: nonce.clone(), iat: utc_now as i32, ..old_refresh }, nonce))
}

async fn new_refresh_save(dsrc: &Source, old_refresh: SavedRefreshToken, utc_now: u64, symmetric_key: &[u8]) -> Result<String, Error> {
    let (new_saved, nonce) = new_refresh(old_refresh.clone(), utc_now)?;
    let new_refresh_id = refresh_transaction(dsrc, old_refresh.id, &new_saved).await?;
    let refresh_token = RefreshToken {
        id: new_refresh_id,
        family_id: old_refresh.family_id,
        nonce
    };

    Ok(encrypt_refresh_token(symmetric_key, refresh_token)?)
}

fn id_access_token(sub: &str, iss: &str, aud_access: Vec<String>, aud_id: Vec<String>, scope: &str, auth_time: u64, id_nonce: &str) -> (AccessTokenUntimed, IdTokenUntimed) {
    let at = AccessTokenUntimed {
        sub: sub.to_owned(),
        iss: iss.to_owned(),
        aud: aud_access,
        scope: scope.to_owned()
    };
    let it = IdTokenUntimed {
        sub: sub.to_owned(),
        iss: iss.to_owned(),
        aud: aud_id,
        auth_time,
        nonce: id_nonce.to_owned()
    };
    (at, it)
}

pub async fn refresh_all_tokens(dsrc: &Source, old_refresh_token: String) -> Result<Tokens, Error> {
    let private_key = get_private_key_bytes(dsrc).await?;
    tracing::debug!("got private key");
    let symmetric_key = get_symmetric_key_bytes(dsrc).await?;
    tracing::debug!("got symmetric key");
    let old_refresh = decrypt_refresh_token(&symmetric_key, old_refresh_token)?;
    tracing::debug!("refresh_token decrypted");
    let utc_now = utc_timestamp();


    let saved_refresh = match get_refresh_by_id(&dsrc, old_refresh.id).await {
        Ok(saved_refresh) => Ok(saved_refresh),
        Err(error) => match error {
            Error::NoRow => {
                delete_family(&dsrc, &old_refresh.family_id).await?;
                return Err(Error::InvalidRefresh)
            }
            e => Err(e)
        }
    }?;

    if saved_refresh.nonce != old_refresh.nonce || saved_refresh.family_id != old_refresh.family_id {
        return Err(Error::InvalidRefresh)
    }
    if saved_refresh.iat as i128 > utc_now as i128 || saved_refresh.iat < 1640690242 {
        return Err(Error::InvalidRefresh)
    }
    if utc_now as i128 > (saved_refresh.exp + GRACE_PERIOD) as i128 {
        return Err(Error::InvalidRefresh)
    }

    let (at, it) = get_finish_tokens_from_save(&saved_refresh, utc_now)?;

    let access_token = encode_token(&private_key, &at)?;
    let id_token = encode_token(&private_key, &it)?;

    let refresh_token = new_refresh_save(dsrc, saved_refresh, utc_now, &symmetric_key).await?;

    Ok(Tokens { access_token, id_token, refresh_token, returned_scope: at.scope })
}

pub async fn new_token_family(dsrc: &Source, user_usph: String, scope: String, id_nonce: String, auth_time: u64) -> Result<Tokens, Error> {
    let private_key = key::get_token_private(dsrc).await?;
    let symmetric_key = get_symmetric_key_bytes(dsrc).await?;
    let utc_now = utc_timestamp();

    let aud_vec: Vec<String> = AUD.to_vec().iter().map(|s| (*s).to_owned()).collect();

    let (at, it) = id_access_token(
        &user_usph,
            ISS,
        aud_vec.clone(),
        aud_vec,
        &scope,
        auth_time,
        &id_nonce
    );

    let at_enc = enc_struct(&at)?;
    let it_enc = enc_struct(&it)?;
    let family_id = rng_urlsafe(16);

    let refresh_saved = SavedRefreshToken {
        id: 0,
        family_id,
        access_value: at_enc,
        id_token_value: it_enc,
        iat: utc_now as i32,
        exp: utc_now as i32 + REFRESH_EXP,
        nonce: "".to_string()
    };
    let refresh_id = refresh_save(&dsrc, &refresh_saved).await?;

    let refresh = RefreshToken {
        id: refresh_id,
        family_id: refresh_saved.family_id,
        nonce: refresh_saved.nonce
    };
    let refresh_token = encrypt_refresh_token(&symmetric_key, refresh)?;
    let (at_fin, it_fin) = get_finish_tokens(at, it, utc_now)?;

    let access_token = encode_token(private_key.as_bytes(), &at_fin)?;
    let id_token = encode_token(private_key.as_bytes(), &it_fin)?;

    Ok(Tokens { access_token, id_token, refresh_token, returned_scope: scope })
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
            sub: "ab".to_string(),
            iss: "".to_string(),
            aud: vec![],
            scope: "".to_string(),
            iat: 0,
            exp: 0
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