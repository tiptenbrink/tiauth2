use opaquebind::generate_keys;
use openssl::pkey::PKey;
use rand::RngCore;
use rand::rngs::OsRng;

use crate::data::Key;
use crate::utility::enc_b64url;

pub fn new_curve25519_keypair() -> Key {
    let (private, public) = generate_keys();

    Key {
        id: 0,
        algorithm: "curve25519ristretto".to_string(),
        public,
        private,
        public_format: "none".to_string(),
        public_encoding: "base64url".to_string(),
        private_format: "none".to_string(),
        private_encoding: "base64url".to_string()
    }
}

pub fn new_ed448_keypair() -> Key {
    let private_key = PKey::generate_ed448().unwrap();
    let public_bytes = private_key.public_key_to_pem().unwrap();
    let public = String::from_utf8(public_bytes).unwrap();
    let private_bytes = private_key.private_key_to_pem_pkcs8().unwrap();
    let private = String::from_utf8(private_bytes).unwrap();

    Key {
        id: 1,
        algorithm: "ed448".to_string(),
        public,
        private,
        public_format: "X509PKCS#1".to_string(),
        public_encoding: "PEM".to_string(),
        private_format: "PKCS#8".to_string(),
        private_encoding: "PEM".to_string()
    }
}

pub fn new_symmetric_keypair() -> Key {
    let mut symmetric_bytes = [0u8; 32];
    OsRng.fill_bytes(&mut symmetric_bytes);
    let symmetric = enc_b64url(symmetric_bytes);

    Key {
        id: 2,
        algorithm: "symmetric".to_string(),
        public: "".to_string(),
        private: symmetric,
        public_format: "".to_string(),
        public_encoding: "".to_string(),
        private_format: "none".to_string(),
        private_encoding: "base64url".to_string()
    }
}