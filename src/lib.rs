mod data;
mod error;

use axum::{
    routing::get,
    Router,
};
use data::user;

pub async fn run_server() {
    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let db_uri = "postgres://dodeka:postpost@localhost:3141/dodeka";

    let psql = data::source::Source::new(db_uri).await.unwrap();
    let u = user::get_user_by_id(psql, 1).await.unwrap();
    println!("{:?}", u);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"127.0.0.1:3031".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}