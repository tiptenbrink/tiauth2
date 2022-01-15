use std::sync::Arc;
use axum::extract::{Extension, Query};
use axum::Json;
use axum::response::{Redirect};
use crate::data::kv::KeyValue;
use crate::server::models::{AuthRequest, FlowUser, OAuthFinish, TokenRequest};
use crate::data::source::Source;
use crate::error::Error;
use crate::utility::random_time_hash_hex;
use url::form_urlencoded::{byte_serialize};

pub async fn oauth_endpoint(Query(auth_request): Query<AuthRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let flow_id = random_time_hash_hex(None);

    dsrc.kv.store_json(&flow_id, &auth_request, 1000).await?;

    Ok(Redirect::to(format!("/credentials?flow_id={}", flow_id).parse().unwrap()))
}

pub async fn oauth_finish(Query(oauth_finish): Query<OAuthFinish>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let auth_request: AuthRequest = dsrc.kv.get_json(&oauth_finish.flow_id).await?
        .ok_or(Error::FlowExpired)?;

    let query: String = byte_serialize(format!("?code={}&state={}", oauth_finish.code, auth_request.state).as_bytes()).collect();
    let redirect = if !auth_request.redirect_uri.ends_with("/") {
        format!("{}/", auth_request.redirect_uri)
    } else {
        auth_request.redirect_uri
    };
    Ok(Redirect::to(format!("{}{}", redirect, query).parse().unwrap()))
}

// pub async fn token(Json(token_request): Json<TokenRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Json<TokenResponse>, Error> {
//     // TODO clientID check
//
//     if token_request.grant_type == "authorization_code" {
//         let redirect_uri = token_request.redirect_uri.ok_or(Error::MissingFieldTokenRequest)?;
//         let code_verifier = token_request.code_verifier.ok_or(Error::MissingFieldTokenRequest)?;
//         let code = token_request.code.ok_or(Error::MissingFieldTokenRequest)?;
//
//         let flow_user: FlowUser = dsrc.kv.get_json(&code).await?
//             .ok_or(Error::FlowExpired)?;
//         let auth_request: AuthRequest = dsrc.kv.get_json(&flow_user.flow_id).await?
//             .ok_or(Error::FlowExpired)?;
//     }
//     Ok()
// }