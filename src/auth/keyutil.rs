use opaquebind::generate_keys;

use crate::data::Key;

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