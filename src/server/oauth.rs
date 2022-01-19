use std::sync::Arc;
use axum::extract::{Extension, Query};
use axum::Json;
use axum::response::{Redirect};
use url::form_urlencoded::{byte_serialize};
use encoding::{Encoding, EncoderTrap};
use encoding::all::ASCII;
use sha2::{Digest, Sha256};
use base64;
use crate::auth::tokens;
use crate::auth::tokens::{new_token_family, refresh_all_tokens};
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

fn oauth_finish_redirect(auth_request: AuthRequest, code: String) -> String {
    let state_encoded: String = byte_serialize(format!("{}", auth_request.state).as_bytes()).collect();
    let redirect = if !auth_request.redirect_uri.ends_with("/") {
        format!("{}/", auth_request.redirect_uri)
    } else {
        auth_request.redirect_uri
    };
    format!("{}?code={}&state={}", redirect, code, state_encoded)
}

pub async fn oauth_finish(Query(oauth_finish): Query<OAuthFinish>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let auth_request: AuthRequest = dsrc.kv.get_json(&oauth_finish.flow_id).await?
        .ok_or(Error::BadFlow(ExpiredFlowId))?;

    let redirect_url = oauth_finish_redirect(auth_request, oauth_finish.code);

    Ok(Redirect::to(redirect_url.parse().unwrap()))
}

fn token_request_checks(redirect_uri_token: &str, redirect_uri_auth: &str, client_id_token: &str,
    client_id_auth: &str, code_verifier: &str, code_challenge_auth: &str) -> Result<(), Error> {
    if client_id_token != client_id_auth {
        return Err(Error::IncorrectField("client_id does not match".to_owned()))
    }
    if redirect_uri_token != redirect_uri_auth {
        return Err(Error::IncorrectField("redirect_uri does not match".to_owned()))
    }
    let ascii_encoded = ASCII.encode(code_verifier, EncoderTrap::Strict)
        .or_else(|e| Err(Error::BadFieldEncoding(e.to_string())))?;
    let computed_challenge_hash = Sha256::digest(&ascii_encoded);
    let challenge = base64::encode(computed_challenge_hash);

    if challenge != code_challenge_auth {
        return Err(Error::BadFlow(BadChallenge))
    }

    Ok(())
}

pub async fn token(Json(token_request): Json<TokenRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Json<TokenResponse>, Error> {
    // TODO clientID check

    let tokens = if token_request.grant_type == "authorization_code" {
        let redirect_uri_token = token_request.redirect_uri.ok_or(Error::MissingFieldTokenRequest)?;
        let code_verifier = token_request.code_verifier.ok_or(Error::MissingFieldTokenRequest)?;
        let code = token_request.code.ok_or(Error::MissingFieldTokenRequest)?;

        let flow_user: FlowUser = dsrc.kv.get_json(&code).await?
            .ok_or(Error::BadFlow(ExpiredCode))?;
        let auth_request: AuthRequest = dsrc.kv.get_json(&flow_user.flow_id).await?
            .ok_or(Error::BadFlow(ExpiredFlowId))?;

        token_request_checks(&redirect_uri_token, &auth_request.redirect_uri,
        &token_request.client_id, &auth_request.client_id, &code_verifier,
                                       &auth_request.code_challenge)?;

        new_token_family(&dsrc).await
    } else if token_request.grant_type == "refresh_token" {
        tracing::debug!("refresh_token request");
        let old_refresh_token = token_request.refresh_token.ok_or(Error::MissingFieldTokenRequest)?;

        refresh_all_tokens(&dsrc, old_refresh_token).await
    } else {
        Err(Error::IncorrectField("token_request only supports authorization_code and refresh_token".to_string()))
    }?;
    
    Ok(Json(TokenResponse{
        id_token: tokens.id_token,
        access_token: tokens.access_token,
        refresh_token: tokens.refresh_token,
        token_type: "Bearer".to_string(),
        expires_in: tokens::ACCESS_EXP as i32,
        scope: tokens.returned_scope
    }))
}