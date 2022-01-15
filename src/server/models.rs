use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub nonce: String
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TJson {
    pub cool: String
}

#[derive(Deserialize)]
pub struct PasswordRequest {
    pub username: String,
    pub client_request: String
}

#[derive(Serialize)]
pub struct PasswordResponse {
    pub server_message: String,
    pub auth_id: String
}