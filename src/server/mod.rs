mod auth;
mod oauth;
mod models;
mod files;

use std::net::SocketAddr;
use std::sync::Arc;
use axum::{AddExtensionLayer, body, Router};
use axum::body::boxed;
use axum::http::{Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use files::serve_static;
use oauth::oauth_endpoint;
use crate::data::source::Source;
use crate::error::Error;
use crate::server::auth::{finish_login, finish_register, start_login, start_register};
use crate::server::oauth::{oauth_finish, token};
use tower_http::cors::{CorsLayer, any};
use tower_http::trace::TraceLayer;
use crate::auth::tokens::new_token_family;
use crate::data::refresh::{refresh_transaction, SavedRefreshToken};


pub async fn run_server() {
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var(
            "RUST_LOG",
            "tiauth2=debug,tower_http=debug",
        )
    }
    tracing_subscriber::fmt::init();

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
    let sv = SavedRefreshToken {
        id: 0,
        family_id: "abas".to_string(),
        access_value: "asdfas".to_string(),
        id_token_value: "asdfasd".to_string(),
        iat: 0,
        exp: 0,
        nonce: "asdfasdf".to_string()
    };
    
    let app = Router::new().route("/", get(|| async { "Hello, World!" }))
        .route("/oauth/authorize/", get(oauth_endpoint))
        .route("/oauth/callback/", get(oauth_finish))
        .route("/oauth/token/", post(token))
        .route("/login/start/", post(start_login))
        .route("/login/finish/", post(finish_login))
        .route("/register/start/", post(start_register))
        .route("/register/finish/", post(finish_register))
        .nest("/credentials", get(serve_static))
        .layer(TraceLayer::new_for_http())
        .layer(AddExtensionLayer::new(dsrc))
        .layer(cors);

    let addr = SocketAddr::from(([127, 0, 0, 1], 3073));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
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