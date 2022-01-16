use std::sync::Arc;
use axum::extract::{Extension, Query};
use axum::Json;
use axum::response::{Redirect};
use url::form_urlencoded::{byte_serialize};
use encoding::{Encoding, EncoderTrap};
use encoding::all::ASCII;
use sha2::{Digest, Sha256};
use sha2::digest::DynDigest;
use base64;
use crate::data::kv::KeyValue;
use crate::server::models::{AuthRequest, FlowUser, OAuthFinish, TokenRequest, TokenResponse};
use crate::data::source::Source;
use crate::error::Error;
use crate::error::BadFlow::{ExpiredFlowId, BadChallenge, ExpiredCode};
use crate::utility::random_time_hash_hex;

pub async fn oauth_endpoint(Query(auth_request): Query<AuthRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let flow_id = random_time_hash_hex(None);

    dsrc.kv.store_json(&flow_id, &auth_request, 1000).await?;

    Ok(Redirect::to(format!("/credentials?flow_id={}", flow_id).parse().unwrap()))
}

pub async fn oauth_finish(Query(oauth_finish): Query<OAuthFinish>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let auth_request: AuthRequest = dsrc.kv.get_json(&oauth_finish.flow_id).await?
        .ok_or(Error::BadFlow(ExpiredFlowId))?;

    let state_encoded: String = byte_serialize(format!("{}", auth_request.state).as_bytes()).collect();
    let redirect = if !auth_request.redirect_uri.ends_with("/") {
        format!("{}/", auth_request.redirect_uri)
    } else {
        auth_request.redirect_uri
    };
    Ok(Redirect::to(format!("{}?code={}&state={}", redirect, oauth_finish.code, state_encoded).parse().unwrap()))
}

pub async fn token(Json(token_request): Json<TokenRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Json<TokenResponse>, Error> {
    // TODO clientID check

    if token_request.grant_type == "authorization_code" {
        let redirect_uri = token_request.redirect_uri.ok_or(Error::MissingFieldTokenRequest)?;
        let code_verifier = token_request.code_verifier.ok_or(Error::MissingFieldTokenRequest)?;
        let code = token_request.code.ok_or(Error::MissingFieldTokenRequest)?;

        let flow_user: FlowUser = dsrc.kv.get_json(&code).await?
            .ok_or(Error::BadFlow(ExpiredCode))?;
        let auth_request: AuthRequest = dsrc.kv.get_json(&flow_user.flow_id).await?
            .ok_or(Error::BadFlow(ExpiredFlowId))?;

        if token_request.client_id != auth_request.client_id {
            return Err(Error::IncorrectField("client_id does not match".to_owned()))
        }
        if redirect_uri != auth_request.redirect_uri {
            return Err(Error::IncorrectField("redirect_uri does not match".to_owned()))
        }

        let ascii_encoded = ASCII.encode(&code_verifier, EncoderTrap::Strict)
            .or_else(|e| Err(Error::BadFieldEncoding(e.to_string())))?;
        let computed_challenge_hash = Sha256::digest(&ascii_encoded);
        let challenge = base64::encode(computed_challenge_hash);
        if challenge != auth_request.code_challenge {
            return Err(Error::BadFlow(BadChallenge))
        }
    }
    Ok(Json(TokenResponse {
        id_token: "".to_string(),
        access_token: "".to_string(),
        refresh_token: "".to_string(),
        token_type: "".to_string(),
        expires_in: 0,
        scope: "".to_string()
    }))
}