mod auth;
mod oauth;
mod models;
mod files;

use std::sync::Arc;
use axum::{AddExtensionLayer, body, Router};
use axum::body::boxed;
use axum::http::{Method, StatusCode};
use axum::http::header::{CONTENT_TYPE, ACCEPT, CONTENT_LENGTH};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post, any as router_any};
use tower::ServiceBuilder;
use files::serve_static;
use oauth::oauth_endpoint;
use crate::data::kv::KeyValue;
use crate::data::source::Source;
use crate::error::Error;
use crate::server::auth::start_login;
use crate::server::models::TJson;
use tower_http::cors::{CorsLayer, any};

pub async fn run_server() {
    let db_uri = "postgres://dodeka:postpost@localhost:3141/dodeka";
    let kv_uri = "redis://127.0.0.1:6379";

    let data_source = Source::new(db_uri, kv_uri).await.unwrap();
    let dsrc = Arc::new(data_source);

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
    // let tj_rd: Option<TJson> = dsrc.kv.get_json("sskey").await.unwrap();
    // println!("{:?}", tj_rd);

    let cors = CorsLayer::new()
        // allow `GET` and `POST` when accessing the resource
        .allow_methods(vec![Method::POST, Method::GET, Method::OPTIONS])
        // allow requests from any origin
        .allow_origin(any())
        .allow_headers(any());
    
    let app = Router::new().route("/", get(|| async { "Hello, World!" }))
        .route("/oauth/authorize/", get(oauth_endpoint))
        .route("/login/start/", router_any(start_login))
        .nest("/credentials", get(serve_static))
        .layer(AddExtensionLayer::new(dsrc))
        .layer(cors);

    axum::Server::bind(&"127.0.0.1:3073".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let body = boxed(body::Full::from(format!("{:?}", self)));

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap()
    }
}