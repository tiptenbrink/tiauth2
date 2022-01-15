use std::sync::Arc;
use axum::extract::{Extension, Query};
use axum::response::Redirect;
use crate::data::kv::KeyValue;
use crate::server::models::AuthRequest;
use crate::data::source::Source;
use crate::error::Error;
use crate::utility::random_time_hash_hex;

pub async fn oauth_endpoint(Query(auth_request): Query<AuthRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let flow_id = random_time_hash_hex(None);

    dsrc.kv.store_json(&flow_id, &auth_request, 1000).await?;

    Ok(Redirect::to(format!("/credentials?flow_id={}", flow_id).parse().unwrap()))
}