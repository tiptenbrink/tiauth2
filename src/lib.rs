mod data;
mod error;
mod utility;

use std::collections::HashMap;
use axum::{
    AddExtensionLayer,
    Router,
};
use axum::body::{Body, BoxBody, boxed};
use axum::http::{Request, StatusCode, Uri};
use axum::response::{Response, Redirect, IntoResponse};
use axum::extract::{Query, Extension};
use axum::routing::get;
use data::user;
use redis;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use crate::data::kv::KeyValue;
use std::sync::Arc;

use tower_http::services::ServeDir;
use crate::data::source::Source;
use crate::error::Error;
use crate::utility::random_time_hash_hex;

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthRequest {
    pub response_type: String,
    pub client_id: String,
    pub redirect_uri: String,
    pub state: String,
    pub code_challenge: String,
    pub code_challenge_method: String,
    pub nonce: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TJson {
    pub cool: String
}

pub async fn run_server() {
    // build our application with a single route


    let db_uri = "postgres://dodeka:postpost@localhost:3141/dodeka";
    let kv_uri = "redis://127.0.0.1:6379";

    let data_source = Source::new(db_uri, kv_uri).await.unwrap();
    let dsrc = Arc::new(data_source);

    let app = Router::new().route("/", get(|| async { "Hello, World!" }))
        .route("/oauth/authorize/", get(oauth_endpoint))
        .nest("/credentials", get(serve_static))
        .layer(AddExtensionLayer::new(dsrc));
    // let mut u = user::get_user_by_id(&dsrc, 1).await.unwrap();
    // println!("{:?}", u);
    //let i = user::new_user(&dsrc, u).await.unwrap();
    //println!("{}", i);
    //u.password_file = "abc".to_owned();
    //user::upsert_user_row(&dsrc, u).await.unwrap();
    // let u = user::get_user_by_usph(&dsrc, "fakerecord").await.unwrap();
    // println!("{:?}", u);

    // let tj = TJson { cool: "mcool".to_string() };
    // dsrc.kv.store_json("sskey", &tj, 1000).await.unwrap();
    // let tj_rd: TJson = dsrc.kv.get_json("sskey").await.unwrap();
    // println!("{:?}", tj_rd);

    axum::Server::bind(&"127.0.0.1:3073".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn oauth_endpoint(auth_request: Query<AuthRequest>, Extension(dsrc): Extension<Arc<Source>>) -> Result<Redirect, Error> {
    let auth_request: AuthRequest = auth_request.0;

    let flow_id = random_time_hash_hex(None);

    dsrc.kv.store_json(&flow_id, &auth_request, 1000).await?;

    Ok(Redirect::to(format!("/credentials?flow_id={}", flow_id).parse().unwrap()))
}

async fn serve_static(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    Ok(res)
}

async fn get_static_file(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();

    // `ServeDir` implements `tower::Service` so we can call it with `tower::ServiceExt::oneshot`
    match ServeDir::new("credentials").oneshot(req).await {
        Ok(res) => Ok(res.map(boxed)),
        Err(err) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", err),
        )),
    }
}