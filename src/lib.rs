mod data;
mod error;

use axum::{
    body::{boxed},
    routing::get,
    Router,
};
use axum::body::{Body, BoxBody};
use axum::http::{Request, StatusCode, Uri};
use axum::response::Response;
use data::user;
use redis;
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use crate::data::kv::KeyValue;

use tower_http::services::ServeDir;

#[derive(Debug, Deserialize, Serialize)]
pub struct AuthJson {
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
    let app = Router::new().route("/", get(|| async { "Hello, World!" }))
        .nest("/credentials", get(handler));

    let db_uri = "postgres://dodeka:postpost@localhost:3141/dodeka";
    let kv_uri = "redis://127.0.0.1:6379";

    let dsrc = data::source::Source::new(db_uri, kv_uri).await.unwrap();
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

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:3037".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(uri: Uri) -> Result<Response<BoxBody>, (StatusCode, String)> {
    let res = get_static_file(uri.clone()).await?;

    if res.status() == StatusCode::NOT_FOUND {
        // try with `.html`
        // TODO: handle if the Uri has query parameters
        match format!("{}.html", uri).parse() {
            Ok(uri_html) => get_static_file(uri_html).await,
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Invalid URI".to_string())),
        }
    } else {
        Ok(res)
    }
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