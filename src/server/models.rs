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

#[derive(Deserialize, Serialize)]
pub struct SavedState {
    pub user_usph: String,
    pub state: String
}

#[derive(Deserialize)]
pub struct FinishLogin {
    pub auth_id: String,
    pub username: String,
    pub client_request: String,
    pub flow_id: String
}

#[derive(Deserialize, Serialize)]
pub struct FlowUser {
    pub user_usph: String,
    pub flow_id: String,
    pub auth_time: u64
}

#[derive(Deserialize)]
pub struct FinishRegister {
    pub auth_id: String,
    pub username: String,
    pub client_request: String
}

#[derive(Deserialize)]
pub struct OAuthFinish {
    pub flow_id: String,
    pub code: String
}

#[derive(Deserialize)]
pub struct TokenRequest {
    pub client_id: String,
    pub grant_type: String,
    pub code: Option<String>,
    pub redirect_uri: Option<String>,
    pub code_verifier: Option<String>,
    pub refresh_token: Option<String>
}

#[derive(Serialize)]
pub struct TokenResponse {
    pub id_token: String,
    pub access_token: String,
    pub refresh_token: String,
    pub token_type: String,
    pub expires_in: i32,
    pub scope: String,
}
